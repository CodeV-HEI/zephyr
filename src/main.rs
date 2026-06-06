use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{Context, Result, anyhow};
use argon2::Argon2;
use base64::Engine;
use clap::{Parser, Subcommand};
use rand::{Rng, RngCore, seq::SliceRandom};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

// ------------------------------------------------------------
// 1. Modèle de données
// ------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultEntry {
    pub service: String,
    pub username: String,
    pub password: String,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vault {
    pub entries: Vec<VaultEntry>,
}

// ------------------------------------------------------------
// 2. Repository (pattern)
// ------------------------------------------------------------
const VAULT_FILE: &str = "vault.enc";
const AES_NONCE_SIZE: usize = 12;

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

// ------------------------------------------------------------
// 3. Chiffrement (Factory simple)
// ------------------------------------------------------------
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedVault {
    salt: String,
    nonce: String,
    ciphertext: String,
}

fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    let argon2 = Argon2::default();
    let mut key = [0u8; 32];
    argon2
        .hash_password_into(master_password.as_bytes(), salt, &mut key)
        .map_err(|e| anyhow!("Dérivation clé échouée: {}", e))?;
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
        .map_err(|e| anyhow!("Échec chiffrement: {:?}", e))?;

    let engine = base64::engine::general_purpose::STANDARD;
    Ok(EncryptedVault {
        salt: engine.encode(salt),
        nonce: engine.encode(nonce_bytes),
        ciphertext: engine.encode(ciphertext),
    })
}

fn decrypt_vault(encrypted: &EncryptedVault, master_password: &str) -> Result<Vault> {
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

// ------------------------------------------------------------
// 4. Commandes CLI enrichies
// ------------------------------------------------------------
#[derive(Parser)]
#[command(name = "Zephyr Vault")]
#[command(about = "Coffre-fort local chiffré avec ASCII art", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ajouter un nouveau compte (service, identifiant)
    Add {
        service: String,
        username: String,
        /// Mot de passe (optionnel, sinon demande interactive ou génération)
        password: Option<String>,
        /// Générer automatiquement un mot de passe fort
        #[arg(long)]
        generate: bool,
        /// Longueur du mot de passe généré (défaut 16)
        #[arg(long, default_value = "16")]
        length: usize,
        /// Inclure des symboles dans le mot de passe généré
        #[arg(long)]
        symbols: bool,
    },
    /// Lister tous les comptes (mots de passe masqués)
    List {
        /// Afficher les mots de passe en clair (dangereux)
        #[arg(long)]
        show: bool,
    },
    /// Rechercher un compte (service ou identifiant)
    Search {
        query: String,
        /// Afficher le mot de passe en clair
        #[arg(long)]
        show: bool,
    },
    /// Supprimer un compte par index
    Remove { index: usize },
    /// Générer un mot de passe fort (sans l'enregistrer)
    Generate {
        #[arg(long, default_value = "16")]
        length: usize,
        #[arg(long)]
        symbols: bool,
    },
    /// Exporter toutes les données (déchiffrées) vers un fichier CSV
    Export { filename: String },
    /// Importer depuis un fichier CSV (format: service,username,password)
    Import { filename: String },
    /// Afficher la bannière ASCII
    Banner,
}

// ------------------------------------------------------------
// 5. Helpers (génération, affichage, etc.)
// ------------------------------------------------------------
fn generate_password(length: usize, use_symbols: bool) -> String {
    const LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    const DIGITS: &[u8] = b"0123456789";
    const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

    let mut charset = Vec::new();
    charset.extend_from_slice(LETTERS);
    charset.extend_from_slice(DIGITS);
    if use_symbols {
        charset.extend_from_slice(SYMBOLS);
    }

    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| *charset.choose(&mut rng).unwrap() as char)
        .collect()
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

