use std::io;
use crate::events::Event;
use crate::state_change::StateChange;
use crate::transfer::state_change::Block;
use crate::transfer::state::ChainState;

pub struct ChainTransitionResult {
    pub new_state: ChainState,
    pub events: Vec<Event>,
}

fn handle_new_block(mut chain_state: ChainState, state_change: Block) -> Result<ChainTransitionResult, io::Error> {
    chain_state.block_number = state_change.block_number;
    Ok(ChainTransitionResult {
        new_state: chain_state,
        events: vec![],
    })
}

pub fn handle_state_change(chain_state: ChainState, state_change: StateChange) -> Result<ChainTransitionResult, io::Error> {
    let new_state = chain_state.clone();
    let result: Result<ChainTransitionResult, io::Error> = match state_change {
        StateChange::Block(block) => {
            handle_new_block(new_state, block)
        }
    };
    result
}
