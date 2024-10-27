use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;

use crate::connectors::{connection::Connection, http::HttpConnection};

pub struct Configuration {
    base_dir: PathBuf,
    connections: Vec<Connection>,
}

impl Default for Configuration {
    fn default() -> Self {
        let base_dir = dirs::data_dir().unwrap_or("/tmp".into()).join("baybridge");
        let connection = Connection::Http(HttpConnection::new("http://localhost:3000"));
        Self::new(base_dir, vec![connection])
    }
}

impl Configuration {
    pub fn new(base_dir: PathBuf, connections: Vec<Connection>) -> Configuration {
        Configuration {
            base_dir,
            connections,
        }
    }

    pub async fn init(&self) -> Result<()> {
        debug!("Creating base directory: {:?}", self.base_dir);
        tokio::fs::create_dir_all(&self.base_dir).await?;
        Ok(())
    }

    pub fn signing_key_path(&self) -> PathBuf {
        self.base_dir.join("private_signing_key")
    }

    pub fn get_connections(&self) -> &Vec<Connection> {
        &self.connections
    }

    pub fn server_database_path(&self) -> PathBuf {
        self.base_dir.join("server.sqlite")
    }
}
