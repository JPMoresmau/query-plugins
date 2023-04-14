use wasmer::Module;
use wasmer_compiler::*;
use wasmer_compiler_llvm::LLVM;

use std::collections::HashMap;

use anyhow::Result;

mod config;
pub use config::{load_connections, load_plugins};
mod sqlite;

pub enum DBConnection {
    SqliteConnection(rusqlite::Connection),
}

pub struct State {
    pub connections: HashMap<String, DBConnection>,
    pub engine: Engine,
    pub plugins: HashMap<String, Module>,
}

impl State {
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
}

pub fn build_engine() -> Engine {
    let compiler_config = LLVM::default();
    EngineBuilder::new(compiler_config).engine()
}
