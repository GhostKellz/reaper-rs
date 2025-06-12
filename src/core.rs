use crate::aur;
use crate::aur::SearchResult;
use crate::aur::upgrade_all;
use crate::backend::{AurBackend, Backend};
use crate::cli::Cli;
use crate::config::ReapConfig;
use crate::flatpak;
use crate::gpg;
use crate::pacman;
use crate::tui;
use crate::tui::LogPane;
use crate::tui::{restore_terminal, setup_terminal};
use crate::utils;
use crate::utils::{audit_flatpak_manifest, pkgb_diff_audit};
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
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Source::Aur => "[AUR]",
            Source::Pacman => "[PACMAN]",
            Source::Flatpak => "[FLATPAK]",
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
                install_with_priority(&pkg, &config, false).await;
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

pub async fn install_with_priority(pkg: &str, _config: &ReapConfig, edit: bool) {
    let priorities = vec![Source::Aur, Source::Pacman, Source::Flatpak];
    for src in priorities {
        match src {
            Source::Aur => {
                if aur::aur_search_results(pkg).iter().any(|r| r.name == pkg) {
                    let tmpdir = std::env::temp_dir().join(format!("reap-aur-{}", pkg));
                    if std::fs::create_dir_all(&tmpdir).is_ok() {
                        let pkgb = aur::get_pkgbuild_preview(pkg);
                        let pkgb_path = tmpdir.join("PKGBUILD");
                        if std::fs::write(&pkgb_path, pkgb).is_ok() {
                            println!("[reap] building package in {} (edit: {})", tmpdir.display(), edit);
                            match utils::build_pkg(&tmpdir, edit) {
                                Ok(_) => {
                                    let verify_result = gpg::gpg_check(&tmpdir);
                                    match verify_result {
                                        Ok(_) => println!("[reap] PKGBUILD signature verified and trusted."),
                                        Err(e) => {
                                            eprintln!("[reap] PKGBUILD verification failed: {e}");
                                        }
                                    }
                                    println!("[reap] Built and installed {}", pkg);
                                }
                                Err(e) => eprintln!("[reap] Build failed for {}: {}", pkg, e),
                            }
                        } else {
                            eprintln!("[reap] Failed to write PKGBUILD for {}", pkg);
                        }
                    } else {
                        eprintln!("[reap] Failed to create temp dir for {}", pkg);
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
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(backend.install(&pkg));
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

pub fn handle_upgrade(parallel: bool) {
    let config = crate::config::ReapConfig::load();
    let installed = crate::pacman::list_installed_aur();
    let mut to_upgrade: Vec<String> = Vec::new();
    for pkg in installed {
        if config.is_ignored(&pkg) {
            println!("[reap] Skipping ignored package: {}", pkg);
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
    if parallel {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_install_parallel(to_upgrade, config.parallel));
    } else {
        for pkg in to_upgrade {
            let _ = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(crate::aur::install(vec![pkg.as_str()]));
        }
    }
}

pub fn handle_rollback(pkg: &str) {
    utils::rollback(pkg);
    crate::hooks::on_rollback(pkg);
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

/// Handle CLI commands based on the provided `Cli` struct
pub async fn handle_cli(cli: &Cli) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::cli::{Commands, FlatpakCmd, GpgCmd, TapCmd};
    match &cli.command {
        Commands::Install { pkgs, parallel } => {
            if *parallel {
                handle_install_parallel(pkgs.clone(), ReapConfig::load().parallel).await;
            } else {
                handle_install(pkgs.clone());
            }
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
            for term in terms {
                match aur::search(term).await {
                    Ok(results) => print_search_results(&results),
                    Err(e) => eprintln!("[reap] Search failed for '{}': {}", term, e),
                }
            }
        }
        Commands::UpgradeAll => {
            upgrade_all().await?;
            println!("[reap] Upgrade all succeeded");
        }
        Commands::FlatpakUpgrade => {
            flatpak::upgrade_flatpak().await?;
            println!("[reap] Flatpak upgrade succeeded");
        }
        Commands::Audit { pkg } => {
            handle_audit(pkg);
        }
        Commands::Rollback { pkg } => {
            handle_rollback(pkg);
        }
        Commands::SyncDb => {
            aur::sync_db().await?;
            println!("[reap] Sync DB succeeded");
        }
        Commands::Upgrade { parallel } => {
            handle_upgrade(*parallel);
        }
        Commands::Pin { pkg } => match utils::pin_package(pkg) {
            Ok(_) => println!("[reap] Pinned {}", pkg),
            Err(e) => eprintln!("[reap] Pin failed for {}: {}", pkg, e),
        },
        Commands::Clean => match utils::clean_cache() {
            Ok(msg) => println!("[reap] {}", msg),
            Err(e) => eprintln!("[reap] Clean failed: {}", e),
        },
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
                print_search_results(&results);
            }
            FlatpakCmd::Install { pkg } => {
                flatpak::install_flatpak(pkg).await?;
                println!("[reap][flatpak] Installed {}", pkg);
            }
            FlatpakCmd::Upgrade => {
                flatpak::upgrade();
            }
            FlatpakCmd::Audit { pkg } => {
                flatpak::print_flatpak_sandbox_info(pkg);
            }
        },
        Commands::Gpg { cmd } => match cmd {
            GpgCmd::Refresh => gpg::refresh_keys(),
            GpgCmd::Import { keyid } => {
                gpg::import_gpg_key_async(keyid).await?;
                println!("[reap] key imported successfully");
            }
            GpgCmd::Show { keyid } => {
                gpg::show_gpg_key_info(keyid);
                if let Some(trust) = gpg::get_trust_level(keyid) {
                    println!("[reap][gpg] Trust level for key {}: {}", keyid, trust);
                }
            }
            GpgCmd::Check { keyid } => {
                gpg::check_key(keyid).await;
            }
            GpgCmd::VerifyPkgbuild { path } => {
                let path = std::path::Path::new(path);
                let _ = gpg::verify_pkgbuild(path);
            }
            GpgCmd::SetKeyserver { url } => {
                utils::cli_set_keyserver(url);
            }
            GpgCmd::CheckKeyserver { url } => {
                utils::check_keyserver_async(url).await;
            }
        },
        Commands::Tap { cmd } => match cmd {
            TapCmd::Add { name, url } => {
                aur::add_tap(name, url);
            }
            TapCmd::List => {
                let taps = aur::get_taps();
                if taps.is_empty() {
                    println!("[reap] No tap repositories configured.");
                } else {
                    println!("[reap] Tap repositories:");
                    for (name, url) in taps {
                        println!("{}={}", name, url);
                    }
                }
            }
        },
        Commands::Completion { shell } => {
            utils::completion(shell);
        }
    }
    Ok(())
}
