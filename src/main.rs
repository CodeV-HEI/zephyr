mod commands;
mod crypto;
mod repository;
mod utils;
mod vault;

use clap::Parser;
use commands::handle_command;
use utils::print_banner;

#[derive(Parser)]
#[command(name = "Zephyr Vault")]
#[command(about = "Coffre-fort local chiffré avec ASCII art", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: commands::Commands,
    /// Ne pas afficher la bannière
    #[arg(long, global = true)]
    quiet: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Affiche la bannière sauf si --quiet ou si la commande est Banner
    if !cli.quiet && !matches!(cli.command, commands::Commands::Banner) {
        print_banner();
    }

    handle_command(cli.command)
}
