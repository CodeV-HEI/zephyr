use anyhow::{Result, bail};
use clap::Subcommand;
use std::io::{self, Write};

use crate::vault::{Vault, VaultEntry};
use crate::repository::VaultRepository;
use crate::utils::{get_master_password, generate_password, list_entries, search_entries, print_entry};

#[derive(Subcommand)]
pub enum Commands {
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
    List {
        #[arg(long)]
        show: bool,
    },
    Search {
        query: String,
        #[arg(long)]
        show: bool,
    },
    Remove { index: usize },
    Generate {
        #[arg(long, default_value = "16")]
        length: usize,
        #[arg(long)]
        symbols: bool,
    },
    Export { filename: String },
    Import { filename: String },
    Banner,
}

pub fn handle_command(cmd: Commands) -> Result<()> {
    match cmd {
        Commands::Banner => {
            crate::utils::print_banner();
            Ok(())
        }
        Commands::Generate { length, symbols } => {
            let pwd = generate_password(length, symbols);
            println!("{}", pwd);
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
            Ok(())
        }
        _ => {
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
                        print!("Mot de passe (non affiché) : ");
                        io::stdout().flush()?;
                        rpassword::read_password()?
                    };
                    if final_password.is_empty() {
                        bail!("Le mot de passe ne peut pas être vide");
                    }
                    let entry = VaultEntry::new(service, username, final_password);
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
                        bail!("Index invalide. Utilisez `list` pour voir les indices.");
                    }
                    let removed = vault.entries.remove(index);
                    VaultRepository::save(&vault, &master_pwd)?;
                    println!("🗑️  Supprimé : {} ({})", removed.service, removed.username);
                }
                _ => unreachable!(),
            }
            Ok(())
        }
    }
}

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
            let entry = VaultEntry::new(
                record[0].to_string(),
                record[1].to_string(),
                record[2].to_string(),
            );
            vault.entries.push(entry);
        }
    }
    println!("✅ Import terminé.");
    Ok(())
}