mod commands;
pub mod manifest;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rmake")]
#[command(version = "0.1.0")]
#[command(about = "Reaper Maker — a makepkg replacement", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Build,
    Install,
    Clean,
    Info,
    Init,
    Lint,
    Sign,
    Verify,
    Publish,
    Sync,
    Submit,
    Upload,
    Test,
    Audit,
    Deps,
    Diff,
    Rebuild,
    Sandbox,
    Watch,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Build => commands::build::run(),
        Commands::Install => commands::install::run(),
        Commands::Clean => commands::clean::run(),
        Commands::Info => commands::info::run(),
        Commands::Init => commands::init::run(),
        Commands::Lint => commands::lint::run(),
        Commands::Sign => commands::sign::run(),
        Commands::Verify => commands::verify::run(),
        Commands::Publish => commands::publish::run(),
        Commands::Sync => commands::sync::run(),
        Commands::Submit => commands::submit::run(),
        Commands::Upload => commands::upload::run(),
        Commands::Test => commands::test::run(),
        Commands::Audit => commands::audit::run(),
        Commands::Deps => commands::deps::run(),
        Commands::Diff => commands::diff::run(),
        Commands::Rebuild => commands::rebuild::run(),
        Commands::Sandbox => commands::sandbox::run(),
        Commands::Watch => commands::watch::run(),
    }
}
