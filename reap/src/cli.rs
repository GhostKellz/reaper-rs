use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "reap", version = "0.1.0", about = "Reaper: Secure, unified Rust-powered meta package manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install one or more packages
    Install {
        pkgs: Vec<String>,
    },
    /// Search for a package
    Search {
        query: String,
    },
    /// Upgrade all packages
    Upgrade,
    /// Rollback a package
    Rollback {
        pkg: String,
    },
    /// Pin a package
    Pin {
        pkg: String,
    },
    /// Launch the interactive TUI
    Tui,
    /// Clean package cache
    Clean,
    /// Run system doctor
    Doctor,
    /// GPG key refresh
    Gpg {
        #[command(subcommand)]
        cmd: GpgCmd,
    },
}

#[derive(Subcommand, Debug)]
pub enum GpgCmd {
    /// Refresh GPG keys
    Refresh,
}
