use anyhow::{Result, bail};
use colored::*;
use rand::seq::SliceRandom;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::repository::VAULT_FILE;
use crate::vault::VaultEntry;

static MASTER_PASSWORD_CACHE: OnceLock<String> = OnceLock::new();

pub fn get_master_password(force_create: bool) -> Result<String> {
    if let Some(pwd) = MASTER_PASSWORD_CACHE.get() {
        return Ok(pwd.clone());
    }

    let vault_exists = PathBuf::from(VAULT_FILE).exists();
    if !vault_exists || force_create {
        println!("{}", "🔐 Création d'un nouveau coffre".cyan().bold());
        let pwd = rpassword::prompt_password("Nouveau mot de passe maître: ")?;
        let confirm = rpassword::prompt_password("Confirmation: ")?;
        if pwd != confirm {
            bail!("{}", "Les mots de passe ne correspondent pas".red());
        }
        if pwd.is_empty() {
            bail!("{}", "Le mot de passe ne peut pas être vide".red());
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

/// Affiche une bannière ASCII propre (sans caractères cassés)
pub fn print_banner() {
    let banner = r#"
    ╔══════════════════════════════════════════════════════════╗
    ║  ███████╗███████╗██████╗ ██╗  ██╗██╗   ██╗██████╗        ║
    ║  ╚══███╔╝██╔════╝██╔══██╗██║  ██║╚██╗ ██╔╝██╔══██╗       ║
    ║    ███╔╝ █████╗  ██████╔╝███████║ ╚████╔╝ ██████╔╝       ║
    ║   ███╔╝  ██╔══╝  ██╔══██╗██╔══██║  ╚██╔╝  ██╔══██╗       ║
    ║  ███████╗███████╗██║  ██║██║  ██║   ██║   ██║  ██║       ║
    ║  ╚══════╝╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝       ║
    ║               ZEPHYR VAULT - v0.2.0                      ║
    ║           Coffre-fort local chiffré (AES-256)            ║
    ╚══════════════════════════════════════════════════════════╝
    "#;
    println!("{}", banner.cyan());
}

/// Affiche une entrée avec largeurs de colonnes adaptatives
pub fn print_entry(
    index: usize,
    entry: &VaultEntry,
    show_password: bool,
    col_widths: (usize, usize, usize),
) {
    let pwd = if show_password {
        entry.password.as_str()
    } else {
        "********"
    };
    println!(
        "{:width_index$} │ {:<width_service$} │ {:<width_username$} │ {}",
        index,
        entry.service,
        entry.username,
        pwd,
        width_index = col_widths.0,
        width_service = col_widths.1,
        width_username = col_widths.2,
    );
}

/// Calcule les largeurs maximales pour chaque colonne
pub fn compute_column_widths(vault: &crate::vault::Vault) -> (usize, usize, usize) {
    let max_index = (vault.entries.len() - 1).to_string().len();
    let max_service = vault
        .entries
        .iter()
        .map(|e| e.service.len())
        .max()
        .unwrap_or(10)
        .max(15); // au moins 15
    let max_username = vault
        .entries
        .iter()
        .map(|e| e.username.len())
        .max()
        .unwrap_or(10)
        .max(15);
    (max_index, max_service, max_username)
}

pub fn list_entries(vault: &crate::vault::Vault, show_passwords: bool) {
    if vault.entries.is_empty() {
        println!("{}", "📭 Coffre vide.".yellow());
        return;
    }
    let widths = compute_column_widths(vault);
    let separator = format!(
        "{:-<width_index$}─┼─{:-<width_service$}─┼─{:-<width_username$}─┼─{:-<20}",
        "",
        "",
        "",
        "",
        width_index = widths.0,
        width_service = widths.1,
        width_username = widths.2,
    );
    println!(
        "{:width_index$} │ {:width_service$} │ {:width_username$} │ {}",
        "Index".bold(),
        "Service".bold(),
        "Username".bold(),
        "Password".bold(),
        width_index = widths.0,
        width_service = widths.1,
        width_username = widths.2,
    );
    println!("{}", separator);
    for (i, entry) in vault.entries.iter().enumerate() {
        print_entry(i, entry, show_passwords, widths);
    }
}

pub fn search_entries<'a>(
    vault: &'a crate::vault::Vault,
    query: &str,
) -> Vec<(usize, &'a VaultEntry)> {
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
