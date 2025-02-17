use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use baybridge::{
    client::{Actions, Expiry},
    configuration::Configuration,
    connectors::{connection::Connection, http::HttpConnection},
    crypto::encode::encode_verifying_key,
    models::{Name, Value},
    server::http::start_http_server,
};
use clap::{command, Parser, Subcommand};

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
        #[clap(short, long, group = "expiry")]
        ttl: Option<u64>,
        // Explicit unix timestamp expires at
        #[clap(short, long, group = "expiry")]
        expires_at: Option<u64>,
        #[clap(short, long)]
        priority: Option<u64>,
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
        .iter()
        .map(|s| {
            Connection::Http(HttpConnection::new(
                url::Url::parse(s).expect("Failed to parse server url: {url}"),
            ))
        })
        .collect();

    let config = Configuration::new(config_dir, servers);
    config.init().await?;

    match cli.command {
        Commands::Serve { peer } => {
            let peer_http_url = peer
                .iter()
                .map(|peer| url::Url::parse(peer).expect("Failed to parse peer url: {url}"))
                .collect();
            start_http_server(&config, peer_http_url).await?
        }
        Commands::Set {
            name,
            value,
            ttl,
            expires_at,
            priority,
        } => {
            let name = Name::new(name);
            let value = Value::new(value.as_bytes().to_vec());
            let expiry = ttl.map(|ttl| Expiry::Ttl(std::time::Duration::from_secs(ttl)));
            let expiry = match expires_at {
                Some(expires_at) => Some(Expiry::ExpiresAt(expires_at)),
                None => expiry,
            };

            Actions::new(config)
                .set()
                .name(name)
                .value(value)
                .maybe_expiry(expiry)
                .maybe_priority(priority)
                .call()
                .await?
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
            for (verifying_key, value) in namespace.mapping {
                println!(
                    "{}: {}",
                    encode_verifying_key(&verifying_key),
                    String::from_utf8_lossy(value.as_bytes())
                );
            }
        }
        Commands::Whoami => {
            let verifying_key = Actions::new(config).whoami().await;
            let encoded_verifying_key = encode_verifying_key(&verifying_key);
            println!("{}", encoded_verifying_key);
        }
    }
    Ok(())
}
