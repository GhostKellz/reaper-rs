use crate::utils;
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
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

fn prompt_confirm(msg: &str) -> bool {
    use std::io::{self, Write};
    print!("{} [y/N]: ", msg);
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    } else {
        false
    }
}

pub fn install(package: &str) {
    if !prompt_confirm(&format!("Install AUR package {}?", package)) {
        println!("[reap] Skipped install for {}", package);
        return;
    }
    let aur_url = format!("https://aur.archlinux.org/{}.git", package);
    let tmp_dir = std::env::temp_dir().join(format!("reap-aur-{}", package));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    let status = std::process::Command::new("git")
        .arg("clone")
        .arg(&aur_url)
        .arg(&tmp_dir)
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("[reap] Failed to clone AUR repo for {}", package);
        return;
    }
    let status = std::process::Command::new("makepkg")
        .current_dir(&tmp_dir)
        .arg("-si")
        .arg("--noconfirm")
        .status();
    if !status.map(|s| s.success()).unwrap_or(false) {
        eprintln!("[reap] makepkg failed for {}", package);
    }
    let _ = std::fs::remove_dir_all(&tmp_dir);
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

pub fn upgrade() {
    println!("[reap] Upgrading system packages...");
    let _ = std::process::Command::new("sudo")
        .arg("pacman")
        .arg("-Syu")
        .status();
    let output = std::process::Command::new("pacman").arg("-Qm").output();
    if let Ok(out) = output {
        let pkgs = String::from_utf8_lossy(&out.stdout);
        let config = crate::config::ReapConfig::load(); // <-- FIXED HERE
        let (tx, rx) = mpsc::channel();
        let mut _count = 0;
        for line in pkgs.lines() {
            let pkg = line.split_whitespace().next().unwrap_or("").to_string();
            if !pkg.is_empty() && !config.is_ignored(&pkg) {
                let tx = tx.clone();
                _count += 1;
                utils::backup_package(&pkg);
                install(&pkg);
                let _ = tx.send(pkg);
            }
        }
        drop(tx);
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
