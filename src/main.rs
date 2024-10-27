use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use baybridge::{
    client::Actions,
    configuration::Configuration,
    connectors::{connection::Connection, http::HttpConnection},
    crypto::encode::encode_verifying_key,
    models::{Name, Value},
};
use clap::{command, Parser, Subcommand};
use http_server::start_http_server;

mod http_server;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[clap(short, long)]
    config_dir: Option<String>,
    #[clap(short, long)]
    server: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Serve {
        #[clap(short, long)]
        peer: Vec<String>,
    },
    Set {
        name: String,
        value: String,
        // Time to live in seconds
        #[clap(short, long)]
        ttl: Option<u64>,
        // Explicit unix timestamp expires at
        expires_at: Option<u64>,
    },
    Delete {
        name: String,
    },
    Get {
        verifying_key: String,
        name: String,
    },
    Namespace {
        name: String,
    },
    List,
    Whoami,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Args::parse();

    let config_dir = cli
        .config_dir
        .map(|path| PathBuf::from_str(&path).unwrap())
        .unwrap_or_else(|| dirs::data_dir().unwrap_or("/tmp".into()).join("baybridge"));
    let servers = if cli.server.is_empty() {
        vec![String::from("http://localhost:3000")]
    } else {
        cli.server
    };
    let servers = servers
        .into_iter()
        .map(|s| Connection::Http(HttpConnection::new(&s)))
        .collect();

    let config = Configuration::new(config_dir, servers);
    config.init().await?;

    match cli.command {
        Commands::Serve { peer } => start_http_server(&config, peer).await?,
        Commands::Set {
            name,
            value,
            ttl,
            expires_at,
        } => {
            let name = Name::new(name);
            let value = Value::new(value.as_bytes().to_vec());
            let expires_at = match ttl {
                Some(ttl) => Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                        + ttl,
                ),
                None => expires_at,
            };

            match expires_at {
                None => Actions::new(config).set(name, value).await?,
                Some(expires_at) => {
                    Actions::new(config)
                        .set_with_expires_at(name, value, expires_at)
                        .await?
                }
            }
        }
        Commands::Delete { name } => {
            let name = Name::new(name);
            Actions::new(config).delete(name).await?
        }
        Commands::Get {
            verifying_key,
            name,
        } => {
            let name = Name::new(name);
            let value = Actions::new(config).get(&verifying_key, &name).await?;
            let value = String::from_utf8_lossy(value.as_bytes());
            println!("{}", value);
        }
        Commands::Namespace { name } => {
            let namespace = Actions::new(config).namespace(&name).await?;
            println!("{:?}", namespace.mapping.keys());
        }
        Commands::List => {
            let verifying_keys = Actions::new(config).list().await?;
            for verifying_key in verifying_keys {
                println!("{}", encode_verifying_key(&verifying_key));
            }
        }
        Commands::Whoami => println!("{}", Actions::new(config).whoami().await),
    }
    Ok(())
}
