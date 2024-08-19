use anyhow::Result;
use baybridge::{
    client::Actions, configuration::Configuration, crypto::encode::encode_verifying_key,
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
    Serve,
    Set { key: String, value: String },
    Get { verifying_key: String, key: String },
    List,
    Whoami,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Args::parse();
    let config = Configuration::new();

    config.init().await?;

    match cli.command {
        Commands::Serve => start_http_server(&config).await?,
        Commands::Set { key, value } => Actions::new(config).set(key, value).await?,
        Commands::Get { verifying_key, key } => {
            println!("{}", Actions::new(config).get(&verifying_key, &key).await?)
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
