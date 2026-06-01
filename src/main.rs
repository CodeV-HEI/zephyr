use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result, anyhow};
use argon2::Argon2;
use base64::Engine;  // <-- AJOUT
use clap::{Parser, Subcommand};
use rand::RngCore;
use rand::rngs::OsRng;  // <-- AJOUT
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

// --- Entrée du coffre ---
#[derive(Debug, Serialize, Deserialize, Clone)]
struct VaultEntry {
    service: String,
    username: String,
    password: String,
}

// --- Structure du coffre (non chiffrée en mémoire) ---
#[derive(Debug, Default, Serialize, Deserialize)]
struct Vault {
    entries: Vec<VaultEntry>,
}

// --- Données chiffrées sur disque ---
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedVault {
    salt: String,
    nonce: String,
    ciphertext: String,
}

// --- CLI ---
#[derive(Parser)]
#[command(name = "Zephyr Vault")]
#[command(about = "Coffre-fort local avec chiffrement AES", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        service: String,
        username: String,
        password: Option<String>,
    },
    Search {
        query: String,
    },
    List,
    Remove {
        index: usize,
    },
}

const VAULT_FILE: &str = "vault.enc";
const AES_NONCE_SIZE: usize = 12;

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("Échec de la dérivation de clé Argon2: {}", e))?;
    Ok(key)
}

fn encrypt_vault(vault: &Vault, master_password: &str) -> Result<EncryptedVault> {
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
        .map_err(|e| anyhow!("Échec chiffrement : {:?}", e))?;

    let base64_engine = base64::engine::general_purpose::STANDARD;
    Ok(EncryptedVault {
        salt: base64_engine.encode(salt),
        nonce: base64_engine.encode(nonce_bytes),
        ciphertext: base64_engine.encode(ciphertext),
    })
}

fn decrypt_vault(encrypted: &EncryptedVault, master_password: &str) -> Result<Vault> {
    let base64_engine = base64::engine::general_purpose::STANDARD;

    let salt = base64_engine.decode(&encrypted.salt).context("Sel invalide")?;
    let nonce_bytes = base64_engine.decode(&encrypted.nonce).context("Nonce invalide")?;
    let ciphertext = base64_engine.decode(&encrypted.ciphertext).context("Ciphertext invalide")?;

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

fn load_vault(master_password: &str) -> Result<Vault> {
    let path = PathBuf::from(VAULT_FILE);
    if !path.exists() {
        return Ok(Vault::default());
    }
    let data = fs::read_to_string(&path).context("Lecture fichier vault.enc")?;
    let encrypted: EncryptedVault = serde_json::from_str(&data).context("Format vault.enc invalide")?;
    decrypt_vault(&encrypted, master_password)
}

fn save_vault(vault: &Vault, master_password: &str) -> Result<()> {
    let encrypted = encrypt_vault(vault, master_password)?;
    let json = serde_json::to_string_pretty(&encrypted)?;
    fs::write(VAULT_FILE, json).context("Écriture du fichier vault.enc")?;
    Ok(())
}

fn print_entry(index: usize, entry: &VaultEntry, show_password: bool) {
    let pwd = if show_password {
        &entry.password
    } else {
        "********"
    };
    println!(
        "{:3} | {:<20} | {:<20} | {}",
        index, entry.service, entry.username, pwd
    );
}

fn list_entries(vault: &Vault) {
    println!("Index | Service              | Username             | Password");
    println!("------+----------------------+----------------------+------------------");
    for (i, entry) in vault.entries.iter().enumerate() {
        print_entry(i, entry, false);
    }
}

fn search_entries<'a>(vault: &'a Vault, query: &str) -> Vec<(usize, &'a VaultEntry)> {
    let q = query.to_lowercase();
    vault
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.service.to_lowercase().contains(&q) || e.username.to_lowercase().contains(&q)
        })
        .collect()
}

fn get_master_password(force_create: bool) -> Result<String> {
    let vault_exists = PathBuf::from(VAULT_FILE).exists();
    if !vault_exists || force_create {
        println!("🔐 Création d'un nouveau coffre");
        let pwd = rpassword::prompt_password("Nouveau mot de passe maître: ")?;
        let confirm = rpassword::prompt_password("Confirmation: ")?;
        if pwd != confirm {
            anyhow::bail!("Les mots de passe ne correspondent pas");
        }
        if pwd.is_empty() {
            anyhow::bail!("Le mot de passe ne peut pas être vide");
        }
        return Ok(pwd);
    }
    let pwd = rpassword::prompt_password("Mot de passe maître: ")?;
    Ok(pwd)
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let force_create = match &cli.command {
        Commands::Add { .. } | Commands::Search { .. } | Commands::List | Commands::Remove { .. } => false,
    };

    let master_pwd = get_master_password(force_create)?;
    let mut vault = load_vault(&master_pwd)?;

    match cli.command {
        Commands::Add { service, username, password } => {
            let pwd = match password {
                Some(p) => p,
                None => {
                    print!("Mot de passe (non affiché) : ");
                    io::stdout().flush()?;
                    rpassword::read_password()?
                }
            };
            if pwd.is_empty() {
                anyhow::bail!("Le mot de passe ne peut pas être vide");
            }
            let entry = VaultEntry {
                service,
                username,
                password: pwd,
            };
            vault.entries.push(entry);
            save_vault(&vault, &master_pwd)?;
            println!("✅ Compte ajouté avec succès !");
        }
        Commands::Search { query } => {
            let results = search_entries(&vault, &query);
            if results.is_empty() {
                println!("Aucun résultat pour '{}'", query);
            } else {
                println!("Index | Service              | Username             | Password");
                println!("------+----------------------+----------------------+------------------");
                for (idx, entry) in results {
                    print_entry(idx, entry, false);
                }
                println!("\nPour voir le mot de passe d'une entrée, utilisez `list` et l'index.");
            }
        }
        Commands::List => {
            if vault.entries.is_empty() {
                println!("📭 Le coffre est vide.");
            } else {
                list_entries(&vault);
                println!("\n💡 Pour supprimer : `zephyr_vault remove <index>`");
            }
        }
        Commands::Remove { index } => {
            if index >= vault.entries.len() {
                anyhow::bail!("Index invalide. Utilisez `list` pour voir les indices.");
            }
            let removed = vault.entries.remove(index);
            save_vault(&vault, &master_pwd)?;
            println!(
                "🗑️  Supprimé : {} ({})",
                removed.service, removed.username
            );
        }
    }

    Ok(())
}