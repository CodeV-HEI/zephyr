mod vault;
mod crypto;
mod repository;
mod commands;
mod utils;

use clap::Parser;
use commands::handle_command;
use utils::print_banner;

#[derive(Parser)]
#[command(name = "Zephyr Vault")]
#[command(about = "Coffre-fort local chiffré avec ASCII art", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Affiche la bannière sauf pour la commande Banner (qui l'affiche elle-même)
    if !matches!(cli.command, commands::Commands::Banner) {
        print_banner();
    }

    handle_command(cli.command)
}