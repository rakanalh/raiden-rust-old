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
use rusqlite::Connection;
use std::process;
use std::rc::Rc;
use web3::api::SubscriptionStream;
use web3::futures::{Future, Stream};
use web3::transports::WebSocket;
use web3::types::{Address, BlockHeader, H256, U64};
use web3::Web3;

pub struct RaidenService {
    pub chain_id: ChainID,
    pub our_address: Address,
    pub secret_key: SecretKey,
    state_manager: StateManager,
    contracts_registry: contracts::abi::ContractRegistry,
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
            state_manager,
            contracts_registry,
        }
    }

    pub fn start(&mut self) {
        if let Err(_e) = self.state_manager.restore_state() {
            let init_chain = ActionInitChain {
                chain_id: self.chain_id.clone(),
                block_number: U64::from(1),
                our_address: self.our_address,
            };

            if let Err(e) = self
                .state_manager
                .transition(StateChange::ActionInitChain(init_chain))
            {
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

            let transition_result =
                self.state_manager
                    .transition(StateChange::ContractReceiveTokenNetworkRegistry(
                        new_network_registry_state_change,
                    ));
            if let Err(e) = transition_result {
                println!("Failed to transition: {}", e);
            }
        };

        self.install_filters();
        self.poll_filters();
        self.poll_filters();
        println!("Chain State {:?}", self.state_manager.current_state);
        //let (web3, ws_subscription) = self.run_blocks_monitor();
        //ws_subscription.unsubscribe();
        //drop(web3);
    }

    fn transition(&mut self, state_change: StateChange) {
        let transition_result = self.state_manager.transition(state_change.clone());
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

    fn install_filters(&mut self) {
        let registry_address: Address = "8CA88eF59acd4C0810f2b6a418Fe7e3efdbAA020"
            .to_string()
            .parse()
            .unwrap();
        self.contracts_registry
            .create_contract_event_filters("TokenNetworkRegistry".to_string(), registry_address);
    }

    pub fn poll_filters(&mut self) {
        let infura_http = "https://kovan.infura.io/v3/6fdc99560fce488cba4a52b6c8c0574b";
        let mut eloop = tokio_core::reactor::Core::new().unwrap();
        let web3 = web3::Web3::new(
            web3::transports::Http::with_event_loop(infura_http, &eloop.handle(), 1).unwrap(),
        );

        let mut state_changes = vec![];
        for (_, contract_filters) in self.contracts_registry.filters.borrow().iter() {
            for filter in contract_filters.values() {
                let event_future = web3.eth().logs((*filter).clone());
                let result = eloop.run(event_future);

                if let Ok(logs) = result {
                    for log in logs {
                        if let Some(state_change) = events::log_to_blockchain_state_change(
                            self.state_manager.current_state.as_ref().unwrap(),
                            &self.contracts_registry,
                            &log,
                        ) {
                            state_changes.push(state_change);
                        }
                    }
                }
            }
        }

        for state_change in state_changes {
            self.transition(state_change);
        }
    }

    pub fn run_blocks_monitor(
        &mut self,
    ) -> (Web3<WebSocket>, SubscriptionStream<WebSocket, BlockHeader>) {
        let infura_ws = "wss://kovan.infura.io/ws/v3/6fdc99560fce488cba4a52b6c8c0574b";
        let (_eloop, ws) = WebSocket::new(infura_ws).unwrap();
        let web3 = web3::Web3::new(ws.clone());

        let mut sub = web3.eth_subscribe().subscribe_new_heads().wait().unwrap();

        (&mut sub)
            .for_each(|block| {
                let block_number = block.number.unwrap();

                println!("Received block: {}", block_number);

                let block_state_change =
                    transfer::state_change::Block::new(self.chain_id.clone(), block_number);

                let _ = self
                    .state_manager
                    .transition(StateChange::Block(block_state_change));

                Ok(())
            })
            .wait()
            .unwrap();

        (web3, sub)
    }
}
