use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use clap::{Parser, Subcommand};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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
    salt: String,           // Sel Argon2 (base64)
    nonce: String,          // Nonce AES (base64, 12 octets)
    ciphertext: String,     // Données chiffrées (base64)
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
    /// Ajouter un nouveau mot de passe
    Add {
        /// Nom du service (ex: GitHub)
        service: String,
        /// Identifiant / email
        username: String,
        /// Mot de passe (optionnel, sinon demandé interactivement)
        password: Option<String>,
    },
    /// Rechercher des comptes (service ou identifiant)
    Search {
        /// Chaîne à rechercher
        query: String,
    },
    /// Lister tous les comptes (mots de passe masqués)
    List,
    /// Supprimer une entrée (par index affiché dans list ou search)
    Remove {
        /// Index de l'entrée (visible dans list/search)
        index: usize,
    },
}

// --- Constantes ---
const VAULT_FILE: &str = "vault.enc";
const AES_NONCE_SIZE: usize = 12; // GCM recommande 12 octets

// -----------------------------------------------------------------------------
// Dérivation de clé AES-256 à partir du mot de passe maître + sel
// -----------------------------------------------------------------------------
fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut key)
        .context("Échec de la dérivation de clé Argon2")?;
    Ok(key)
}

// -----------------------------------------------------------------------------
// Chiffrement du vault avec AES-256-GCM
// -----------------------------------------------------------------------------
fn encrypt_vault(vault: &Vault, master_password: &str) -> Result<EncryptedVault> {
    // 1. Sérialiser le vault en JSON
    let plaintext = serde_json::to_vec(vault).context("Sérialisation JSON")?;

    // 2. Générer un sel aléatoire pour Argon2 (16 octets)
    let salt: [u8; 16] = rand::random();
    let key = derive_key(master_password, &salt)?;

    // 3. Générer un nonce aléatoire (12 octets)
    let mut nonce_bytes = [0u8; AES_NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 4. Chiffrer
    let cipher = Aes256Gcm::new_from_slice(&key).context("Initialisation AES")?;
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .map_err(|e| anyhow::anyhow!("Échec chiffrement : {:?}", e))?;

    Ok(EncryptedVault {
        salt: base64::encode(salt),
        nonce: base64::encode(nonce_bytes),
        ciphertext: base64::encode(ciphertext),
    })
}

// -----------------------------------------------------------------------------
// Déchiffrement du vault
// -----------------------------------------------------------------------------
fn decrypt_vault(encrypted: &EncryptedVault, master_password: &str) -> Result<Vault> {
    // 1. Décoder sel, nonce, ciphertext
    let salt = base64::decode(&encrypted.salt).context("Sel invalide")?;
    let nonce_bytes = base64::decode(&encrypted.nonce).context("Nonce invalide")?;
    let ciphertext = base64::decode(&encrypted.ciphertext).context("Ciphertext invalide")?;

    if nonce_bytes.len() != AES_NONCE_SIZE {
        anyhow::bail!("Nonce doit faire 12 octets");
    }

    // 2. Re-dériver la clé
    let key = derive_key(master_password, &salt)?;

    // 3. Déchiffrer
    let cipher = Aes256Gcm::new_from_slice(&key).context("Initialisation AES")?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| anyhow::anyhow!("Mot de passe maître incorrect ou données corrompues"))?;

    // 4. Désérialiser
    let vault: Vault = serde_json::from_slice(&plaintext).context("JSON invalide")?;
    Ok(vault)
}

// -----------------------------------------------------------------------------
// Lecture / écriture du fichier coffre
// -----------------------------------------------------------------------------
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

// -----------------------------------------------------------------------------
// Helpers d'affichage
// -----------------------------------------------------------------------------
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

fn search_entries(vault: &Vault, query: &str) -> Vec<(usize, &VaultEntry)> {
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

// -----------------------------------------------------------------------------
// Demande du mot de passe maître (avec confirmation à la première utilisation)
// -----------------------------------------------------------------------------
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

// -----------------------------------------------------------------------------
// Fonction principale
// -----------------------------------------------------------------------------
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Déterminer si l'opération nécessite de forcer la création d'un nouveau vault
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