use std::io;
use crate::state_change::StateChange;
use crate::transfer::state::ChainState;
use crate::transfer::chain::{ChainTransitionResult, handle_state_change};

#[derive(Default)]
pub struct StateManager {
    current_state: Option<ChainState>
}

impl StateManager{
    pub fn init_state(&mut self) -> Result<bool, io::Error> {
        self.current_state = Some(ChainState {
            block_number: web3::types::BlockNumber::Earliest
        });
        Ok(true)
    }

    pub fn restore_state(&self) -> Result<bool, io::Error> {
        Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid state"))
    }

    pub fn dispatch(&mut self, state_change: StateChange) -> Result<bool, io::Error> {
        let transition: Result<ChainTransitionResult, io::Error> = handle_state_change(self.current_state.unwrap(), state_change);
        let result = match transition {
            Ok(transition_result) => {
                self.current_state = Some(transition_result.new_state);
                Ok(true)
            },
            Err(_e) => Err(io::Error::new(io::ErrorKind::InvalidData, "Opps"))
        };
        result
    }
}
