//! Main library code.

use serde::Serialize;
use serde_yaml::with::singleton_map_recursive::serialize;
use wasmer::{imports, Module, Store};
use wasmer_compiler::*;
use wasmer_compiler_llvm::LLVM;

use anyhow::{anyhow, Result};
use serde_json::{json, Map, Value};
use std::{collections::HashMap, fmt::Display, sync::Mutex};
use tabled::builder::Builder;

mod config;
pub use config::{load_connections, load_plugins};
mod parse;
pub use parse::parse_parameter_values;
mod postgres;
mod sqlite;

wai_bindgen_wasmer::import!("query.wai");

pub use crate::query::*;

/// Only supported databases for now.
const SQLITE: &str = "sqlite";
const POSTGRES: &str = "postgres";

/// Connections to databases.
pub enum DBConnection {
    SqliteConnection(Mutex<rusqlite::Connection>),
    PostgresConnection(Box<::tokio_postgres::config::Config>),
}

impl DBConnection {
    /// Type of connection.
    pub fn db_type(&self) -> &'static str {
        match self {
            DBConnection::SqliteConnection(..) => SQLITE,
            DBConnection::PostgresConnection(..) => POSTGRES,
        }
    }

    /// Execute the query against the DB and returns the result.
    pub(crate) async fn execute(&self, state: &mut ExecutionState) -> Result<Option<QueryResult>> {
        match self {
            DBConnection::SqliteConnection(connection) => sqlite::execute(connection, state),
            DBConnection::PostgresConnection(config) => {
                crate::postgres::execute(config, state).await
            }
        }
    }
}

/// An optional list of rows as result.
//pub type QueryResult = Option<Vec<Vec<VariableResult>>>;

/// Add two results together.
pub(crate) fn add_result(
    qr1: Option<QueryResult>,
    qr2: Option<QueryResult>,
) -> Option<QueryResult> {
    match (qr1, qr2) {
        (None, qr2) => qr2,
        (qr1, None) => qr1,
        (Some(mut vec1), Some(mut vec2)) => {
            vec1.values.append(&mut vec2.values);
            Some(vec1)
        }
    }
}

/// Keep general engine state.
pub struct State {
    /// Connections by name.
    pub connections: HashMap<String, DBConnection>,
    /// WASM Engine.
    pub engine: Engine,
    /// Plugins by name.
    pub plugins: HashMap<String, Module>,
}

impl State {
    /// Load state from local files.
    pub fn load_from_disk() -> Result<State> {
        let connections = load_connections("config/connections.yaml")?;
        let engine = build_engine();
        let plugins = load_plugins(&engine, "plugins")?;
        Ok(State {
            connections,
            engine,
            plugins,
        })
    }

    /// Run a plugin with untyped parameters.
    pub async fn run_untyped(
        &self,
        plugin: &str,
        connection: &str,
        variables: &HashMap<&str, &str>,
    ) -> Result<Option<QueryResult>> {
        let module = self.get_plugin(plugin)?;
        let params = self.get_metadata(module)?.parameters;
        let values = parse_parameter_values(&params, variables)?;
        self.run_connection(connection, module, &values).await
    }

    /// Run a plugin with typed parameters.
    pub async fn run_typed<'a>(
        &'a self,
        plugin: &str,
        connection: &str,
        variables: &[VariableParam<'a>],
    ) -> Result<Option<QueryResult>> {
        let module = self.get_plugin(plugin)?;
        self.run_connection(connection, module, variables).await
    }

    /// Run a module with the given variables.
    async fn run_connection<'a>(
        &'a self,
        connection: &str,
        module: &Module,
        variables: &[VariableParam<'a>],
    ) -> Result<Option<QueryResult>> {
        let connection = self.get_connection(connection)?;
        self.run(connection, module, variables).await
    }

    /// Run a module knowing the connection and variables.
    pub async fn run<'a>(
        &'a self,
        connection: &DBConnection,
        module: &Module,
        variables: &[VariableParam<'a>],
    ) -> Result<Option<QueryResult>> {
        let mut store = Store::new(&self.engine);
        let mut imports = imports! {};
        let (query, _instance) = Query::instantiate(&mut store, module, &mut imports)?;
        let execution = query.start(&mut store, variables)?;

        let mut es = ExecutionState {
            store,
            query,
            execution,
        };
        connection.execute(&mut es).await
    }

    /// Get plugin module by name.
    pub fn get_plugin(&self, plugin: &str) -> Result<&Module> {
        self.plugins
            .get(plugin)
            .ok_or(anyhow!("no plugin named {plugin} registered"))
    }

    /// Get metadata for a module.
    pub fn get_metadata(&self, module: &Module) -> Result<QueryMetadata> {
        let mut store = Store::new(&self.engine);
        let mut imports = imports! {};
        let (query, _instance) = Query::instantiate(&mut store, module, &mut imports)?;

        let metadata = query.metadata(&mut store)?;
        Ok(metadata)
    }

