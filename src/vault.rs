use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultEntry {
    pub service: String,
    pub username: String,
    pub password: String,
    pub created_at: u64,
    pub updated_at: u64,
}

impl VaultEntry {
    pub fn new(service: String, username: String, password: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            service,
            username,
            password,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vault {
    pub entries: Vec<VaultEntry>,
}