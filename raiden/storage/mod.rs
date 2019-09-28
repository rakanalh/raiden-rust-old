extern crate rusqlite;

use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result as SQLiteResult};
use std::result::Result;

use crate::errors::SerializationError;
use crate::state_change::StateChange;

pub fn setup_database(conn: &Connection) -> SQLiteResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS state (
             id integer primary key,
             name text not null unique
         )",
        NO_PARAMS,
    )?;

    Ok(())
}

pub fn store_state_change(
    conn: &Connection,
    state_change: StateChange,
) -> Result<bool, SerializationError> {
    let serialized_state_change = serde_json::to_string(&state_change);
    Ok(true)
}
