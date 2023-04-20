use anyhow::Result;

use query_runner::*;

#[test]
fn load_connections_from_file() -> Result<()> {
    let connections = load_connections("config/connections.yaml")?;
    assert!(connections.contains_key("memory"));
    assert!(matches!(
        connections.get("memory"),
        Some(DBConnection::SqliteConnection(_))
    ));
    assert!(connections.contains_key("postgres1"));
    assert!(matches!(
        connections.get("postgres1"),
        Some(DBConnection::PostgresConnection(_))
    ));
    Ok(())
}

#[test]
fn load_plugins_from_file() -> Result<()> {
    let engine = build_engine();
    let plugins = load_plugins(&engine, "plugins")?;
    assert!(plugins.contains_key("test_collect"));
    assert!(plugins.contains_key("test_collect2"));
    Ok(())
}
