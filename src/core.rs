use crate::aur;
use crate::aur::SearchResult;
use crate::aur::upgrade_all;
use crate::aur::handle_search;
use crate::backend::{AurBackend, Backend};
use crate::cli::Cli;
use crate::config::ReapConfig;
use crate::flatpak;
use crate::pacman;
use crate::tui::LogPane;
use crate::utils;
use crate::hooks;
use crate::tui;
use crate::utils::{pkgb_diff_audit, audit_flatpak_manifest};
use crate::tui::{setup_terminal, restore_terminal};
use crate::gpg;
use futures::FutureExt;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Aur,
    Pacman,
    Flatpak,
    ChaoticAur,
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Source::Aur => "[AUR]",
            Source::Pacman => "[PACMAN]",
            Source::Flatpak => "[FLATPAK]",
            Source::ChaoticAur => "[ChaoticAUR]",
        }
    }
}

pub fn get_installed_packages() -> HashMap<String, Source> {
    let mut pkgs = HashMap::new();
    // AUR (yay or pacman)
    let yay = which::which("yay").is_ok();
    let aur_cmd = if yay { "yay" } else { "pacman" };
    if let Ok(out) = Command::new(aur_cmd).arg("-Qq").output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            pkgs.insert(line.trim().to_string(), Source::Aur);
        }
    }
    // Flatpak
    if let Ok(out) = Command::new("flatpak").arg("list").arg("--app").output() {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            let name = line.split_whitespace().next().unwrap_or("");
            if !name.is_empty() {
                pkgs.insert(name.to_string(), Source::Flatpak);
            }
        }
    }
    pkgs
}

pub async fn unified_search(query: &str) -> Vec<SearchResult> {
    let aur_fut = async { aur::search(query).await.unwrap_or_else(|_| vec![]) };
    let pacman_fut = async { vec![] };
    let flatpak_fut = async { vec![] };
    let (aur, pacman, flatpak): (Vec<SearchResult>, Vec<SearchResult>, Vec<SearchResult>) =
        tokio::join!(aur_fut, pacman_fut, flatpak_fut);
    let mut results = Vec::new();
    results.extend(aur);
    results.extend(pacman);
    results.extend(flatpak);
    results
}

pub async fn parallel_install(pkgs: &[&str]) {
    // TODO: Wire this into CLI flow in core::handle_cli()
    let mut tasks = Vec::new();
    for &pkg in pkgs {
        let pkg = pkg.to_string();
        tasks.push(tokio::spawn(async move {
            match aur::install(vec![&pkg]).await {
                Ok(_) => println!("[reap] Installed {}", pkg),
                Err(e) => eprintln!("[reap] Install failed for {}: {:?}", pkg, e),
            }
            pkg
        }));
    }
    let results = join_all(tasks).await;
    let mut failed = Vec::new();
    for res in results {
        match res {
            Ok(pkg) => println!("[reap] Installed {}", pkg),
            Err(e) => {
                eprintln!("[reap] Install failed: {:?}", e);
                failed.push(e);
            }
        }
    }
    if !failed.is_empty() {
        eprintln!("[reap] Some installs failed.");
    } else {
        println!("[reap] All installs complete.");
    }
}

pub async fn parallel_upgrade(
    pkgs: Vec<String>,
    config: Arc<ReapConfig>,
    log: Option<Arc<LogPane>>,
) {
    let tasks: Vec<_> = pkgs
        .into_iter()
        .map(|pkg| {
            let config = config.clone();
            let log = log.clone();
            tokio::spawn(async move {
                install_with_priority(&pkg, &config).await;
                if let Some(log) = log.as_ref() {
                    log.push(&format!("[reap] Upgraded {}", pkg));
                } else {
                    println!("[reap] Upgraded {}", pkg);
                }
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })
        })
        .collect();
    let results = join_all(tasks).await;
    for result in results {
        if let Err(e) = result {
            if let Some(log) = log.as_ref() {
                log.push(&format!("❌ Upgrade failed: {:?}", e));
            } else {
                println!("❌ Upgrade failed: {:?}", e);
            }
        }
    }
    if let Some(log) = log.as_ref() {
        log.push("[reap] All upgrades complete.");
    } else {
        println!("[reap] All upgrades complete.");
    }
}

pub async fn install_with_priority(pkg: &str, _config: &ReapConfig) {
    // TODO: Wire this into CLI flow in core::handle_cli()
    let priorities = vec![Source::Aur, Source::Pacman, Source::Flatpak];
    for src in priorities {
        match src {
            Source::Aur => {
                if aur::aur_search_results(pkg).iter().any(|r| r.name == pkg) {
                    match aur::install(vec![pkg]).await {
                        Ok(_) => println!("[reap] Installed {}", pkg),
                        Err(e) => eprintln!("[reap] Install failed for {}: {:?}", pkg, e),
                    }
                    return;
                }
            }
            Source::Pacman => {
                let output = std::process::Command::new("pacman")
                    .arg("-Si")
                    .arg(pkg)
                    .output();
                if let Ok(out) = output {
                    if out.status.success() {
                        pacman::install(pkg);
                        return;
                    }
                }
            }
            Source::Flatpak => {
                let output = std::process::Command::new("flatpak")
                    .arg("info")
                    .arg(pkg)
                    .output();
                if let Ok(out) = output {
                    if out.status.success() {
                        flatpak::install(pkg);
                        return;
                    }
                }
            }
            _ => {}
        }
    }
    println!("[reap] Package '{}' not found in any source.", pkg);
}

