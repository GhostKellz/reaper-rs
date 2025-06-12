use crate::utils;
use futures::future::join_all;
use once_cell::sync::Lazy;
use owo_colors::OwoColorize;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::sync::mpsc;

static TAP_REPOS: Lazy<Mutex<HashMap<String, String>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("reap/taps.json");
    if let Ok(data) = fs::read_to_string(&config_path) {
        if let Ok(json) = serde_json::from_str::<HashMap<String, String>>(&data) {
            map = json;
        }
    }
    Mutex::new(map)
});

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: crate::core::Source,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AurResult {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
    pub version: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Maintainer")]
    pub maintainer: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "URL")]
    pub url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AurResponse {
    pub results: Vec<AurResult>,
}

pub struct AurInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

pub fn fetch_package_info(pkg: &str) -> Result<AurInfo, Box<dyn Error + Send + Sync>> {
    let url = format!("https://aur.archlinux.org/rpc/?v=5&type=info&arg[]={}", pkg);
    let client = Client::new();
    let resp = client.get(&url).send()?;
    let aur_resp: AurResponse = resp.json()?;
    if let Some(r) = aur_resp.results.into_iter().next() {
        Ok(AurInfo {
            name: r.name,
            version: r.version,
            description: r.description,
        })
    } else {
        Err("Package not found".into())
    }
}

pub fn clone_repo(_pkg: &str, _dest: &std::path::Path) -> bool {
    // TODO: Implement real clone logic
    true
}

pub async fn search(query: &str) -> Result<Vec<SearchResult>, Box<dyn Error + Send + Sync>> {
    let url = format!(
        "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
        query
    );
    let client = reqwest::Client::new();
    let resp = client.get(&url).send().await?;
    let aur_resp: AurResponse = resp.json().await?;
    Ok(aur_resp
        .results
        .into_iter()
        .map(|r| SearchResult {
            name: r.name,
            version: r.version,
            description: r.description.unwrap_or_default(),
            source: crate::core::Source::Aur,
        })
        .collect())
}

pub fn aur_search_results(query: &str) -> Vec<AurResult> {
    let url = format!(
        "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
        query
    );
    if let Ok(resp) = reqwest::blocking::get(&url) {
        if let Ok(json) = resp.json::<AurResponse>() {
            return json.results;
        }
    }
    vec![]
}

// fn prompt_confirm(msg: &str) -> bool {
//     use std::io::{self, Write};
//     print!("{} [y/N]: ", msg);
//     let _ = io::stdout().flush();
//     let mut input = String::new();
//     if io::stdin().read_line(&mut input).is_ok() {
//         matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
//     } else {
//         false
//     }
// }

pub async fn install(pkgs: Vec<&str>) {
    let yay = which::which("yay").is_ok();
    let bin = if yay { "yay" } else { "pacman" };
    println!("[reap] Installing packages: {:?} ({} -S)...", pkgs, bin);
    let mut tasks = Vec::new();
    for &package in &pkgs {
        let bin = bin.to_string();
        let pkg = package.to_string();
        tasks.push(tokio::spawn(async move {
            let deps = get_deps(&pkg);
            if !deps.is_empty() {
                eprintln!("[reap] Dependencies for {}: {:?}", pkg.yellow(), deps);
                for dep in &deps {
                    if !crate::pacman::is_installed(dep) {
                        println!("[reap] Installing missing dependency: {}", dep.yellow());
                        let status = std::process::Command::new(&bin)
                            .arg("-S")
                            .arg(dep)
                            .status()
                            .expect("Failed to run -S <dep>");
                        if status.success() {
                            println!("[reap] Installed dependency: {}", dep.green());
                        } else {
                            eprintln!("[reap] Failed to install dependency: {}", dep.red());
                        }
                    } else {
                        println!("[reap] Dependency already installed: {}", dep.green());
                    }
                }
            } else {
                println!("[reap] No dependencies found for {}.", pkg);
            }
            let status = std::process::Command::new(&bin)
                .arg("-S")
                .arg(&pkg)
                .status()
                .expect("Failed to run -S <pkg>");
            (pkg, status.success())
        }));
    }
    let results = join_all(tasks).await;
    for res in results {
        match res {
            Ok((pkg, true)) => println!("[reap] Installed {}.", pkg.green()),
            Ok((pkg, false)) => eprintln!("[reap] Install failed for {}.", pkg.red()),
            Err(e) => eprintln!("[reap] Task join error: {}", e),
        }
    }
}

pub fn uninstall(package: &str) {
    let yay = which::which("yay").is_ok();
    let bin = if yay { "yay" } else { "pacman" };
    println!("[reap] Uninstalling {} ({} -R)...", package.yellow(), bin);
    let status = Command::new(bin)
        .arg("-R")
        .arg(package)
        .status()
        .expect("Failed to run -R <pkg>");
    if status.success() {
        println!("[reap] Uninstalled {}.", package.green());
    } else {
        eprintln!("[reap] Uninstall failed for {}.", package.red());
    }
}

