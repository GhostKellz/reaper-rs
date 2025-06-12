use crate::aur;
use crate::aur::SearchResult;
use crate::backend;
use crate::config::ReapConfig;
use crate::flatpak;
use crate::pacman;
use crate::tui::LogPane;
use crate::utils::audit_pkgbuild;
use crate::{gpg, hooks};
use futures::FutureExt;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Aur,
    Pacman,
    Flatpak,
    ChaoticAur,
    Apt,
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Source::Aur => "[AUR]",
            Source::Pacman => "[PACMAN]",
            Source::Flatpak => "[FLATPAK]",
            Source::ChaoticAur => "[ChaoticAUR]",
            Source::Apt => "[APT]",
        }
    }
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

pub async fn parallel_install(
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
                if config.is_ignored(&pkg) {
                    if let Some(log) = log.as_ref() {
                        log.push(&format!("[reap] Skipping ignored package: {}", pkg));
                    } else {
                        println!("[reap] Skipping ignored package: {}", pkg);
                    }
                    return Ok::<(), Box<dyn std::error::Error + Send + Sync>>(());
                }
                if let Some(src) = detect_source(&pkg) {
                    if src == Source::Aur {
                        let pkgb = aur::get_pkgbuild_preview(&pkg);
                        audit_pkgbuild(&pkgb, None);
                    }
                }
                install_with_priority(&pkg, &config);
                if let Some(log) = log.as_ref() {
                    log.push(&format!("[reap] Installed {}", pkg));
                } else {
                    println!("[reap] Installed {}", pkg);
                }
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })
        })
        .collect();
    let results = join_all(tasks).await;
    for result in results {
        if let Err(e) = result {
            if let Some(log) = log.as_ref() {
                log.push(&format!("❌ Install failed: {:?}", e));
            } else {
                println!("❌ Install failed: {:?}", e);
            }
        }
    }
    if let Some(log) = log.as_ref() {
        log.push("[reap] All installs complete.");
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
                install_with_priority(&pkg, &config);
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

pub fn install_with_priority(pkg: &str, _config: &ReapConfig) {
    let priorities = vec![Source::Aur, Source::Pacman, Source::Flatpak];
    for src in priorities {
        match src {
            Source::Aur => {
                if aur::aur_search_results(pkg).iter().any(|r| r.name == pkg) {
                    aur::install(pkg);
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
        let output = std::process::Command::new("pacman")
            .arg("-Si")
            .arg(pkg)
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                return Some(Source::Pacman);
            }
        }
        let output = std::process::Command::new("flatpak")
            .arg("info")
            .arg(pkg)
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                return Some(Source::Flatpak);
            }
        }
        let output = std::process::Command::new("apt-cache")
            .arg("show")
            .arg(pkg)
            .output();
        if let Ok(out) = output {
            if out.status.success() {
                return Some(Source::Apt);
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
    for pkg in pkgs {
        println!("{} Installing {}...", "[reap]".cyan().bold(), pkg.bold());
        let info = aur::fetch_package_info(&pkg);
        if info.is_err() {
            eprintln!(
                "{} Package '{}' not found in AUR.",
                "[reap]".red().bold(),
                pkg.red()
            );
            continue;
        }
        let tmp_dir = std::env::temp_dir().join(format!("reap-aur-{}", pkg));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        if !aur::clone_repo(&pkg, &tmp_dir) {
            eprintln!(
                "{} Failed to clone repo for {}",
                "[reap]".red().bold(),
                pkg.red()
            );
            continue;
        }
        if !gpg::verify_pkgbuild(&tmp_dir) {
            eprintln!(
                "{} PKGBUILD verification failed for {}",
                "[reap]".red().bold(),
                pkg.red()
            );
            continue;
        }
        if let Err(e) = backend::build_and_install(&tmp_dir) {
            eprintln!(
                "{} Failed to build {}: {e}",
                "[reap]".red().bold(),
                pkg.red()
            );
            continue;
        }
        hooks::on_install(&pkg);
        println!(
            "{} Installed {}.",
            "[reap]".green().bold(),
            pkg.green().bold()
        );
    }
}

pub async fn handle_install_parallel(pkgs: Vec<String>, max_parallel: usize) {
    use owo_colors::OwoColorize;
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
            if let Err(e) = std::panic::AssertUnwindSafe(async {
                handle_install(vec![pkg.clone()]);
            })
            .catch_unwind()
            .await
            {
                pb.println(format!(
                    "{} {}: {:?}",
                    "❌".red().bold().to_string(),
                    pkg,
                    e
                ));
            } else {
                pb.println(format!("{} {}", "✔".green().bold().to_string(), pkg));
            }
            pb.inc(1);
            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        }));
    }
    let results = join_all(handles).await;
    for result in results {
        if let Err(e) = result {
            eprintln!("{} Failed: {e}", "❌".red().bold().to_string());
        }
    }
    pb.finish_with_message("All installs complete.");
    println!("{} All installs complete.", "✔".green().bold().to_string());
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

pub fn handle_rollback(pkg: String) {
    let backup = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("reaper/backups")
        .join(format!("{}.pkg.tar.zst", pkg));
    if !backup.exists() {
        eprintln!("[reap] No backup found for {}.", pkg);
        return;
    }
    let status = std::process::Command::new("sudo")
        .arg("pacman")
        .arg("-U")
        .arg(&backup)
        .status();
    if status.map(|s| s.success()).unwrap_or(false) {
        println!("[reap] Rolled back {}.", pkg);
        hooks::on_rollback(&pkg);
    } else {
        eprintln!("[reap] Rollback failed for {}.", pkg);
    }
}

pub fn handle_doctor() {
    use owo_colors::OwoColorize;
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
        println!(
            "{} {}",
            if *ok {
                "✔".green().bold().to_string()
            } else {
                "✗".red().bold().to_string()
            },
            label.bold()
        );
    }
    println!(
        "{} Config file: {}",
        if config_ok {
            "✔".green().bold().to_string()
        } else {
            "✗".red().bold().to_string()
        },
        config_path.display()
    );
    println!(
        "{} AUR network access",
        if aur_ok {
            "✔".green().bold().to_string()
        } else {
            "✗".red().bold().to_string()
        }
    );
}

pub fn handle_pin(_pkg: String) {
    println!("[reap] Pinning not yet implemented.");
}

pub fn handle_tui() {
    println!("[reap] TUI not yet implemented.");
}

pub fn handle_clean() {
    println!("[reap] Clean not yet implemented.");
}

pub fn handle_gpg_refresh() {
    crate::gpg::refresh_keys();
}