pub fn detect_source(pkg: &str) -> Option<Source> {
    if aur::aur_search_results(pkg).iter().any(|r| r.name == pkg) {
        Some(Source::Aur)
    } else {
        let output = std::process::Command::new("flatpak")
            .arg("search")
            .arg(pkg)
            .output();
        if let Ok(out) = output {
            if out.status.success() && !String::from_utf8_lossy(&out.stdout).trim().is_empty() {
                return Some(Source::Flatpak);
            }
        }
        None
    }
}

pub async fn install(pkg: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match detect_source(pkg) {
        Some(Source::Aur) => {
            match aur::install(vec![pkg]).await {
                Ok(_) => println!("[reap] Installed {}", pkg),
                Err(e) => eprintln!("[reap] Install failed for {}: {:?}", pkg, e),
            }
        }
        Some(Source::Flatpak) => {
            match flatpak::install_flatpak(pkg).await {
                Ok(_) => println!("[reap] Installed {}", pkg),
                Err(e) => eprintln!("[reap] Install failed for {}: {:?}", pkg, e),
            }
        }
        _ => eprintln!("[reap] Could not detect source for '{}'.", pkg),
    }
    Ok(())
}

pub fn print_search_results(results: &[SearchResult]) {
    for r in results {
        println!(
            "{} {} {} - {}",
            r.source.label(),
            r.name,
            r.version,
            r.description
        );
    }
}

pub fn handle_install(pkgs: Vec<String>) {
    let backend: Box<dyn Backend> = Box::new(AurBackend::new());
    for pkg in pkgs {
        println!("[reap] Installing {}...", pkg);
        tokio::runtime::Runtime::new().unwrap().block_on(backend.install(&pkg));
    }
}

pub async fn handle_install_parallel(pkgs: Vec<String>, max_parallel: usize) {
    // TODO: Wire this into CLI flow in core::handle_cli()
    let semaphore = Arc::new(Semaphore::new(max_parallel));
    let pb = ProgressBar::new(pkgs.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Failed to create ProgressStyle")
            .progress_chars("#>-"),
    );
    let mut handles = Vec::new();
    for pkg in pkgs {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let pb = pb.clone();
        let pkg = pkg.clone();
        handles.push(tokio::spawn(async move {
            let _permit = permit;
            let _ = std::panic::AssertUnwindSafe(async {
                handle_install(vec![pkg.clone()]);
            })
            .catch_unwind()
            .await;
            pb.inc(1);
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }));
    }
    let _ = join_all(handles).await;
    pb.finish_with_message("All installs complete.");
    println!("[reap] All installs complete.");
}

pub fn handle_upgrade() {
    // TODO: Wire this into CLI flow in core::handle_cli()
    let config = crate::config::ReapConfig::load();
    let installed = crate::pacman::list_installed_aur();
    let mut to_upgrade: Vec<String> = Vec::new();
    for pkg in installed {
        if config.is_ignored(&pkg) {
            continue;
        }
        if let Ok(remote) = crate::aur::fetch_package_info(&pkg) {
            let local_ver = crate::pacman::get_version(&pkg);
            if local_ver.as_deref() != Some(&remote.version) {
                to_upgrade.push(pkg.to_string());
            }
        }
    }
    if to_upgrade.is_empty() {
        println!("[reap] All AUR packages up to date.");
        return;
    }
    println!("[reap] Upgrading: {:?}", to_upgrade);
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(handle_install_parallel(to_upgrade, config.parallel));
}

pub fn handle_rollback(pkg: &str) {
    // TODO: Wire this into CLI flow in core::handle_cli()
    utils::rollback(pkg);
}

pub fn handle_audit(pkg: &str) {
    match crate::core::detect_source(pkg) {
        Some(crate::core::Source::Aur) => {
            let pkgb = crate::aur::get_pkgbuild_preview(pkg);
            pkgb_diff_audit(pkg, &pkgb);
        }
        Some(crate::core::Source::Flatpak) => {
            let output = std::process::Command::new("flatpak")
                .arg("info")
                .arg(pkg)
                .output();
            if let Ok(out) = output {
                audit_flatpak_manifest(&String::from_utf8_lossy(&out.stdout), None);
            } else {
                println!("[AUDIT][FLATPAK] Could not get info for {}.", pkg);
            }
        }
        _ => println!("[AUDIT] Unknown package source for {}.", pkg),
    }
}

pub async fn handle_tui() {
    setup_terminal();
    crate::tui::launch_tui().await;
    restore_terminal();
}

