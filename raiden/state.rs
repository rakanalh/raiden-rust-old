use crate::enums::Event;
use crate::enums::StateChange;
use crate::errors;
use crate::storage;
use crate::transfer::chain::{self, ChainTransition};
use crate::transfer::state::ChainState;
use std::result;
use std::sync::{Arc, Mutex, RwLockWriteGuard};

pub type Result<T> = result::Result<T, errors::StateTransitionError>;

pub struct StateManager {
    dbconn: Arc<Mutex<rusqlite::Connection>>,
    pub current_state: Option<ChainState>,
}

impl StateManager {
    pub fn new(dbconn: Arc<Mutex<rusqlite::Connection>>) -> StateManager {
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

    fn dispatch(&mut self, state_change: StateChange) -> Result<Vec<Event>> {
        let current_state = self.current_state.clone();

        let transition: Result<ChainTransition> = chain::state_transition(current_state, state_change);

        match transition {
            Ok(transition_result) => {
                self.current_state.replace(transition_result.new_state);
                Ok(transition_result.events)
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

    pub fn transition(mut manager: RwLockWriteGuard<StateManager>, state_change: StateChange) -> Result<Vec<Event>> {
        match manager.store_state_change(state_change.clone()) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }?;
        let result = manager.dispatch(state_change.clone());

        result
    }
}
