use std::pin::Pin;
use std::task::Poll;

use anyhow::{anyhow, Result};
use futures_util::{Stream, StreamExt};
use pin_project::pin_project;
use serde_yaml::Value;
use tokio_postgres::types::{ToSql, Type};
use tokio_postgres::{Client, Config, NoTls, RowStream, Statement};

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
pub(crate) async fn execute(
    config: &Config,
    state: &mut ExecutionState,
) -> Result<Option<QueryResult>> {
    let mut result = Option::None;
    let mut stream = execute_stream(config, state).await?;
    while let Some(res) = stream.next().await {
        result = add_result(result, Some(res?));
    }
    Ok(result)
}

/// Execute a query and return the result.
pub(crate) async fn execute_stream<'a>(
    config: &Config,
    state: &'a mut ExecutionState,
) -> Result<Pin<Box<impl Stream<Item = Result<QueryResult>> + 'a>>> {
    // Get the query SQL.
    let query = state
        .query
        .execution_query_string(&mut state.store, &state.execution)?;
    // Get parameters.
    let params = state
        .query
        .execution_variables(&mut state.store, &state.execution)?;

    let query = positional("$", 1, &query, &params);

    let (client, connection) = config.connect(NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let stmt = client.prepare(&query).await?;

    let st = QueryResultStream::new(state, client, stmt, &params).await?;
    Ok(Box::pin(st))
}

#[pin_project]
struct QueryResultStream<'a> {
    state: &'a mut ExecutionState,
    client: Client,
    stmt: Statement,
    #[pin]
    it: RowStream,
    at_end: bool,
}

impl<'a> QueryResultStream<'a> {
    async fn new(
        state: &'a mut ExecutionState,
        client: Client,
        stmt: Statement,
        params: &[VariableResult],
    ) -> Result<QueryResultStream<'a>> {
        let it = client.query_raw(&stmt, params).await?;
        Ok(QueryResultStream {
            state,
            client,
            stmt,
            it,
            at_end: false,
        })
    }
}

impl<'a> Stream for QueryResultStream<'a> {
    type Item = Result<QueryResult>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        if *this.at_end {
            return Poll::Ready(None);
        }
        match this.it.poll_next(cx) {
            Poll::Ready(orow) => {
                match orow {
                    Some(Ok(row)) => {
                        // Build row.
                        let columns = this.stmt.columns();
                        let mut result_one = Vec::with_capacity(columns.len());
                        for (ix, col) in columns.iter().enumerate() {
                            match col.type_() {
                                &Type::INT2 | &Type::INT4 | &Type::INT8 => {
                                    result_one.push(Variable {
                                        name: col.name(),
                                        value: ValueResult::DataInteger(row.get(ix)),
                                    })
                                }
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
                                _ => {
                                    return Poll::Ready(Some(Err(anyhow!(
                                        "unsupported type {}",
                                        col.type_()
                                    ))))
                                }
                            }
                        }
                        // Send row to plugin.
                        match this.state.row(result_one).transpose() {
                            Some(res) => Poll::Ready(Some(res)),
                            None => {
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            },
                        }
                    }
                    Some(Err(err)) => Poll::Ready(Some(Err(anyhow!(err)))),
                    None => {
                        *this.at_end = true;
                        let columns = this.stmt.columns();
                        let names: Vec<&str> = columns.iter().map(|c| c.name()).collect();
                        Poll::Ready(
                            this.state
                                .query
                                .execution_end(&mut this.state.store, &this.state.execution, &names)
                                .map_err(|err| anyhow!(err))
                                .transpose(),
                        )
                    }
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl ToSql for VariableResult {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> std::result::Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
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

    fn encode_format(&self, ty: &Type) -> tokio_postgres::types::Format {
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
        out: &mut tokio_postgres::types::private::BytesMut,
    ) -> std::result::Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>>
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
