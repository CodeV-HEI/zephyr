use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::crypto::{EncryptedVault, decrypt_vault, encrypt_vault};
use crate::vault::Vault;

pub const VAULT_FILE: &str = "vault.enc";

pub struct VaultRepository;

impl VaultRepository {
    pub fn load(master_password: &str) -> Result<Vault> {
        let path = PathBuf::from(VAULT_FILE);
        if !path.exists() {
            return Ok(Vault::default());
        }
        let data = fs::read_to_string(&path).context("Lecture fichier vault.enc")?;
        let encrypted: EncryptedVault = serde_json::from_str(&data).context("Format invalide")?;
        decrypt_vault(&encrypted, master_password)
    }

    pub fn save(vault: &Vault, master_password: &str) -> Result<()> {
        let encrypted = encrypt_vault(vault, master_password)?;
        let json = serde_json::to_string_pretty(&encrypted)?;
        fs::write(VAULT_FILE, json).context("Écriture vault.enc")?;
        Ok(())
    }
}
