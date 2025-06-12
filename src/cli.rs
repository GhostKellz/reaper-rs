use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "reap",
    version = "0.1.0",
    about = "Reaper: Secure, unified Rust-powered meta package manager"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(short = 'S', long = "sync", value_name = "PKG", num_args = 0.., conflicts_with_all = ["remove", "search", "upgrade", "local"])]
    pub sync: Option<Vec<String>>,
    #[arg(short = 'R', long = "remove", value_name = "PKG", num_args = 0.., conflicts_with_all = ["sync", "search", "upgrade", "local"], help = "Remove a package")]
    pub remove: Option<Vec<String>>,
    #[arg(short = 'U', long = "local", value_name = "PATH", num_args = 0.., conflicts_with_all = ["sync", "remove", "search", "upgrade"])]
    pub local: Option<Vec<String>>,
    #[arg(short = 'S', long = "search", value_name = "TERM", num_args = 0.., conflicts_with_all = ["sync", "remove", "upgrade", "local"], help = "Search for a package")]
    pub search: Option<Vec<String>>,
    #[arg(short = 'y', long = "refresh", conflicts_with = "upgrade")]
    pub refresh: bool,
    #[arg(short = 'u', long = "upgrade", conflicts_with = "refresh")]
    pub upgrade: bool,
    #[arg(short = 'y', long = "syncdb", help = "Sync package database")] // -Sy
    pub syncdb: bool,
    #[arg(short = 'u', long = "upgradeall", help = "Upgrade all packages")] // -Su
    pub upgradeall: bool,
    #[arg(
        short = 'S',
        long = "install",
        value_name = "PKG",
        help = "Install a package"
    )]
    pub install: Option<String>,
    #[arg(
        long = "backend",
        value_name = "BACKEND",
        default_value = "aur",
        help = "Select backend: aur, flatpak"
    )]
    pub backend: String,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Install one or more packages
    Install { pkgs: Vec<String> },
    /// Remove one or more packages
    Remove { pkgs: Vec<String> },
    /// Install local packages
    Local { pkgs: Vec<String> },
    /// Search for packages
    Search { terms: Vec<String> },
    /// Upgrade all packages
    Upgrade,
    /// Upgrade all packages
    UpgradeAll,
    /// Upgrade Flatpak packages
    FlatpakUpgrade,
    /// Audit a package
    Audit { pkg: String },
    /// Rollback a package
    Rollback { pkg: String },
    /// Sync package database
    SyncDb,
    /// Pin a package
    Pin { pkg: String },
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
    /// Import a GPG key
    Import { keyid: String },
    /// Show details of a GPG key
    Show { keyid: String },
}