pub fn get_pkgbuild_preview(pkg: &str) -> String {
    let url = format!(
        "https://aur.archlinux.org/cgit/aur.git/plain/PKGBUILD?h={}",
        pkg
    );
    if let Ok(resp) = reqwest::blocking::get(&url) {
        if let Ok(text) = resp.text() {
            return text;
        }
    }
    String::from("[reap] PKGBUILD not found.")
}

pub fn get_deps(pkg: &str) -> Vec<String> {
    let pkgb = get_pkgbuild_preview(pkg);
    let mut deps = Vec::new();
    let mut in_dep = false;
    let mut dep_buf = String::new();
    for line in pkgb.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("depends=") {
            in_dep = true;
            dep_buf.push_str(trimmed.split_once('=').map(|x| x.1).unwrap_or("").trim());
            if trimmed.ends_with(')') {
                in_dep = false;
            }
        } else if in_dep {
            dep_buf.push_str(trimmed);
            if trimmed.ends_with(')') {
                in_dep = false;
            }
        }
        if !in_dep && !dep_buf.is_empty() {
            let dep_line = dep_buf.trim_matches(&['(', ')', '"', '\'', ' '] as &[_]);
            deps.extend(
                dep_line
                    .split_whitespace()
                    .map(|s| s.trim_matches(&['"', '\'', ' '] as &[_]))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
            dep_buf.clear();
        }
    }
    deps
}

pub async fn upgrade() {
    println!("[reap] Upgrading system packages...");
    let _ = Command::new("sudo").arg("pacman").arg("-Syu").status();
    let output = Command::new("pacman").arg("-Qm").output();
    if let Ok(out) = output {
        let pkgs = String::from_utf8_lossy(&out.stdout);
        let config = crate::config::ReapConfig::load();
        let (tx, rx) = mpsc::channel();
        let mut _count = 0;
        let mut tasks = Vec::new();
        for line in pkgs.lines() {
            let pkg = line.split_whitespace().next().unwrap_or("").to_string();
            if !pkg.is_empty() && !config.is_ignored(&pkg) {
                let tx = tx.clone();
                _count += 1;
                utils::backup_package(&pkg);
                tasks.push(tokio::spawn(async move {
                    crate::aur::install(vec![pkg.as_str()]).await;
                    let _ = tx.send(pkg);
                }));
            }
        }
        drop(tx);
        let _ = join_all(tasks).await;
        for pkg in rx {
            println!("[reap] Finished upgrade for {}", pkg);
        }
    }
}

pub fn add_tap(name: &str, url: &str) {
    let mut taps = TAP_REPOS.lock().unwrap();
    taps.insert(name.to_string(), url.to_string());
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("reap/taps.json");
    let _ = fs::create_dir_all(config_path.parent().unwrap());
    let _ = fs::write(&config_path, serde_json::to_string_pretty(&*taps).unwrap());
    println!("[reap] Added tap repo: {} -> {}", name, url);
}

pub fn get_taps() -> HashMap<String, String> {
    TAP_REPOS.lock().unwrap().clone()
}

pub async fn sync_db() {
    println!("[reap] Syncing package database (yay -Sy)...");
    let status = Command::new("yay")
        .arg("-Sy")
        .status()
        .expect("Failed to run yay -Sy");
    if status.success() {
        println!("[reap] Database sync complete.");
    } else {
        eprintln!("[reap] Database sync failed.");
    }
}

pub async fn upgrade_all() {
    println!("[reap] Upgrading all packages (yay -Syu)...");
    let status = Command::new("yay")
        .arg("-Syu")
        .status()
        .expect("Failed to run yay -Syu");
    if status.success() {
        println!("[reap] System upgrade complete.");
    } else {
        eprintln!("[reap] System upgrade failed.");
    }
}

pub fn install_local(path: &str) {
    use std::path::Path;
    let file = Path::new(path);
    if !file.exists() {
        eprintln!("[reap] Local package file does not exist: {}", path.red());
        return;
    }
    let ext = file.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !(ext == "zst" || path.ends_with(".pkg.tar.zst")) {
        println!(
            "[reap] Local package file must be a .zst or .pkg.tar.zst: {}",
            path.yellow()
        );
        return;
    }
    println!(
        "[reap] Installing local package from {} (sudo pacman -U)...",
        path.yellow()
    );
    let status = Command::new("sudo")
        .arg("pacman")
        .arg("-U")
        .arg(path)
        .status()
        .expect("Failed to run sudo pacman -U <file>");
    if status.success() {
        println!("[reap] Local install complete: {}.", path.green());
    } else {
        eprintln!("[reap] Local install failed: {}.", path.red());
    }
}
