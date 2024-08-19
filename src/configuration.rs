use anyhow::Result;
use std::path::PathBuf;

use crate::connectors::{connection::Connection, http::HttpConnection};

pub struct Configuration {
    base_dir: PathBuf,
    connection: Connection,
}

impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

impl Configuration {
    pub fn new() -> Configuration {
        Configuration {
            base_dir: dirs::config_dir().unwrap().join("baybridge"),
            connection: Connection::Http(HttpConnection::new("http://localhost:3000")),
        }
    }

    pub async fn init(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.base_dir).await?;
        Ok(())
    }

    pub fn signing_key_path(&self) -> PathBuf {
        self.base_dir.join("private_signing_key")
    }

    pub fn connection(&self) -> &Connection {
        &self.connection
    }

    pub fn server_database_path(&self) -> PathBuf {
        self.base_dir.join("server.sqlite")
    }
}
