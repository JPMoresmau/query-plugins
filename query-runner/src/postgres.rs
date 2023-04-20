use anyhow::{anyhow, Result};
use postgres::fallible_iterator::FallibleIterator;
use postgres::types::{ToSql, Type};
use postgres::{Config, NoTls};
use serde_yaml::Value;

use crate::VariableResult;
use crate::{
    add_result, parse::positional, DBConnection, ExecutionState, QueryResult, ValueResult, Variable,
};

/// Create a new connection from a configuration value.
pub(crate) fn new_connection(value: Value) -> Result<DBConnection> {
    let config = value
        .get("config")
        .ok_or(anyhow!("No config provided"))?
        .as_str()
        .ok_or(anyhow!("config is not a string"))?;

    Ok(DBConnection::PostgresConnection(Box::new(config.parse()?)))
}

/// Execute a query and return the result.
pub(crate) fn execute(config: &Config, state: &mut ExecutionState) -> Result<Option<QueryResult>> {
    // Get the query SQL.
    let query = state
        .query
        .execution_query_string(&mut state.store, &state.execution)?;
    // Get parameters.
    let params = state
        .query
        .execution_variables(&mut state.store, &state.execution)?;

    let query = positional("$", 1, &query, &params);

    let mut client = config.connect(NoTls)?;
    let stmt = client.prepare(&query)?;
    let columns = stmt.columns();

    let mut it = client.query_raw(&stmt, &params)?;
    // final result.
    let mut result = Option::None;
    while let Some(row) = it.next()? {
        // Build row.
        let mut result_one = Vec::with_capacity(columns.len());
        for (ix, col) in columns.iter().enumerate() {
            match col.type_() {
                &Type::INT2 | &Type::INT4 | &Type::INT8 => result_one.push(Variable {
                    name: col.name(),
                    value: ValueResult::DataInteger(row.get(ix)),
                }),
                &Type::TEXT => result_one.push(Variable {
                    name: col.name(),
                    value: ValueResult::DataString(row.get(ix)),
                }),
                &Type::BOOL => result_one.push(Variable {
                    name: col.name(),
                    value: ValueResult::DataBoolean(row.get(ix)),
                }),
                &Type::FLOAT4 | &Type::FLOAT8 => result_one.push(Variable {
                    name: col.name(),
                    value: ValueResult::DataDecimal(row.get(ix)),
                }),
                _ => return Err(anyhow!("unsupported type {}", col.type_())),
            }
        }
        // Send row to plugin.
        let res = state.row(result_one)?;
        result = add_result(result, res);
    }

    // End.
    let names: Vec<&str> = columns.iter().map(|c| c.name()).collect();
    let end = state
        .query
        .execution_end(&mut state.store, &state.execution, &names)?;
    Ok(add_result(result, end))
}

impl ToSql for VariableResult {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut postgres::types::private::BytesMut,
    ) -> std::result::Result<postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        match &self.value {
            ValueResult::DataBoolean(b) => b.to_sql(ty, out),
            ValueResult::DataDecimal(b) => b.to_sql(ty, out),
            ValueResult::DataInteger(b) => b.to_sql(ty, out),
            ValueResult::DataString(b) => b.to_sql(ty, out),
            ValueResult::DataTimestamp(b) => b.to_sql(ty, out),
        }
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized,
    {
        bool::accepts(ty) || f64::accepts(ty) || i64::accepts(ty) || String::accepts(ty)
    }

    fn encode_format(&self, ty: &Type) -> postgres::types::Format {
        match &self.value {
            ValueResult::DataBoolean(b) => b.encode_format(ty),
            ValueResult::DataDecimal(b) => b.encode_format(ty),
            ValueResult::DataInteger(b) => b.encode_format(ty),
            ValueResult::DataString(b) => b.encode_format(ty),
            ValueResult::DataTimestamp(b) => b.encode_format(ty),
        }
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut postgres::types::private::BytesMut,
    ) -> std::result::Result<postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    {
        match &self.value {
            ValueResult::DataBoolean(b) => b.to_sql_checked(ty, out),
            ValueResult::DataDecimal(b) => b.to_sql_checked(ty, out),
            ValueResult::DataInteger(b) => b.to_sql_checked(ty, out),
            ValueResult::DataString(b) => b.to_sql_checked(ty, out),
            ValueResult::DataTimestamp(b) => b.to_sql_checked(ty, out),
        }
    }
}
