use std::collections::HashMap;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

use query_runner::*;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Connection management
    Connections {
        #[command(subcommand)]
        subcommand: ConnectionSubCommand,
    },
    /// Plugin management
    Plugins {
        #[command(subcommand)]
        subcommand: PluginSubCommand,
    },
    /// Run a plugin
    Run {
        /// Name of the plugin to run
        #[arg(short, long)]
        plugin: String,
        /// Name of the connection to use
        #[arg(short, long)]
        connection: String,
        /// Parameters in name=value format
        params: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
enum ConnectionSubCommand {
    /// List connections
    List,
}

#[derive(Debug, Subcommand)]
enum PluginSubCommand {
    /// List plugins
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Connections { subcommand } => match subcommand {
            ConnectionSubCommand::List => {
                let connections = load_connections("config/connections.yaml")?;
                for (name, connection) in connections.into_iter() {
                    println!("- {name}: {}", connection.db_type());
                }
            }
        },
        Command::Plugins { subcommand } => match subcommand {
            PluginSubCommand::List => {
                let engine = build_engine();
                let plugins = load_plugins(&engine, "plugins")?;
                for name in plugins.keys() {
                    println!("- {name}");
                }
            }
        },
        Command::Run {
            plugin,
            connection,
            params,
        } => {
            let st = State::load_from_disk()?;
            let mut variables = HashMap::new();
            for p in params.iter() {
                if let Some((name, value)) = p.split_once('=') {
                    variables.insert(name, value);
                } else {
                    return Err(anyhow!("{p} is not a valid name=value parameter"));
                }
            }
            let res = st.run_untyped(&plugin, &connection, &variables).await?;
            match res {
                None => println!("<no result>"),
                Some(res) => {
                    println!("{}", table_result(&res));
                }
            }
        }
    }
    Ok(())
}
