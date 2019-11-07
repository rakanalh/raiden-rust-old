extern crate tokio;
extern crate tokio_core;
extern crate web3;

use crate::blockchain::contracts;
use crate::blockchain::events;
use crate::enums::{ChainID, StateChange};
use crate::state::StateManager;
use crate::storage;
use crate::transfer;
use crate::transfer::state::TokenNetworkRegistryState;
use crate::transfer::state_change::{ActionInitChain, ContractReceiveTokenNetworkRegistry};
use ethsign::SecretKey;
use futures::IntoFuture;
use rusqlite::Connection;
use std::cell::RefCell;
use std::process;
use std::rc::Rc;
use tokio_core::reactor;
use web3::futures::{Future, Stream};
use web3::transports::WebSocket;
use web3::types::{Address, H256, U64};

pub struct RaidenService {
    pub chain_id: ChainID,
    pub our_address: Address,
    pub secret_key: SecretKey,
    state_manager: Rc<RefCell<StateManager>>,
    contracts_registry: Rc<contracts::abi::ContractRegistry>,
}

impl RaidenService {
    pub fn new(chain_id: ChainID, our_address: Address, secret_key: SecretKey) -> RaidenService {
        let conn = match Connection::open("raiden.db") {
            Ok(conn) => Rc::new(conn),
            Err(e) => {
                eprintln!("Could not connect to database: {}", e);
                process::exit(1)
            }
        };

        if let Err(e) = storage::setup_database(&conn) {
            eprintln!("Could not setup database: {}", e);
            process::exit(1)
        }

        let state_manager = StateManager::new(Rc::clone(&conn));
        let contracts_registry = contracts::abi::ContractRegistry::default();
        RaidenService {
            chain_id,
            our_address,
            secret_key,
            contracts_registry: Rc::new(contracts_registry),
            state_manager: Rc::new(RefCell::new(state_manager)),
        }
    }

    pub fn start(&self, eloop: &reactor::Core) {
        let state_manager = self.state_manager.clone();
        let mut initialize = false;
        if let Err(_e) = state_manager.borrow_mut().restore_state() {
            initialize = true;
        }

        if initialize {
            let init_chain = ActionInitChain {
                chain_id: self.chain_id.clone(),
                block_number: U64::from(1),
                our_address: self.our_address,
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
                println!("Failed to transition: {}", e);
            }
        }

        self.install_filters();
        self.poll_filters(&eloop);
        println!(
            "Chain State {:?}",
            self.state_manager.borrow().current_state
        );
        let ws_subscription = self.run_blocks_monitor(&eloop);
        eloop.handle().spawn(ws_subscription);
        //drop(web3);
    }

    fn transition(&self, state_change: StateChange) {
        let transition_result =
            StateManager::transition(self.state_manager.clone(), state_change.clone());
        if let Err(e) = transition_result {
            println!("Failed to transition: {}", e);
        }

        self.after_state_change(state_change);
    }

    fn after_state_change(&self, state_change: StateChange) {
        match state_change {
            StateChange::ContractReceiveTokenNetworkCreated(state_change) => {
                let token_network_address = state_change.token_network.address;
                self.contracts_registry.create_contract_event_filters(
                    "TokenNetwork".to_string(),
                    token_network_address,
                );
            }
            _ => (),
        }
    }

    fn install_filters(&self) {
        let registry_address: Address = "8CA88eF59acd4C0810f2b6a418Fe7e3efdbAA020"
            .to_string()
            .parse()
            .unwrap();
        self.contracts_registry
            .create_contract_event_filters("TokenNetworkRegistry".to_string(), registry_address);
    }

    pub fn poll_filters(&self, eloop: &reactor::Core) {
        let infura_http = "https://kovan.infura.io/v3/6fdc99560fce488cba4a52b6c8c0574b";

        let web3 = web3::Web3::new(
            web3::transports::Http::with_event_loop(infura_http, &eloop.handle(), 1).unwrap(),
        );

        for (_, contract_filters) in self.contracts_registry.filters.borrow().iter() {
            for filter in contract_filters.values() {
                let current_state = self.state_manager.borrow().current_state.clone();
                let contracts_registry = self.contracts_registry.clone();
                let state_manager = self.state_manager.clone();
                let event_future = web3
                    .eth()
                    .logs((*filter).clone())
                    .into_future()
                    .and_then(move |logs| {
                        for log in logs {
                            if let Some(state_change) = events::log_to_blockchain_state_change(
                                &current_state,
                                &contracts_registry,
                                &log,
                            ) {
                                println!("State transition {:#?}", state_change);
                                let _ =
                                    StateManager::transition(state_manager.clone(), state_change);
                            }
                        }
                        futures::future::ok(())
                    })
                    .map_err(|e| println!("Error {}", e));

                let _result = eloop.handle().spawn(event_future);
            }
        }
    }

    pub fn run_blocks_monitor(
        &self,
        eloop: &reactor::Core,
    ) -> impl futures::Future<Item = (), Error = ()> + 'static {
        println!("Connecting websocket");
        let infura_ws = "wss://kovan.infura.io/ws/v3/6fdc99560fce488cba4a52b6c8c0574b";
        let ws = WebSocket::with_event_loop(infura_ws, &eloop.handle()).unwrap();
        let web3 = web3::Web3::new(ws.clone());
        println!("Connected");
        let state_manager = self.state_manager.clone();
        let chain_id = self.chain_id.clone();
        Box::new(
            web3.eth_subscribe()
                .subscribe_new_heads()
                .and_then(move |sub| {
                    sub.for_each(move |block| {
                        let block_number = block.number.unwrap();

                        println!("Received block: {}", block_number);

                        let block_state_change =
                            transfer::state_change::Block::new(chain_id.clone(), block_number);

                        let _ = StateManager::transition(
                            state_manager.clone(),
                            StateChange::Block(block_state_change),
                        );

                        Ok(())
                    })
                })
                .map_err(|e| eprintln!("Error fetching block {}", e))
                .into_future(),
        )
    }
}
