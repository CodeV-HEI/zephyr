use anyhow::{Result, bail};
use clap::Subcommand;
use colored::*;
use std::io::{self, Write};

use crate::repository::VaultRepository;
use crate::utils::{
    compute_column_widths, generate_password, get_master_password, list_entries, print_entry,
    search_entries,
};
use crate::vault::{Vault, VaultEntry};

#[derive(Subcommand)]
pub enum Commands {
    /// Ajouter un nouveau compte
    Add {
        service: String,
        username: String,
        password: Option<String>,
        #[arg(long)]
        generate: bool,
        #[arg(long, default_value = "16")]
        length: usize,
        #[arg(long)]
        symbols: bool,
    },
    /// Lister tous les comptes
    List {
        #[arg(long)]
        show: bool,
        #[arg(long, short)]
        quiet: bool,
    },
    /// Rechercher un compte
    Search {
        query: String,
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
    /// Exporter en CSV
    Export { filename: String },
    /// Importer depuis CSV
    Import { filename: String },
    /// Copier le mot de passe d'un compte dans le presse-papiers
    Copy {
        /// Index ou texte de recherche (service ou username)
        target: String,
    },
    /// Afficher la bannière
    Banner,
    /// Supprimer tout le coffre (danger)
    Wipe {
        #[arg(long)]
        force: bool,
    },
}

pub fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Banner => {
            crate::utils::print_banner();
            Ok(())
        }
        Commands::Generate { length, symbols } => {
            let pwd = generate_password(length, symbols);
            println!("{}", pwd.green());
            Ok(())
        }
        Commands::Export { filename } => {
            let master_pwd = get_master_password(false)?;
            let vault = VaultRepository::load(&master_pwd)?;
            export_csv(&vault, &filename)?;
            Ok(())
        }
        Commands::Import { filename } => {
            let master_pwd = get_master_password(false)?;
            let mut vault = VaultRepository::load(&master_pwd)?;
            import_csv(&filename, &mut vault)?;
            VaultRepository::save(&vault, &master_pwd)?;
            println!("{}", "✅ Import terminé.".green());
            Ok(())
        }
        Commands::Wipe { force } => {
            if !force {
                println!(
                    "{}",
                    "⚠️  Utilisez --force pour supprimer définitivement le coffre.".yellow()
                );
                return Ok(());
            }
            std::fs::remove_file(crate::repository::VAULT_FILE)?;
            println!("{}", "🗑️  Coffre effacé.".red());
            Ok(())
        }
        Commands::Copy { target } => {
            let master_pwd = get_master_password(false)?;
            let vault = VaultRepository::load(&master_pwd)?;
            // On cherche par index (si numérique) ou par texte
            let entry: Option<&VaultEntry> = if let Ok(idx) = target.parse::<usize>() {
                vault.entries.get(idx)
            } else {
                let results = search_entries(&vault, &target);
                if results.len() == 1 {
                    Some(results[0].1)
                } else {
                    None
                }
            };
            match entry {
                Some(e) => {
                    let mut clipboard = arboard::Clipboard::new()?;
                    clipboard.set_text(e.password.clone())?;
                    println!(
                        "{} '{}'",
                        "📋 Mot de passe copié pour".green(),
                        e.service.cyan()
                    );
                }
                None => {
                    bail!("{}", "Aucun compte unique trouvé.".red());
                }
            }
            Ok(())
        }
        _ => {
            let quiet = matches!(cmd, Commands::List { quiet: true, .. });
            if !quiet {
                crate::utils::print_banner();
            }
            let master_pwd = get_master_password(false)?;
            let mut vault = VaultRepository::load(&master_pwd)?;

            match cmd {
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
                        print!("{}", "Mot de passe (non affiché) : ".yellow());
                        io::stdout().flush()?;
                        rpassword::read_password()?
                    };
                    if final_password.is_empty() {
                        bail!("{}", "Le mot de passe ne peut pas être vide".red());
                    }
                    let entry = VaultEntry::new(service, username, final_password);
                    vault.entries.push(entry);
                    VaultRepository::save(&vault, &master_pwd)?;
                    println!("{}", "✅ Compte ajouté avec succès !".green());
                }
                Commands::List { show, quiet: _ } => {
                    list_entries(&vault, show);
                }
                Commands::Search { query, show } => {
                    let results = search_entries(&vault, &query);
                    if results.is_empty() {
                        println!("{}", format!("Aucun résultat pour '{}'", query).yellow());
                    } else {
                        let widths = compute_column_widths(&vault);
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
                        for (idx, entry) in results {
                            print_entry(idx, entry, show, widths);
                        }
                    }
                }
                Commands::Remove { index } => {
                    if index >= vault.entries.len() {
                        bail!(
                            "{}",
                            "Index invalide. Utilisez `list` pour voir les indices.".red()
                        );
                    }
                    let removed = vault.entries.remove(index);
                    VaultRepository::save(&vault, &master_pwd)?;
                    println!(
                        "{} {} ({})",
                        "🗑️ Supprimé :".red(),
                        removed.service,
                        removed.username
                    );
                }
                _ => unreachable!(),
            }
            Ok(())
        }
    }
}

fn export_csv(vault: &Vault, filename: &str) -> Result<()> {
    let mut wtr = csv::Writer::from_path(filename)?;
    wtr.write_record(["service", "username", "password"])?;
    for entry in &vault.entries {
        wtr.write_record([&entry.service, &entry.username, &entry.password])?;
    }
    wtr.flush()?;
    println!("{} {}", "✅ Exporté vers".green(), filename);
    Ok(())
}

fn import_csv(filename: &str, vault: &mut Vault) -> Result<()> {
    let mut rdr = csv::Reader::from_path(filename)?;
    for result in rdr.records() {
        let record = result?;
        if record.len() >= 3 {
            let entry = VaultEntry::new(
                record[0].to_string(),
                record[1].to_string(),
                record[2].to_string(),
            );
            vault.entries.push(entry);
        }
    }
    Ok(())
}
