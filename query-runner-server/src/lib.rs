//! Web service API implementation.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use http::Method;
use query_runner::{parse_parameter_values, Parameter};
use serde::Serialize;
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

/// State.
struct AppState {
    runner: query_runner::State,
}

/// App routes.
pub fn app() -> Result<Router> {
    let runner_state = Arc::new(AppState {
        runner: query_runner::State::load_from_disk()?,
    });

    let cors = CorsLayer::new().allow_origin(Any).allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/connections", get(connections))
        .route("/plugins", get(plugins))
        .route("/plugins/:name", get(plugin_metadata))
        .route("/plugins/:name/:connection", post(plugin_execute))
        .with_state(runner_state)
        .layer(cors);

    Ok(app)
}

/// List connections.
async fn connections(State(state): State<Arc<AppState>>) -> Json<Vec<Connection>> {
    let mut conns = Vec::new();
    for (name, conn) in state.runner.connections.iter() {
        conns.push(Connection {
            name: name.to_owned(),
            db_type: conn.db_type(),
        });
    }
    conns.sort();
    Json(conns)
}

/// List plugins.
async fn plugins(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Plugin>>, AppError> {
    let mut conns = Vec::new();
    for (name, module) in state.runner.plugins.iter() {
        match state.runner.get_metadata(module) {
            Ok(metadata) => conns.push(Plugin {
                name: name.to_owned(),
                description: metadata.description,
            }),
            Err(err) => {
                tracing::error!("{err}");
                return Err(AppError::PluginMetadata);
            }
        }
    }
    conns.sort();
    Ok(Json(conns))
}

/// Retrieve plugin metadata.
async fn plugin_metadata(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<PluginMetadata>, AppError> {
    match state.runner.plugins.get(&name) {
        Some(module) => match state.runner.get_metadata(module) {
            Ok(metadata) => Ok(Json(PluginMetadata {
                name,
                description: metadata.description,
                parameters: metadata.parameters,
            })),
            Err(err) => {
                tracing::error!("{err}");
                Err(AppError::PluginMetadata)
            }
        },
        None => Err(AppError::PluginMissing(name)),
    }
}

/// Execute plugin.
async fn plugin_execute(
    State(state): State<Arc<AppState>>,
    Path((plugin, connection)): Path<(String, String)>,
    Json(variables): Json<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    match state.runner.plugins.get(&plugin) {
        Some(module) => match state.runner.connections.get(&connection) {
            Some(conn) => match state.runner.get_metadata(module) {
                Ok(metadata) => {
                    let params = metadata.parameters;
                    match parse_parameter_values(&params, &variables) {
                        Ok(values) => match state.runner.run(conn, module, &values).await {
                            Ok(Some(qr)) => Ok(Json(qr.into())),
                            Ok(None) => Ok(Json(json!("no results returned"))),
                            Err(err) => {
                                tracing::error!("{err}");
                                Err(AppError::PluginExecution(
                                    plugin,
                                    connection,
                                    err.to_string(),
                                ))
                            }
                        },
                        Err(err) => Err(AppError::ExecutionParameters(
                            plugin,
                            connection,
                            err.to_string(),
                        )),
                    }
                }
                Err(err) => {
                    tracing::error!("{err}");
                    Err(AppError::PluginMetadata)
                }
            },
            None => Err(AppError::ConnectionMissing(connection)),
        },
        None => Err(AppError::PluginMissing(plugin)),
    }
}

/// Any error we may encounter.
enum AppError {
    PluginMetadata,
    PluginMissing(String),
    PluginExecution(String, String, String),
    ConnectionMissing(String),
    ExecutionParameters(String, String, String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::ConnectionMissing(name) => {
                (StatusCode::NOT_FOUND, format!("connection `{name}` not found"))
            },
            AppError::PluginMetadata => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("Error retrieving plugin metadata"),
            ),
            AppError::PluginMissing(name) => {
                (StatusCode::NOT_FOUND, format!("plugin `{name}` not found"))
            },
            AppError::PluginExecution(plugin, connection, error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("plugin `{plugin}` failed on connection `{connection}`: {error}"))
            },
            AppError::ExecutionParameters(plugin, connection, error) => {
                (StatusCode::BAD_REQUEST, format!("plugin `{plugin}` failed on connection `{connection}` while parsing parameters: {error}"))
            },
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Connection info.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Connection {
    name: String,
    db_type: &'static str,
}

/// Plugin info.
#[derive(Serialize, PartialEq, Eq, PartialOrd, Ord)]
struct Plugin {
    name: String,
    description: String,
}

/// Plugin metadata.
#[derive(Serialize, PartialEq, Eq)]
struct PluginMetadata {
    name: String,
    description: String,
    parameters: Vec<Parameter>,
}
