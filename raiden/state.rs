use std::io;
use std::any::Any;
use crate::traits::StateChange;
use crate::transfer::state::ChainState;
use crate::transfer::state_change::Block;

pub struct StateManager {
    current_state: Option<ChainState>
}

impl StateManager{
    pub fn new() -> StateManager {
        StateManager {
            current_state: None
        }
    }

    pub fn dispatch(&self, state_change: Box<dyn Any>) -> Result<bool, io::Error> {
        if let Ok(block) = state_change.downcast::<Block>() {
            println!("{:?}", block.block_number)
        }
        Ok(true)
    }
}