    /// Get a connection by name.
    pub fn get_connection(&self, connection: &str) -> Result<&DBConnection> {
        self.connections
            .get(connection)
            .ok_or(anyhow!("no connection named {connection} registered"))
    }
}

/// Build a new WASM engine.
pub fn build_engine() -> Engine {
    let compiler_config = LLVM::default();
    EngineBuilder::new(compiler_config).engine()
}

/// Stores everything related to one plugin execution.
pub(crate) struct ExecutionState {
    /// The store.
    pub(crate) store: Store,
    /// The query instance.
    pub(crate) query: Query,
    /// The actual execution.
    pub(crate) execution: Execution,
}

impl ExecutionState {
    /// Send a row to the execution.
    fn row(&mut self, row: Vec<Variable>) -> Result<Option<QueryResult>> {
        let params: Vec<VariableParam<'_>> = row.iter().map(Variable::as_param).collect();
        let r = self
            .query
            .execution_row(&mut self.store, &self.execution, &params)?;
        Ok(r)
    }
}

/// Halfway between `VariableResult` and `VariableParam`:
/// the name is a reference but the value is owned.
pub(crate) struct Variable<'a> {
    pub(crate) name: &'a str,
    pub(crate) value: ValueResult,
}

impl<'a> Variable<'a> {
    /// Converts a Variable to a `VariableParam`, referencing the `Variable` data.
    fn as_param(&'a self) -> VariableParam<'a> {
        VariableParam {
            name: self.name,
            value: match &self.value {
                ValueResult::DataBoolean(b) => ValueParam::DataBoolean(*b),
                ValueResult::DataDecimal(d) => ValueParam::DataDecimal(*d),
                ValueResult::DataInteger(i) => ValueParam::DataInteger(*i),
                ValueResult::DataString(s) => ValueParam::DataString(s.as_deref()),
                ValueResult::DataTimestamp(t) => ValueParam::DataTimestamp(t.as_deref()),
            },
        }
    }
}

impl Display for ValueResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueResult::DataBoolean(None) => write!(f, "<null>"),
            ValueResult::DataBoolean(Some(b)) => write!(f, "{b}"),
            ValueResult::DataDecimal(None) => write!(f, "<null>"),
            ValueResult::DataDecimal(Some(d)) => write!(f, "{d}"),
            ValueResult::DataInteger(None) => write!(f, "<null>"),
            ValueResult::DataInteger(Some(i)) => write!(f, "{i}"),
            ValueResult::DataString(None) => write!(f, "<null>"),
            ValueResult::DataString(Some(s)) => write!(f, "{s}"),
            ValueResult::DataTimestamp(None) => write!(f, "<null>"),
            ValueResult::DataTimestamp(Some(s)) => write!(f, "{s}"),
        }
    }
}

/// Format result as a table.
pub fn table_result(qr: &QueryResult) -> String {
    let mut builder = Builder::default();
    builder.set_header(&qr.names);
    for row in qr.values.iter() {
        builder.push_record(row.iter().map(|r| format!("{r}")));
    }

    builder.build().to_string()
}

impl Display for ParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterType::TypeBoolean => write!(f, "bool"),
            ParameterType::TypeDecimal => write!(f, "decimal"),
            ParameterType::TypeInteger => write!(f, "integer"),
            ParameterType::TypeString => write!(f, "string"),
            ParameterType::TypeTimestamp => write!(f, "timestamp"),
        }
    }
}

impl From<ValueResult> for Value {
    fn from(vr: ValueResult) -> Self {
        match vr {
            ValueResult::DataBoolean(Some(b)) => Value::Bool(b),
            ValueResult::DataDecimal(Some(d)) => json!(d),
            ValueResult::DataInteger(Some(i)) => json!(i),
            ValueResult::DataString(Some(s)) => Value::String(s),
            ValueResult::DataTimestamp(Some(t)) => Value::String(t),
            _ => Value::Null,
        }
    }
}

impl From<QueryResult> for Value {
    fn from(qr: QueryResult) -> Value {
        let mut map = Map::new();
        map.insert(
            "names".to_string(),
            Value::Array(qr.names.into_iter().map(|s| s.into()).collect()),
        );
        let mut rows = Vec::new();
        for row in qr.values.into_iter() {
            rows.push(Value::Array(row.into_iter().map(|v| v.into()).collect()));
        }
        map.insert("values".to_string(), Value::Array(rows));

        Value::Object(map)
    }
}

impl From<Parameter> for Value {
    fn from(p: Parameter) -> Self {
        json!({
            "name": p.name,
            "type": p.parameter_type.to_string(),
        })
    }
}

impl Serialize for Parameter {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serialize(
            &json!({
                "name": self.name,
                "type": self.parameter_type.to_string(),
            }),
            serializer,
        )
    }
}

impl PartialEq for Parameter {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.parameter_type == other.parameter_type
    }
}

impl Eq for Parameter {}
