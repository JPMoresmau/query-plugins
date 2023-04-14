mod config;
mod sqlite;

pub use config::load_connections;

pub enum DBConnection {
    SqliteConnection(rusqlite::Connection),
}