fn list_entries(vault: &Vault, show_passwords: bool) {
    if vault.entries.is_empty() {
        println!("📭 Coffre vide.");
        return;
    }
    println!("Index | Service              | Username             | Password");
    println!("------+----------------------+----------------------+------------------");
    for (i, entry) in vault.entries.iter().enumerate() {
        print_entry(i, entry, show_passwords);
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

// ------------------------------------------------------------
// 6. ASCII Banner
// ------------------------------------------------------------
fn print_banner() {
    let banner = r#"
    ███████╗███████╗██████╗ ██╗  ██╗██╗   ██╗██████╗ 
    ╚══███╔╝██╔════╝██╔══██╗██║  ██║╚██╗ ██╔╝██╔══██╗
      ███╔╝ █████╗  ██████╔╝███████║ ╚████╔╝ ██████╔╝
     ███╔╝  ██╔══╝  ██╔══██╗██╔══██║  ╚██╔╝  ██╔══██╗
    ███████╗███████╗██║  ██║██║  ██║   ██║   ██║  ██║
    ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝
                     ZEPHYR VAULT
              Coffre-fort local chiffré
    "#;
    println!("{}", banner);
}

// ------------------------------------------------------------
// 7. Master password (avec cache simple pour la session)
// ------------------------------------------------------------
static mut MASTER_PASSWORD_CACHE: Option<String> = None;

fn get_master_password(force_create: bool) -> Result<String> {
    unsafe {
        if let Some(pwd) = &MASTER_PASSWORD_CACHE {
            return Ok(pwd.clone());
        }
    }
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
        unsafe {
            MASTER_PASSWORD_CACHE = Some(pwd.clone());
        }
        return Ok(pwd);
    }
    let pwd = rpassword::prompt_password("Mot de passe maître: ")?;
    unsafe {
        MASTER_PASSWORD_CACHE = Some(pwd.clone());
    }
    Ok(pwd)
}

// ------------------------------------------------------------
// 8. Export / Import CSV
// ------------------------------------------------------------
fn export_csv(vault: &Vault, filename: &str) -> Result<()> {
    let mut wtr = csv::Writer::from_path(filename)?;
    wtr.write_record(&["service", "username", "password"])?;
    for entry in &vault.entries {
        wtr.write_record(&[&entry.service, &entry.username, &entry.password])?;
    }
    wtr.flush()?;
    println!("✅ Exporté vers {}", filename);
    Ok(())
}

fn import_csv(filename: &str, vault: &mut Vault) -> Result<()> {
    let mut rdr = csv::Reader::from_path(filename)?;
    for result in rdr.records() {
        let record = result?;
        if record.len() >= 3 {
            let entry = VaultEntry {
                service: record[0].to_string(),
                username: record[1].to_string(),
                password: record[2].to_string(),
                created_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            vault.entries.push(entry);
        }
    }
    println!("✅ Import terminé.");
    Ok(())
}

// ------------------------------------------------------------
// 9. Main
// ------------------------------------------------------------
fn main() -> Result<()> {
    let cli = Cli::parse();

    // Afficher la bannière pour toute commande sauf "banner" (évite double)
    if !matches!(cli.command, Commands::Banner) {
        print_banner();
    }

    match cli.command {
        Commands::Banner => {
            print_banner();
            return Ok(());
        }
        Commands::Generate { length, symbols } => {
            let pwd = generate_password(length, symbols);
            println!("{}", pwd);
            return Ok(());
        }
        Commands::Export { filename } => {
            let master_pwd = get_master_password(false)?;
            let vault = VaultRepository::load(&master_pwd)?;
            export_csv(&vault, &filename)?;
            return Ok(());
        }
        Commands::Import { filename } => {
            let master_pwd = get_master_password(false)?;
            let mut vault = VaultRepository::load(&master_pwd)?;
            import_csv(&filename, &mut vault)?;
            VaultRepository::save(&vault, &master_pwd)?;
            return Ok(());
        }
        _ => {}
    }

    let master_pwd = get_master_password(false)?;
    let mut vault = VaultRepository::load(&master_pwd)?;

    match cli.command {
        Commands::Add {
            service,
            username,
            password,
            generate,
            length,
            symbols,
        } => {
            let final_password = if generate {
                generate_password(length, symbols)
            } else if let Some(p) = password {
                p
            } else {
                print!("Mot de passe (non affiché) : ");
                io::stdout().flush()?;
                rpassword::read_password()?
            };
            if final_password.is_empty() {
                anyhow::bail!("Le mot de passe ne peut pas être vide");
            }
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let entry = VaultEntry {
                service,
                username,
                password: final_password,
                created_at: now,
                updated_at: now,
            };
            vault.entries.push(entry);
            VaultRepository::save(&vault, &master_pwd)?;
            println!("✅ Compte ajouté avec succès !");
        }
        Commands::List { show } => {
            list_entries(&vault, show);
        }
        Commands::Search { query, show } => {
            let results = search_entries(&vault, &query);
            if results.is_empty() {
                println!("Aucun résultat pour '{}'", query);
            } else {
                println!("Index | Service              | Username             | Password");
                println!("------+----------------------+----------------------+------------------");
                for (idx, entry) in results {
                    print_entry(idx, entry, show);
                }
            }
        }
        Commands::Remove { index } => {
            if index >= vault.entries.len() {
                anyhow::bail!("Index invalide. Utilisez `list` pour voir les indices.");
            }
            let removed = vault.entries.remove(index);
            VaultRepository::save(&vault, &master_pwd)?;
            println!("🗑️  Supprimé : {} ({})", removed.service, removed.username);
        }
        _ => unreachable!(),
    }

    Ok(())
}