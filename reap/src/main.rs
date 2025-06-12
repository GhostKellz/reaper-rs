mod aur;
mod backend;
mod cli;
mod config;
mod core;
mod flatpak;
mod gpg;
mod hooks;
mod pacman;
mod tui;
mod utils;

use clap::Parser;
use cli::{Cli, Commands, GpgCmd};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let backend = cli.backend.clone();
    match cli.command {
        Commands::Install { pkgs } => {
            let backend = core::get_backend(&backend);
            for pkg in pkgs {
                backend.install(&pkg).await;
            }
        }
        Commands::Remove { pkgs: _ } => todo!("Remove not yet implemented"),
        Commands::Search { query } => core::handle_search(query).await,
        Commands::Upgrade => core::handle_upgrade(),
        Commands::Rollback { pkg } => core::handle_rollback(&pkg),
        Commands::Pin { pkg } => core::handle_pin(pkg),
        Commands::Tui => core::handle_tui().await,
        Commands::Clean => core::handle_clean(),
        Commands::Doctor => core::handle_doctor(),
        Commands::Gpg { cmd } => match cmd {
            GpgCmd::Refresh => core::handle_gpg_refresh(),
        },
    }
}
