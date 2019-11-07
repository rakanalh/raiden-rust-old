use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::transfer::state_change::{
    ActionInitChain, Block, ContractReceiveChannelOpened, ContractReceiveTokenNetworkCreated,
    ContractReceiveTokenNetworkRegistry,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ChainID {
    Mainnet = 1,
    Ropsten = 3,
    Rinkeby = 4,
    Goerli = 5,
    Kovan = 42,
}

impl FromStr for ChainID {
    type Err = ();

    fn from_str(s: &str) -> Result<ChainID, ()> {
        match s {
            "mainnet" => Ok(ChainID::Mainnet),
            "ropsten" => Ok(ChainID::Ropsten),
            "rinkeby" => Ok(ChainID::Rinkeby),
            "goerli" => Ok(ChainID::Goerli),
            "kovan" => Ok(ChainID::Kovan),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum StateChange {
    Block(Block),
    ActionInitChain(ActionInitChain),
    ContractReceiveTokenNetworkRegistry(ContractReceiveTokenNetworkRegistry),
    ContractReceiveTokenNetworkCreated(ContractReceiveTokenNetworkCreated),
    ContractReceiveChannelOpened(ContractReceiveChannelOpened),
}
