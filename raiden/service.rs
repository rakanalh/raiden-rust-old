use crate::blockchain::contracts;
use crate::state::StateManager;
use crate::state_change::StateChange;
use crate::storage;
use crate::transfer;
use crate::transfer::state::TokenNetworkRegistryState;
use crate::transfer::state_change::ContractReceiveTokenNetworkRegistry;
use rusqlite::Connection;
use std::process;
use std::rc::Rc;
use web3::api::SubscriptionStream;
use web3::futures::{Future, Stream};
use web3::transports::WebSocket;
use web3::types::BlockHeader;
use web3::types::{H256, U64};
use web3::Web3;

pub struct RaidenService {
    dbconn: Rc<Connection>,
    state_manager: StateManager,
}

impl RaidenService {
    pub fn default() -> RaidenService {
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
        RaidenService {
            dbconn: conn,
            state_manager,
        }
    }

    pub fn start(&mut self) {
        if let Err(_e) = self.state_manager.restore_state() {
            let _res = self.state_manager.init_state();

            let block_number = U64::one();
            let block_hash = H256::zero();
            let token_network_registry = contracts::get_token_network_registry_address();

            let contract_receive_token_network_registry = ContractReceiveTokenNetworkRegistry {
                transaction_hash: None,
                token_network_registry,
                block_number,
                block_hash,
            };

            let last_log_block_number = U64::from(0);
            let last_log_block_hash = H256::zero();

            let token_network_registry = TokenNetworkRegistryState::default();
            let new_network_state_change = ContractReceiveTokenNetworkRegistry::new(
                H256::zero(),
                token_network_registry.address,
                last_log_block_number,
                last_log_block_hash,
            );

            let transition_result =
                self.state_manager
                    .transition(StateChange::ContractReceiveTokenNetworkRegistry(
                        contract_receive_token_network_registry,
                    ));
            if let Err(e) = transition_result {
                println!("Failed to transition: {}", e);
            }
        };

        let (web3, ws_subscription) = self.run_blocks_monitor();
        ws_subscription.unsubscribe();
        drop(web3);
    }

    pub fn run_blocks_monitor(
        &mut self,
    ) -> (Web3<WebSocket>, SubscriptionStream<WebSocket, BlockHeader>) {
        let (_eloop, ws) = WebSocket::new("ws://47.100.34.71:8547").unwrap();
        let web3 = web3::Web3::new(ws.clone());

        let mut sub = web3.eth_subscribe().subscribe_new_heads().wait().unwrap();

        (&mut sub)
            .for_each(|block| {
                let block_number = block.number.unwrap().as_u64();

                println!("Received block: {}", block_number);

                let block_state_change = transfer::state_change::Block::new(1, block_number);

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
