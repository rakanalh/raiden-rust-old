use serde::{Deserialize, Serialize};
use web3::types::{Address, H256, U64};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Block {
    pub chain_id: u32,
    pub block_number: u64,
}

impl Block {
    pub fn new(chain_id: u32, block_number: u64) -> Block {
        Block {
            chain_id,
            block_number,
        }
    }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct ContractReceiveTokenNetworkRegistry {
    pub transaction_hash: Option<H256>,
    pub token_network_registry: Address,
    pub block_number: U64,
    pub block_hash: H256,
}

impl ContractReceiveTokenNetworkRegistry {
    pub fn new(
        transaction_hash: H256,
        token_network_registry: Address,
        block_number: U64,
        block_hash: H256,
    ) -> Self {
        ContractReceiveTokenNetworkRegistry {
            transaction_hash: Some(transaction_hash),
            token_network_registry,
            block_number,
            block_hash,
        }
    }
}
