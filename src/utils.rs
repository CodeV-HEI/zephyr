use anyhow::{Result, bail};
use rand::seq::SliceRandom;
// use rand::Rng;
use rpassword;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::repository::VAULT_FILE;

static MASTER_PASSWORD_CACHE: OnceLock<String> = OnceLock::new();

pub fn get_master_password(force_create: bool) -> Result<String> {
    if let Some(pwd) = MASTER_PASSWORD_CACHE.get() {
        return Ok(pwd.clone());
    }

    let vault_exists = PathBuf::from(VAULT_FILE).exists();
    if !vault_exists || force_create {
        println!("🔐 Création d'un nouveau coffre");
        let pwd = rpassword::prompt_password("Nouveau mot de passe maître: ")?;
        let confirm = rpassword::prompt_password("Confirmation: ")?;
        if pwd != confirm {
            bail!("Les mots de passe ne correspondent pas");
        }
        if pwd.is_empty() {
            bail!("Le mot de passe ne peut pas être vide");
        }
        MASTER_PASSWORD_CACHE.set(pwd.clone()).unwrap();
        return Ok(pwd);
    }

    let pwd = rpassword::prompt_password("Mot de passe maître: ")?;
    MASTER_PASSWORD_CACHE.set(pwd.clone()).unwrap();
    Ok(pwd)
}

pub fn generate_password(length: usize, use_symbols: bool) -> String {
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

pub fn print_banner() {
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

pub fn print_entry(index: usize, entry: &crate::vault::VaultEntry, show_password: bool) {
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

pub fn list_entries(vault: &crate::vault::Vault, show_passwords: bool) {
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

pub fn search_entries<'a>(
    vault: &'a crate::vault::Vault,
    query: &str,
) -> Vec<(usize, &'a crate::vault::VaultEntry)> {
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