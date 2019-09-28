use serde::{Deserialize, Serialize};

use crate::transfer::state_change::{Block, ContractReceiveTokenNetworkRegistry};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum StateChange {
    Block(Block),
    ContractReceiveTokenNetworkRegistry(ContractReceiveTokenNetworkRegistry),
}
