use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result, anyhow};
use argon2::Argon2;
use base64::Engine;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use crate::vault::Vault;

const AES_NONCE_SIZE: usize = 12;

#[derive(Debug, Serialize, Deserialize)]
pub struct EncryptedVault {
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("Dérivation clé échouée: {}", e))?;
    Ok(key)
}

pub fn encrypt_vault(vault: &Vault, master_password: &str) -> Result<EncryptedVault> {
    let plaintext = serde_json::to_vec(vault).context("Sérialisation JSON")?;
    let salt: [u8; 16] = rand::random();
    let key = derive_key(master_password, &salt)?;

    let mut nonce_bytes = [0u8; AES_NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow!("Initialisation AES: {}", e))?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow!("Échec chiffrement: {:?}", e))?;

    let engine = base64::engine::general_purpose::STANDARD;
    Ok(EncryptedVault {
        salt: engine.encode(salt),
        nonce: engine.encode(nonce_bytes),
        ciphertext: engine.encode(ciphertext),
    })
}

pub fn decrypt_vault(encrypted: &EncryptedVault, master_password: &str) -> Result<Vault> {
    let engine = base64::engine::general_purpose::STANDARD;
    let salt = engine.decode(&encrypted.salt).context("Sel invalide")?;
    let nonce_bytes = engine.decode(&encrypted.nonce).context("Nonce invalide")?;
    let ciphertext = engine.decode(&encrypted.ciphertext).context("Ciphertext invalide")?;

    if nonce_bytes.len() != AES_NONCE_SIZE {
        anyhow::bail!("Nonce doit faire 12 octets");
    }

    let key = derive_key(master_password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| anyhow!("Initialisation AES: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow!("Mot de passe maître incorrect ou données corrompues"))?;

    let vault: Vault = serde_json::from_slice(&plaintext).context("JSON invalide")?;
    Ok(vault)
}