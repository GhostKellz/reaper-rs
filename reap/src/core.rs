use crate::aur;
use crate::pacman;
use crate::flatpak;
use crate::config::ReapConfig;
use crate::utils::audit_pkgbuild;
use std::sync::Arc;
use tokio::task;
use futures::future::join_all;

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

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: Source,
}

pub async fn unified_search(query: &str) -> Vec<SearchResult> {
    let aur = task::spawn_blocking({
        let q = query.to_string();
        move || {
            aur::aur_search_results(&q)
                .into_iter()
                .map(|r| SearchResult {
                    name: r.name,
                    version: r.version,
                    description: r.description.unwrap_or_default(),
                    source: Source::Aur,
                })
                .collect::<Vec<_>>()
        }
    });
    let pacman = task::spawn_blocking({
        let q = query.to_string();
        move || {
            let output = std::process::Command::new("pacman")
                .arg("-Ss")
                .arg(&q)
                .output();
            let mut results = Vec::new();
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let mut last_name = String::new();
                for line in stdout.lines() {
                    if line.starts_with("core/") || line.starts_with("extra/") || line.starts_with("community/") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let name_ver: Vec<&str> = parts[0].split('/').collect();
                            if name_ver.len() == 2 {
                                last_name = name_ver[1].to_string();
                                let version = parts[1].to_string();
                                results.push(SearchResult {
                                    name: last_name.clone(),
                                    version,
                                    description: String::new(),
                                    source: Source::Pacman,
                                });
                            }
                        }
                    } else if !last_name.is_empty() && !line.trim().is_empty() {
                        if let Some(last) = results.last_mut() {
                            last.description = line.trim().to_string();
                        }
                    }
                }
            }
            results
        }
    });
    let flatpak = task::spawn_blocking({
        let q = query.to_string();
        move || {
            let output = std::process::Command::new("flatpak")
                .arg("search")
                .arg(&q)
                .output();
            let mut results = Vec::new();
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                for line in stdout.lines().skip(1) {
                    let cols: Vec<&str> = line.split_whitespace().collect();
                    if cols.len() >= 2 {
                        let name = cols[0].to_string();
                        let version = cols.get(1).unwrap_or(&"").to_string();
                        let description = cols.get(2..).map(|c| c.join(" ")).unwrap_or_default();
                        results.push(SearchResult {
                            name,
                            version,
                            description,
                            source: Source::Flatpak,
                        });
                    }
                }
            }
            results
        }
    });
    let (aur, pacman, flatpak) = tokio::join!(aur, pacman, flatpak);
    let mut results = Vec::new();
    results.extend(aur.unwrap_or_default());
    results.extend(pacman.unwrap_or_default());
    results.extend(flatpak.unwrap_or_default());
    results
}

pub async fn parallel_install(pkgs: Vec<String>, config: Arc<ReapConfig>) {
    let tasks: Vec<_> = pkgs.into_iter().map(|pkg| {
        let config = config.clone();
        task::spawn(async move {
            if config.is_ignored(&pkg) {
                println!("[reap] Skipping ignored package: {}", pkg);
                return;
            }
            if let Some(src) = detect_source(&pkg) {
                if src == Source::Aur {
                    let pkgb = aur::get_pkgbuild_preview(&pkg);
                    audit_pkgbuild(&pkgb, None);
                }
            }
            install_with_priority(&pkg, &config); 
        })
    }).collect();
    let _ = join_all(tasks).await;
    println!("[reap] All installs complete.");
}

pub async fn parallel_upgrade(pkgs: Vec<String>, config: Arc<ReapConfig>) {
    let tasks: Vec<_> = pkgs.into_iter().map(|pkg| {
        let config = config.clone();
        task::spawn(async move {
            if config.is_ignored(&pkg) {
                println!("[reap] Skipping ignored package: {}", pkg);
                return;
            }
            install_with_priority(&pkg, &config);
        })
    }).collect();
    let _ = join_all(tasks).await;
    println!("[reap] All upgrades complete.");
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
        println!("{} {} {} - {}", r.source.label(), r.name, r.version, r.description);
    }
}