pub fn handle_doctor() {
    let checks = [
        ("gpg", "GPG available", which::which("gpg").is_ok()),
        ("git", "Git available", which::which("git").is_ok()),
        (
            "makepkg",
            "makepkg available",
            which::which("makepkg").is_ok(),
        ),
        (
            "flatpak",
            "Flatpak available",
            which::which("flatpak").is_ok(),
        ),
    ];
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config/reaper/brew.lua");
    let config_ok = config_path.exists();
    let aur_ok = Command::new("curl")
        .arg("-sSf")
        .arg("https://aur.archlinux.org")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    println!("[reap doctor] System diagnostics:");
    for (_, label, ok) in &checks {
        println!("{} {}", if *ok { "✔" } else { "✗" }, label);
    }
    println!(
        "{} Config file: {}",
        if config_ok { "✔" } else { "✗" },
        config_path.display()
    );
    println!("{} AUR network access", if aur_ok { "✔" } else { "✗" });
}

/// Handle CLI commands based on the provided `Cli` struct
pub async fn handle_cli(cli: &Cli) {
    use crate::cli::{Commands, FlatpakCmd, GpgCmd};
    match &cli.command {
        Commands::Install { pkgs } => {
            // Add parallel flag logic if needed
            handle_install(pkgs.clone());
        }
        Commands::Remove { pkgs } => {
            for pkg in pkgs {
                aur::uninstall(pkg);
            }
        }
        Commands::Local { pkgs } => {
            for pkg in pkgs {
                aur::install_local(pkg);
            }
        }
        Commands::Search { terms } => {
            handle_search(terms).await;
        }
        Commands::UpgradeAll => {
            match upgrade_all().await {
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
            handle_audit(pkg);
        }
        Commands::Rollback { pkg } => {
            hooks::on_rollback(pkg);
            println!("[reap] Rollback hook triggered for {}", pkg);
        }
        Commands::SyncDb => {
            match aur::sync_db().await {
                Ok(_) => println!("[reap] Sync DB succeeded"),
                Err(e) => eprintln!("[reap] Sync DB failed: {:?}", e),
            }
        }
        Commands::Upgrade => {
            // Example: parallel upgrade logic
            let config = Arc::new(ReapConfig::load());
            let pkgs = aur::get_outdated();
            let log = None;
            parallel_upgrade(pkgs, config, log).await;
        }
        Commands::Pin { pkg } => {
            match utils::pin_package(pkg) {
                Ok(_) => println!("[reap] Pinned {}", pkg),
                Err(e) => eprintln!("[reap] Pin failed for {}: {}", pkg, e),
            }
        }
        Commands::Clean => {
            match utils::clean_cache() {
                Ok(msg) => println!("[reap] {}", msg),
                Err(e) => eprintln!("[reap] Clean failed: {}", e),
            }
        }
        Commands::Doctor => {
            utils::doctor_report().map_or_else(
                |e| eprintln!("[reap doctor] Error: {}", e),
                |msg| println!("[reap doctor] {}", msg),
            );
        }
        Commands::Tui => {
            setup_terminal();
            tui::run_ui().await;
            restore_terminal();
        }
        Commands::Flatpak { cmd } => match cmd {
            FlatpakCmd::Search { query } => {
                let results = flatpak::search(query);
                for result in results {
                    println!("{} {} - {}", result.name, result.version, result.description);
                }
            }
            FlatpakCmd::Install { pkg } => {
                match flatpak::install_flatpak(pkg).await {
                    Ok(_) => println!("[reap][flatpak] Installed {}", pkg),
                    Err(e) => eprintln!("[reap][flatpak] Install failed for {}: {}", pkg, e),
                }
            }
            FlatpakCmd::Upgrade => {
                match flatpak::upgrade_flatpak().await {
                    Ok(_) => println!("[reap][flatpak] Upgrade succeeded"),
                    Err(e) => eprintln!("[reap][flatpak] Upgrade failed: {}", e),
                }
            }
            FlatpakCmd::Audit { pkg } => {
                flatpak::print_flatpak_sandbox_info(pkg);
            }
        },
        Commands::Gpg { cmd } => match cmd {
            GpgCmd::Refresh => gpg::refresh_keys(),
            GpgCmd::Import { keyid } => {
                gpg::import_gpg_key_async(keyid).await;
            }
            GpgCmd::Show { keyid } => {
                gpg::show_gpg_key_info_async(keyid).await;
            }
            GpgCmd::Check { keyid } => {
                gpg::check_key(keyid).await;
            }
            GpgCmd::VerifyPkgbuild { path } => {
                let path = std::path::Path::new(path);
                let _ = gpg::gpg_check(path);
            }
        },
    }
}

pub fn get_backend(backend_str: &str) -> Box<dyn Backend> {
    match backend_str {
        "aur" => Box::new(AurBackend::new()),
        "flatpak" => Box::new(crate::backend::FlatpakBackend::new()),
        _ => {
            eprintln!(
                "[reap] Unknown backend: {}. Defaulting to AUR.",
                backend_str
            );
            Box::new(AurBackend::new())
        }
    }
}
