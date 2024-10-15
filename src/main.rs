use anyhow::Result;
use baybridge::{
    client::Actions,
    configuration::Configuration,
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
    let config = Configuration::default();

    config.init().await?;

    match cli.command {
        Commands::Serve { peer } => start_http_server(&config, peer).await?,
        Commands::Set { name, value } => {
            let name = Name::new(name);
            let value = Value::new(value.as_bytes().to_vec());
            Actions::new(config).set(name, value).await?
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
