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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let _ = cli.backend.clone();
    if let Err(e) = core::handle_cli(&cli).await {
        eprintln!("[reap] CLI error: {e}");
        std::process::exit(1);
    }
}
