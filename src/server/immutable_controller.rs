use std::{path::PathBuf, sync::Arc};

use bincode::config::standard;
use tokio::sync::RwLock;

use crate::models::ContentBlock;

#[derive(Debug, Clone)]
pub struct ImmutableController {
    basedir: PathBuf,
    filesystem_lock: Arc<RwLock<()>>,
}

impl ImmutableController {
    pub async fn new(basedir: PathBuf) -> Self {
        tokio::fs::create_dir_all(&basedir).await.unwrap();
        Self {
            basedir,
            filesystem_lock: Arc::new(RwLock::new(())),
        }
    }

    pub async fn get(&self, hash: &blake3::Hash) -> Option<ContentBlock> {
        let _guard = self.filesystem_lock.read().await;
        let path = self.basedir.join(hash.to_string());
        let encoded = tokio::fs::read(&path).await.ok()?;
        bincode::decode_from_slice(&encoded, standard())
            .ok()
            .map(|v| v.0)
    }

    pub async fn set(&self, content: &ContentBlock) -> blake3::Hash {
        let _guard = self.filesystem_lock.write().await;
        let encoded = bincode::encode_to_vec(content, standard()).unwrap();
        let hash = blake3::hash(&encoded);
        let path = self.basedir.join(hash.to_string());
        if !path.exists() {
            tokio::fs::write(&path, &encoded).await.unwrap();
        }
        hash
    }
}
