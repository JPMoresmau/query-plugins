use anyhow::{anyhow, Result};
use serde_yaml::Value;

use crate::DBConnection;

use rusqlite::*;

pub fn new_connection(value: Value) -> Result<DBConnection> {
    let path = value
        .get("path")
        .ok_or(anyhow!("No path provided"))?
        .as_str()
        .ok_or(anyhow!("path is not a string"))?;
    if path == "memory" {
        Ok(DBConnection::SqliteConnection(Connection::open_in_memory()?))
    } else {
        Ok(DBConnection::SqliteConnection(Connection::open(path)?))
    }
}
