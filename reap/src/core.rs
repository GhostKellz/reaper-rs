use crate::aur;
use crate::aur::SearchResult;
use crate::backend::{AurBackend, Backend};
use crate::cli::Cli;
use crate::config::ReapConfig;
use crate::flatpak;
use crate::pacman;
use crate::tui::LogPane;
use crate::utils;
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
    let mut tasks = Vec::new();
    for &pkg in pkgs {
        let pkg = pkg.to_string();
        tasks.push(tokio::spawn(async move {
            aur::install(vec![&pkg]).await;
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
    let priorities = vec![Source::Aur, Source::Pacman, Source::Flatpak];
    for src in priorities {
        match src {
            Source::Aur => {
                if aur::aur_search_results(pkg).iter().any(|r| r.name == pkg) {
                    aur::install(vec![pkg]).await;
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

pub async fn install(pkg: &str) {
    match detect_source(pkg) {
        Some(Source::Aur) => {
            aur::install(vec![pkg]).await;
        }
        Some(Source::Flatpak) => {
            flatpak::install_flatpak(pkg);
        }
        _ => eprintln!("[reap] Could not detect source for '{}'.", pkg),
    }
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
        backend.install(&pkg);
    }
}

pub async fn handle_install_parallel(pkgs: Vec<String>, max_parallel: usize) {
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

pub async fn handle_search(query: String) {
    let results: Vec<SearchResult> = aur::search(&query).await.unwrap_or_else(|_| Vec::new());
    for result in &results {
        let installed = pacman::is_installed(&result.name);
        let marker = if installed { "[*]" } else { "   " };
        println!(
            "{} {} {} - {}",
            marker, result.name, result.version, result.description
        );
    }
}

pub fn handle_upgrade() {
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
    utils::rollback(pkg);
}

pub fn handle_audit(pkg: &str) {
    utils::audit_package(pkg);
}

pub async fn handle_tui() {
    crate::tui::launch_tui().await;
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

pub fn handle_pin(_pkg: String) {
    println!("[reap] Pinning not yet implemented.");
}

pub fn handle_clean() {
    println!("[reap] Clean not yet implemented.");
}

pub fn handle_gpg_refresh() {
    crate::gpg::refresh_keys();
}

/// Handle CLI commands based on the provided `Cli` struct
pub async fn handle_cli(cli: &Cli) {
    if let Some(pkgs) = &cli.sync {
        for pkg in pkgs {
            aur::install(vec![pkg]).await;
        }
    }
    if let Some(pkgs) = &cli.remove {
        for pkg in pkgs {
            aur::uninstall(pkg);
        }
    }
    if let Some(pkgs) = &cli.local {
        for path in pkgs {
            aur::install_local(path);
        }
    }
    if let Some(terms) = &cli.search {
        for term in terms {
            aur::search(term).await;
        }
    }
    if cli.upgradeall {
        upgrade_all().await;
    }
    if cli.syncdb {
        aur::sync_db().await;
    }
    if let Some(pkg) = &cli.install {
        aur::install(vec![pkg]).await;
    }
}

pub async fn upgrade_all() {
    println!("[reap] Upgrading AUR packages...");
    aur::upgrade_all().await;
    println!("[reap] Upgrading Flatpak packages...");
    flatpak::upgrade_flatpak();
    println!("[reap] All enabled backends upgraded.");
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
