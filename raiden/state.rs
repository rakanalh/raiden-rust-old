use crate::errors;
use crate::state_change::StateChange;
use crate::storage;
use crate::transfer::chain::{handle_state_change, ChainTransitionResult};
use crate::transfer::state::ChainState;
use std::rc::Rc;
use std::result;

type Result<T> = result::Result<T, errors::StateTransitionError>;

pub struct StateManager {
    dbconn: Rc<rusqlite::Connection>,
    current_state: Option<ChainState>,
}

impl StateManager {
    pub fn new(dbconn: Rc<rusqlite::Connection>) -> StateManager {
        StateManager {
            dbconn,
            current_state: None,
        }
    }

    pub fn init_state(&mut self) -> result::Result<bool, errors::RaidenError> {
        self.current_state = Some(ChainState { block_number: 1 });
        Ok(true)
    }

    pub fn restore_state(&self) -> result::Result<bool, errors::RaidenError> {
        Err(errors::RaidenError {
            msg: String::from("Invalid state"),
        })
    }

    pub fn transition(&mut self, state_change: StateChange) -> Result<bool> {
        match self.store_state_change(state_change) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }?;
        self.dispatch(state_change)
    }

    fn dispatch(&mut self, state_change: StateChange) -> Result<bool> {
        let current_state = self.current_state.clone().unwrap();

        let transition: Result<ChainTransitionResult> =
            handle_state_change(current_state, state_change);

        match transition {
            Ok(transition_result) => {
                self.current_state.replace(transition_result.new_state);
                Ok(true)
            }
            Err(_e) => Err(errors::StateTransitionError {}),
        }
    }

    fn store_state_change(&self, state_change: StateChange) -> Result<bool> {
        match storage::store_state_change(&self.dbconn, state_change) {
            Ok(result) => Ok(result),
            Err(_) => Err(errors::StateTransitionError {}),
        }
    }
}
