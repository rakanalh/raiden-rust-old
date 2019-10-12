use web3::types::Address;
use crate::enums::StateChange;
use crate::errors;
use crate::storage;
use crate::transfer::chain::{self, ChainTransitionResult};
use crate::transfer::state::ChainState;
use std::rc::Rc;
use std::result;

type Result<T> = result::Result<T, errors::StateTransitionError>;

pub struct StateManager {
    dbconn: Rc<rusqlite::Connection>,
    pub current_state: Option<ChainState>,
}

impl StateManager {
    pub fn new(dbconn: Rc<rusqlite::Connection>) -> StateManager {
        StateManager {
            dbconn,
            current_state: None,
        }
    }

    pub fn restore_state(&self) -> result::Result<bool, errors::RaidenError> {
        Err(errors::RaidenError {
            msg: String::from("Invalid state"),
        })
    }

    pub fn transition(&mut self, state_change: StateChange) -> Result<bool> {
        match self.store_state_change(state_change.clone()) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }?;
        self.dispatch(state_change)
    }

    fn dispatch(&mut self, state_change: StateChange) -> Result<bool> {
        let current_state = self.current_state.clone();
        
        let transition: Result<ChainTransitionResult> =
            chain::handle_state_change(current_state, state_change);

        match transition {
            Ok(transition_result) => {
                self.current_state.replace(transition_result.new_state);
                Ok(true)
            }
            Err(e) => Err(errors::StateTransitionError {
                msg: format!("Could not transition: {}", e),
            }),
        }
    }

    fn store_state_change(&self, state_change: StateChange) -> Result<bool> {
        match storage::store_state_change(&self.dbconn, state_change) {
            Ok(result) => Ok(result),
            Err(e) => Err(errors::StateTransitionError {
                msg: format!("Could not store state change: {}", e),
            }),
        }
    }
}
