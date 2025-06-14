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
use cli::Cli;
use crate::cli::Commands;
use reap::tap;

#[cfg(debug_assertions)]
async fn test_parallel_runners() {
    use crate::config::ReapConfig;
    use crate::core::{install_with_priority, parallel_install, parallel_upgrade};
    use crate::tui::LogPane;
    let config = std::sync::Arc::new(ReapConfig::load());
    let log = std::sync::Arc::new(LogPane::default());
    parallel_install(&["yay".to_string(), "zsh".to_string()], config.clone(), log.clone()).await;
    parallel_upgrade(&["firefox".to_string(), "ripgrep".to_string()], config.clone(), log.clone()).await;
    install_with_priority("htop", config, true, log, &crate::core::InstallOptions::default()).await;
}

#[tokio::main]
async fn main() {
    // Auto-sync enabled taps before any command
    if let Err(e) = tap::sync_enabled_taps() {
        eprintln!("Warning: Failed to sync taps: {}", e);
    }
    #[cfg(debug_assertions)]
    tokio::spawn(test_parallel_runners());
    let cli = Cli::parse();
    // All install/upgrade flows use Reap's own async/parallel logic (no yay/paru fallback)
    if let Err(e) = core::handle_cli(&cli).await {
        eprintln!("[reap] CLI error: {e}");
        std::process::exit(1);
    }
    match cli.command {
        Commands::Audit { pkg } => core::handle_audit(&pkg),
        Commands::Rollback { pkg } => core::handle_rollback(&pkg),
        Commands::SyncDb => println!("Syncing pacman database..."),
        Commands::Pin { pkg } => {
            if let Err(e) = crate::utils::pin_package(&pkg) {
                eprintln!("[reap] Pin failed: {}", e);
            } else {
                println!("[reap] Pinned {}", pkg);
            }
        }
        Commands::Tui => {
            let config = config::ReapConfig::load();
            tokio::spawn(crate::tui::launch_tui());
        }
        _ => {}
    }
}
