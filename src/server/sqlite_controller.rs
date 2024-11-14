use std::{path::PathBuf, sync::Arc};

use itertools::Itertools;
use rusqlite::params;
use tokio::sync::Mutex;
use tracing::debug;
use ed25519_dalek::VerifyingKey;

use crate::{
    api::StateHash,
    client::Event,
    crypto::{encode::encode_verifying_key, Signed},
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
        connection.execute(
            "CREATE TABLE IF NOT EXISTS peers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                last_hash BLOB
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

    pub async fn delete_old_events(&self, verifying_key: VerifyingKey, name: &str) -> anyhow::Result<usize> {
        let normalized_verifying_key = encode_verifying_key(&verifying_key);
        let database_guard = self.connection.lock().await;

        let num_deleted = database_guard.execute(
            "DELETE FROM events WHERE verifying_key = ? AND name = ? AND expires_at IS NOT NULL
             ORDER BY priority DESC, id DESC
             OFFSET 1",
            (normalized_verifying_key.as_bytes(),
             name),
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
        let mut stmt =
            database_guard.prepare("SELECT signed_event FROM events ORDER BY signed_event")?;
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

    pub async fn current_state_hash(&self) -> anyhow::Result<StateHash> {
        let all_signed_events = self.signed_events().await?;
        let serialized_events = bincode::serialize(&all_signed_events)?;
        let hash = blake3::hash(&serialized_events);
        Ok(StateHash { hash })
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

    pub async fn insert_event(&self, signed_event: Signed<Event>) -> anyhow::Result<usize> {
        let name = signed_event.inner.name();
        let priority = signed_event.inner.priority();
        let verifying_key = signed_event.verifying_key;
        let expires_at = signed_event.inner.expires_at();

        let normalized_verifying_key = encode_verifying_key(&verifying_key);
        let signed_event_serialized = bincode::serialize(&signed_event)?;
        let database_guard = self.connection.lock().await;
        let insert_result = database_guard.execute(
            "INSERT INTO events (verifying_key, name, signed_event, priority, expires_at) VALUES (?, ?, ?, ?, ?)",
            params![
                normalized_verifying_key.as_bytes(),
                name.as_str().as_bytes(),
                signed_event_serialized.as_slice(),
                priority,
                expires_at,
            ],
        );
        match insert_result {
            Ok(num_inserted) => Ok(num_inserted),
            Err(e) => {
                debug!("Ignoring error inserting event: {:?}", e);
                Ok(0)
            }
        }
    }

    pub async fn insert_events(&self, events: Vec<Signed<Event>>) -> anyhow::Result<()> {
        for event in events {
            self.insert_event(event).await?;
        }
        Ok(())
    }

    pub async fn set_peer_last_hash(&self, peer_url: &str, hash: StateHash) -> anyhow::Result<()> {
        let database_guard = self.connection.lock().await;
        database_guard.execute(
            "INSERT INTO peers (url, last_hash) VALUES (?, ?)
             ON CONFLICT(url) DO UPDATE SET last_hash = excluded.last_hash",
            params![peer_url, hash.hash.as_bytes()],
        )?;
        Ok(())
    }

    pub async fn get_peer_last_hash(&self, peer_url: &str) -> Option<StateHash> {
        let database_guard = self.connection.lock().await;
        let mut stmt = database_guard
            .prepare("SELECT last_hash FROM peers WHERE url = ?")
            .unwrap();
        let hash = stmt
            .query_map([peer_url], |row| {
                let hash_bytes: [u8; 32] = row.get(0)?;
                Ok(blake3::Hash::from_bytes(hash_bytes))
            })
            .ok()?
            .at_most_one()
            .ok()??
            .ok()?;
        Some(StateHash { hash })
    }
}
