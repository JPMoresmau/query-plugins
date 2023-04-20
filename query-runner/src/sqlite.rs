//! SQLLite implementation.

use anyhow::{anyhow, Result};
use serde_yaml::Value;

use crate::{add_result, parse::positional, DBConnection, ExecutionState, QueryResult, Variable};

use rusqlite::*;

use crate::query::*;

/// Create a new connection from a configuration value.
pub(crate) fn new_connection(value: Value) -> Result<DBConnection> {
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

/// Execute a query and return the result.
pub(crate) fn execute(
    connection: &Connection,
    state: &mut ExecutionState,
) -> Result<Option<QueryResult>> {
    // Get the query SQL.
    let query = state
        .query
        .execution_query_string(&mut state.store, &state.execution)?;
    // Get parameters.
    let params = state
        .query
        .execution_variables(&mut state.store, &state.execution)?;

    let query = positional("?", 1, &query, &params);

    // Prepare statement.
    let mut stmt = connection.prepare(&query).map_err(|op| anyhow!(op))?;
    // Bind parameters.
    for (idx, param) in params.iter().enumerate() {
        match &param.value {
            ValueResult::DataBoolean(Some(b)) => stmt.raw_bind_parameter(idx + 1, b)?,
            ValueResult::DataDecimal(Some(d)) => stmt.raw_bind_parameter(idx + 1, d)?,
            ValueResult::DataInteger(Some(i)) => stmt.raw_bind_parameter(idx + 1, i)?,
            ValueResult::DataString(Some(s)) => stmt.raw_bind_parameter(idx + 1, s)?,
            ValueResult::DataTimestamp(Some(t)) => stmt.raw_bind_parameter(idx + 1, t)?,
            // Unbound parameters default to NULL.
            _ => {}
        }
    }
    // Get columns name and type.
    let columns: Vec<(String, String)> = stmt
        .columns()
        .iter()
        .map(|c| {
            (
                c.name().to_string(),
                c.decl_type().unwrap_or_default().to_string(),
            )
        })
        .collect();
    let mut rows = stmt.raw_query();
    // final result.
    let mut result = Option::None;
    // Loop through all rows.
    while let Some(row) = rows.next()? {
        // Build row.
        let mut result_one = Vec::with_capacity(columns.len());
        for (ix, (name, typ)) in columns.iter().enumerate() {
            match typ.as_str() {
                "INTEGER" => result_one.push(Variable {
                    name,
                    value: ValueResult::DataInteger(row.get(ix)?),
                }),
                "TEXT" => result_one.push(Variable {
                    name,
                    value: ValueResult::DataString(row.get(ix)?),
                }),
                "BOOL" => result_one.push(Variable {
                    name,
                    value: ValueResult::DataBoolean(row.get(ix)?),
                }),
                "REAL" => result_one.push(Variable {
                    name,
                    value: ValueResult::DataDecimal(row.get(ix)?),
                }),
                _ => return Err(anyhow!("unsupported type {typ}")),
            }
        }
        // Send row to plugin.
        let res = state.row(result_one)?;
        result = add_result(result, res);
    }
    // End.
    let names: Vec<&str> = columns.iter().map(|(n, _)| n.as_str()).collect();
    let end = state
        .query
        .execution_end(&mut state.store, &state.execution, &names)?;
    Ok(add_result(result, end))
}
