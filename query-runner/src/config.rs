//! Configuration utilities.

use std::fs::{self, File};
use std::{collections::HashMap, io::BufReader};

use anyhow::{anyhow, Result};
use wasmer::{Module, Store};
use wasmer_compiler::Engine;

use crate::DBConnection;

/// Load connections from the given file.
pub fn load_connections(path: &str) -> Result<HashMap<String, DBConnection>> {
    let file = BufReader::new(File::open(path)?);
    let values: HashMap<String, serde_yaml::Value> = serde_yaml::from_reader(file)?;
    let mut connections = HashMap::new();
    for (name, value) in values.into_iter() {
        let db = value
            .get("db")
            .ok_or(anyhow!("No db field"))?
            .as_str()
            .ok_or(anyhow!("db field is not a string"))?;
        let conn = match db {
            "sqlite" => crate::sqlite::new_connection(value)?,
            "postgres" => crate::postgres::new_connection(value)?,
            _ => return Err(anyhow!("unknown database type {db}")),
        };
        connections.insert(name, conn);
    }
    Ok(connections)
}

/// Load plugins from the given folder.
pub fn load_plugins(engine: &Engine, path: &str) -> Result<HashMap<String, Module>> {
    let paths = fs::read_dir(path).unwrap();
    let store = Store::new(engine);
    let mut plugins = HashMap::new();
    for path in paths {
        let path = path?;

        let module = Module::from_file(&store, path.path())?;
        let name = path
            .path()
            .file_stem()
            .ok_or(anyhow!("no file name!"))?
            .to_str()
            .ok_or(anyhow!("Cannot get file name"))?
            .to_owned();
        plugins.insert(name, module);
    }
    Ok(plugins)
}
