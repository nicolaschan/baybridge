use std::{path::PathBuf, sync::Arc};

use ed25519_dalek::VerifyingKey;
use rusqlite::params;
use tokio::sync::Mutex;

use crate::{
    client::Event,
    crypto::{encode::encode_verifying_key, Signed},
    models::{self, Name},
};

#[derive(Clone)]
pub struct SqliteController {
    connection: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteController {
    pub fn new(database_path: &PathBuf) -> anyhow::Result<Self> {
        let connection = rusqlite::Connection::open(database_path)?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    verifying_key BLOB NOT NULL,
                    name BLOB NOT NULL,
                    signed_event BLOB NOT NULL UNIQUE,
                    priority BIGINT NOT NULL,
                    expires_at INTEGER
                )",
            (),
        )?;
        let connection = Arc::new(Mutex::new(connection));
        Ok(Self { connection })
    }

    pub async fn delete_expired_events(&self, unix_timestamp: u64) -> anyhow::Result<usize> {
        let database_guard = self.connection.lock().await;
        let num_deleted = database_guard.execute(
            "DELETE FROM events WHERE expires_at <= ?",
            (unix_timestamp,),
        )?;
        Ok(num_deleted)
    }

    pub async fn event_count(&self) -> anyhow::Result<usize> {
        let database_guard = self.connection.lock().await;
        let mut stmt = database_guard.prepare("SELECT COUNT(*) FROM events")?;
        let count = stmt.query_row([], |row| row.get(0))?;
        Ok(count)
    }

    pub async fn signed_events(&self) -> anyhow::Result<Vec<Signed<Event>>> {
        let database_guard = self.connection.lock().await;
        let mut stmt = database_guard.prepare("SELECT signed_event FROM events")?;
        let signed_events = stmt
            .query_map([], |row| {
                let signed_event_serialized: Vec<u8> = row.get(0)?;
                let signed_event: Signed<Event> =
                    bincode::deserialize(&signed_event_serialized).unwrap();
                Ok(signed_event)
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(signed_events)
    }

    pub async fn current_state_hash(&self) -> anyhow::Result<models::Hash> {
        let all_signed_events = self.signed_events().await?;
        let serialized_events = bincode::serialize(&all_signed_events)?;
        let blake3_hash = blake3::hash(&serialized_events);
        Ok(models::Hash(blake3_hash))
    }

    pub async fn events_by_key_and_name(
        &self,
        verifying_key: String,
        name: String,
    ) -> anyhow::Result<Vec<Signed<Event>>> {
        let database_guard = self.connection.lock().await;
        let mut stmt = database_guard
            .prepare("SELECT signed_event FROM events WHERE verifying_key = ? AND name = ?")?;
        let events = stmt
            .query_map([verifying_key.as_bytes(), name.as_bytes()], |row| {
                let signed_event_serialized: Vec<u8> = row.get(0)?;
                let signed_event: Signed<Event> =
                    bincode::deserialize(&signed_event_serialized).unwrap();
                Ok(signed_event)
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(events)
    }

    pub async fn events_by_namespace(&self, name: &str) -> anyhow::Result<Vec<Signed<Event>>> {
        let database_guard = self.connection.lock().await;
        let mut stmt = database_guard.prepare("SELECT signed_event FROM events WHERE name = ?")?;
        let events = stmt
            .query_map([name.as_bytes()], |row| {
                let signed_event_serialized: Vec<u8> = row.get(0)?;
                let signed_event: Signed<Event> =
                    bincode::deserialize(&signed_event_serialized).unwrap();
                Ok(signed_event)
            })?
            .filter_map(Result::ok)
            .collect();
        Ok(events)
    }

    pub async fn insert_event(
        &self,
        verifying_key: VerifyingKey,
        name: Name,
        signed_event: Signed<Event>,
        priority: u64,
        expires_at: Option<u64>,
    ) -> anyhow::Result<usize> {
        let normalized_verifying_key = encode_verifying_key(&verifying_key);
        let signed_event_serialized = bincode::serialize(&signed_event)?;
        let database_guard = self.connection.lock().await;
        let num_inserted = database_guard.execute(
            "INSERT INTO events (verifying_key, name, signed_event, priority, expires_at) VALUES (?, ?, ?, ?, ?)",
            params![
                normalized_verifying_key.as_bytes(),
                name.as_str().as_bytes(),
                signed_event_serialized.as_slice(),
                priority,
                expires_at,
            ],
        )?;
        Ok(num_inserted)
    }
}
