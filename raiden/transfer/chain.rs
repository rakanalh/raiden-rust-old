use crate::errors::StateTransitionError;
use crate::events::Event;
use crate::state_change::StateChange;
use crate::transfer::state::ChainState;
use crate::transfer::state_change::{Block, ContractReceiveTokenNetworkRegistry};

pub struct ChainTransitionResult {
    pub new_state: ChainState,
    pub events: Vec<Event>,
}

fn handle_new_block(
    mut chain_state: ChainState,
    state_change: Block,
) -> Result<ChainTransitionResult, StateTransitionError> {
    chain_state.block_number = state_change.block_number;
    Ok(ChainTransitionResult {
        new_state: chain_state,
        events: vec![],
    })
}

fn handle_contract_receive_token_network_registry(
    chain_state: ChainState,
    state_change: ContractReceiveTokenNetworkRegistry,
) -> Result<ChainTransitionResult, StateTransitionError> {
    Err(StateTransitionError {})
}

pub fn handle_state_change(
    chain_state: ChainState,
    state_change: StateChange,
) -> Result<ChainTransitionResult, StateTransitionError> {
    let new_state = chain_state;
    let result: Result<ChainTransitionResult, StateTransitionError> = match state_change {
        StateChange::Block(block) => handle_new_block(new_state, block),
        StateChange::ContractReceiveTokenNetworkRegistry(
            contract_receive_token_network_registry,
        ) => handle_contract_receive_token_network_registry(
            new_state,
            contract_receive_token_network_registry,
        ),
    };
    result
}
