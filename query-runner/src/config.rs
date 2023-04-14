use std::fs::File;
use std::{collections::HashMap, io::BufReader};

use anyhow::{anyhow, Result};

use crate::DBConnection;

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
            _ => return Err(anyhow!("unknown database type {db}")),
        };
        connections.insert(name, conn);
    }
    Ok(connections)
}
