extern crate tokio;
extern crate tokio_core;
extern crate web3;

use crate::blockchain::contracts;
use crate::blockchain::events;
use crate::cli;
use crate::enums::{ChainID, StateChange};
use crate::state::StateChangeCallback;
use crate::state::StateManager;
use crate::storage;
use crate::transfer;
use crate::transfer::state::TokenNetworkRegistryState;
use crate::transfer::state_change::{ActionInitChain, ContractReceiveTokenNetworkRegistry};
use ethsign::SecretKey;
use futures::IntoFuture;
use rusqlite::Connection;
use slog::Logger;
use std::cell::RefCell;
use std::process;
use std::sync::{Arc, Mutex};
use tokio_core::reactor;
use web3::futures::{Future, Stream};
use web3::transports::WebSocket;
use web3::types::{Address, H256, U64};

#[derive(Clone)]
pub struct RaidenService {
    pub chain_id: Arc<ChainID>,
    pub our_address: Arc<Address>,
    pub secret_key: Arc<SecretKey>,
    state_manager: Arc<RefCell<StateManager>>,
    contracts_registry: Arc<contracts::abi::ContractRegistry>,
    log: Logger,
}

impl StateChangeCallback for RaidenService {
    fn on_state_change(&self, state_change: StateChange) {
        self.handle_state_change(state_change);
    }
}

impl RaidenService {
    pub fn new(chain_id: ChainID, our_address: Address, secret_key: SecretKey, log: Logger) -> RaidenService {
        let conn = match Connection::open("raiden.db") {
            Ok(conn) => Arc::new(Mutex::new(conn)),
            Err(e) => {
                crit!(log, "Could not connect to database: {}", e);
                process::exit(1)
            }
        };

        if let Err(e) = storage::setup_database(&conn.lock().unwrap()) {
            crit!(log, "Could not setup database: {}", e);
            process::exit(1)
        }

        let state_manager = StateManager::new(Arc::clone(&conn));
        let contracts_registry = contracts::abi::ContractRegistry::default();
        RaidenService {
            chain_id: Arc::new(chain_id),
            our_address: Arc::new(our_address),
            secret_key: Arc::new(secret_key),
            contracts_registry: Arc::new(contracts_registry),
            state_manager: Arc::new(RefCell::new(state_manager)),
            log: log,
        }
    }

    pub fn initialize(&self, config: &cli::Config) {
        let state_manager = self.state_manager.clone();
        let mut initialize = false;
        if let Err(_e) = state_manager.borrow_mut().restore_state() {
            initialize = true;
        }

        if initialize {
            let init_chain = ActionInitChain {
                chain_id: self.chain_id.as_ref().clone(),
                block_number: U64::from(1),
                our_address: self.our_address.as_ref().clone(),
            };
            if let Err(e) = StateManager::transition(
                self.state_manager.clone(),
                StateChange::ActionInitChain(init_chain),
            ) {
                panic!(format!("Could not initialize chain state: {}", e));
            }

            let token_network_registry_address = contracts::get_token_network_registry_address();
            let token_network_registry =
                TokenNetworkRegistryState::new(token_network_registry_address, vec![]);

            let last_log_block_number = U64::from(1);
            let last_log_block_hash = H256::zero();

            let new_network_registry_state_change = ContractReceiveTokenNetworkRegistry::new(
                H256::zero(),
                token_network_registry,
                last_log_block_number,
                last_log_block_hash,
            );

            let transition_result = StateManager::transition(
                self.state_manager.clone(),
                StateChange::ContractReceiveTokenNetworkRegistry(new_network_registry_state_change),
            );
            if let Err(e) = transition_result {
                warn!(self.log, "Failed to transition: {}", e);
            }
        }
        let service = self.clone();
        self.state_manager
            .borrow_mut()
            .register_callback(Box::new(service));

        self.install_filters();
        self.poll_filters(config.eth_http_rpc_endpoint.clone());
        //self.poll_filters(config.eth_http_rpc_endpoint.clone());
    }

    pub fn start(&self, handle: &reactor::Handle, config: cli::Config) {
        debug!(self.log,
            "Chain State {:?}",
            self.state_manager.borrow().current_state
        );
        self.run_blocks_monitor(handle, config.eth_socket_rpc_endpoint);
    }

    fn handle_state_change(&self, state_change: StateChange) {
        match state_change {
            StateChange::ContractReceiveTokenNetworkCreated(state_change) => {
                let token_network_address = state_change.token_network.address;
                self.contracts_registry.create_contract_event_filters(
                    "TokenNetwork".to_string(),
                    token_network_address,
                );
            }
            StateChange::Block(state_change) => {}
            _ => (),
        }
    }

    fn install_filters(&self) {
        let token_network_registry_address = contracts::get_token_network_registry_address();
        self.contracts_registry.create_contract_event_filters(
            "TokenNetworkRegistry".to_string(),
            token_network_registry_address,
        );
    }

    pub fn poll_filters(&self, http_rpc_endpoint: String) {
        let mut eloop = tokio_core::reactor::Core::new().unwrap();

        let web3 = web3::Web3::new(
            web3::transports::Http::with_event_loop(&http_rpc_endpoint, &eloop.handle(), 1)
                .unwrap(),
        );
        let filters = self.contracts_registry.filters.clone();
        for (_, contract_filters) in filters.borrow().iter() {
            for filter in contract_filters.values() {
                let current_state = self.state_manager.borrow().current_state.clone();
                let contracts_registry = self.contracts_registry.clone();
                let state_manager = self.state_manager.clone();

                let event_future = web3.eth().logs((*filter).clone());
                if let Ok(logs) = eloop.run(event_future) {
                    for log in logs {
                        if let Some(state_change) = events::log_to_blockchain_state_change(
                            &current_state,
                            &contracts_registry,
                            &log,
                        ) {
                            debug!(self.log, "State transition {:#?}", state_change);
                            let _ = StateManager::transition(state_manager.clone(), state_change);
                        }
                    }
                }
            }
        }
    }

    pub fn run_blocks_monitor(&self, handle: &reactor::Handle, eth_socket_rpc_endpoint: String) {
        let ws = WebSocket::with_event_loop(&eth_socket_rpc_endpoint, handle).unwrap();
        let web3 = web3::Web3::new(ws.clone());
        let state_manager = self.state_manager.clone();
        let chain_id = self.chain_id.clone();
        let log = self.log.clone();

        let sockets_future = web3
            .eth_subscribe()
            .subscribe_new_heads()
            .and_then(move |sub| {
                sub.for_each(move |block| {
                    let block_number = block.number.unwrap();

                    debug!(log.clone(), "Received block: {}", block_number);

                    let block_state_change =
                        transfer::state_change::Block::new(chain_id.as_ref().clone(), block_number);

                    let _ = StateManager::transition(
                        state_manager.clone(),
                        StateChange::Block(block_state_change),
                    );

                    Ok(())
                })
            })
            .map_err(|e| eprintln!("Error fetching block {}", e))
            .into_future();
        handle.spawn(sockets_future);
    }
}
