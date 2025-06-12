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
    let _ = cli.backend.clone();
    match cli.command {
        Commands::Install { pkgs } => {
            for pkg in pkgs {
                match aur::install(vec![&pkg]).await {
                    Ok(_) => println!("[reap] Installed {}", pkg),
                    Err(e) => eprintln!("[reap] Install failed for {}: {:?}", pkg, e),
                }
            }
        }
        Commands::Remove { pkgs } => {
            for pkg in pkgs {
                aur::uninstall(&pkg);
            }
        }
        Commands::Local { pkgs } => {
            for pkg in pkgs {
                aur::install_local(&pkg);
            }
        }
        Commands::Search { terms } => {
            core::handle_search(&terms).await;
        }
        Commands::Upgrade => core::handle_upgrade(),
        Commands::UpgradeAll => {
            match aur::upgrade_all().await {
                Ok(_) => println!("[reap] Upgrade all succeeded"),
                Err(e) => eprintln!("[reap] Upgrade all failed: {:?}", e),
            }
        }
        Commands::FlatpakUpgrade => {
            match flatpak::upgrade_flatpak().await {
                Ok(_) => println!("[reap] Flatpak upgrade succeeded"),
                Err(e) => eprintln!("[reap] Flatpak upgrade failed: {:?}", e),
            }
        }
        Commands::Audit { pkg } => {
            utils::audit_package(&pkg);
            println!("[reap] Audit complete for {}", pkg);
        }
        Commands::Rollback { pkg } => {
            core::handle_rollback(&pkg);
            println!("[reap] Rollback complete for {}", pkg);
        }
        Commands::Pin { pkg } => core::handle_pin(pkg),
        Commands::SyncDb => {
            match aur::sync_db().await {
                Ok(_) => println!("[reap] Sync DB succeeded"),
                Err(e) => eprintln!("[reap] Sync DB failed: {:?}", e),
            }
        }
        Commands::Tui => core::handle_tui().await,
        Commands::Clean => core::handle_clean(),
        Commands::Doctor => core::handle_doctor(),
        Commands::Gpg { cmd } => match cmd {
            GpgCmd::Refresh => core::handle_gpg_refresh(),
        },
    }
}
