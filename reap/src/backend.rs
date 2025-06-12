#![allow(dead_code)]

use crate::aur::SearchResult;
use async_trait::async_trait;
use futures::FutureExt;
use std::error::Error;
use std::path::Path;
use std::process::Command;

#[async_trait]
pub trait Backend: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    async fn search<'a>(&'a self, query: &'a str) -> Vec<SearchResult>;
    async fn install<'a>(&'a self, package: &'a str);
    async fn upgrade<'a>(&'a self);
    async fn audit<'a>(&'a self, package: &'a str);
    async fn gpg_check<'a>(&'a self, package: &'a str);
}

pub fn build_and_install(pkgdir: &Path) -> Result<(), Box<dyn Error + Send + Sync>> {
    let status = Command::new("makepkg")
        .arg("-si")
        .arg("--noconfirm")
        .current_dir(pkgdir)
        .status()?;
    if !status.success() {
        return Err("makepkg failed".into());
    }
    Ok(())
}

pub struct AurBackend;
impl AurBackend {
    pub fn new() -> Self {
        AurBackend
    }
}

#[async_trait]
impl Backend for AurBackend {
    fn name(&self) -> &'static str {
        "AUR"
    }
    fn is_available(&self) -> bool {
        which::which("yay").is_ok() || which::which("paru").is_ok()
    }
    async fn search<'a>(&'a self, query: &'a str) -> Vec<SearchResult> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
            urlencoding::encode(query)
        );
        let resp = client.get(&url).send().await;
        if let Ok(resp) = resp {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                if let Some(results) = json.get("results").and_then(|r| r.as_array()) {
                    return results
                        .iter()
                        .filter_map(|item| {
                            Some(SearchResult {
                                name: item.get("Name")?.as_str()?.to_string(),
                                version: item
                                    .get("Version")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                description: item
                                    .get("Description")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string(),
                                source: crate::core::Source::Aur,
                            })
                        })
                        .collect();
                }
            }
        }
        Vec::new()
    }
    async fn install<'a>(&'a self, package: &'a str) {
        crate::aur::install(vec![package]).await;
    }
    async fn upgrade<'a>(&'a self) {
        crate::aur::upgrade_all().await;
    }
    async fn audit<'a>(&'a self, package: &'a str) {
        crate::utils::audit_package(package);
    }
    async fn gpg_check<'a>(&'a self, package: &'a str) {
        crate::gpg::check_key(package).await;
    }
}

pub struct PacmanBackend;
#[async_trait]
impl Backend for PacmanBackend {
    fn name(&self) -> &'static str {
        "Pacman"
    }
    fn is_available(&self) -> bool {
        std::process::Command::new("which")
            .arg("pacman")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    async fn search<'a>(&'a self, query: &'a str) -> Vec<SearchResult> {
        crate::core::unified_search(query)
            .now_or_never()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.source == crate::core::Source::Pacman)
            .collect()
    }
    async fn install<'a>(&'a self, package: &'a str) {
        crate::pacman::install(package);
    }
    async fn upgrade<'a>(&'a self) { /* handled in aur::upgrade for now */
    }
    async fn audit<'a>(&'a self, package: &'a str) {
        println!("[reap] Pacman audit for {} (not implemented)", package);
    }
    async fn gpg_check<'a>(&'a self, package: &'a str) {
        println!("[reap] Pacman GPG check for {} (not implemented)", package);
    }
}

pub struct FlatpakBackend;

impl FlatpakBackend {
    pub fn new() -> Self {
        FlatpakBackend
    }
}

#[async_trait]
impl Backend for FlatpakBackend {
    fn name(&self) -> &'static str {
        "flatpak"
    }
    fn is_available(&self) -> bool {
        true
    }
    async fn search<'a>(&'a self, query: &'a str) -> Vec<SearchResult> {
        crate::flatpak::search(query)
    }
    async fn install<'a>(&'a self, package: &'a str) {
        crate::flatpak::install_flatpak(package);
    }
    async fn upgrade<'a>(&'a self) {
        crate::flatpak::upgrade_flatpak();
    }
    async fn audit<'a>(&'a self, _package: &'a str) {
        todo!("Flatpak audit not yet implemented");
    }
    async fn gpg_check<'a>(&'a self, _package: &'a str) {
        todo!("Flatpak GPG check not yet implemented");
    }
}

pub struct AptBackend;
#[async_trait]
impl Backend for AptBackend {
    fn name(&self) -> &'static str {
        "Apt"
    }
    fn is_available(&self) -> bool {
        std::process::Command::new("which")
            .arg("apt")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    async fn search<'a>(&'a self, query: &'a str) -> Vec<SearchResult> {
        let output = std::process::Command::new("apt-cache")
            .arg("search")
            .arg(query)
            .output();
        let mut results = Vec::new();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let mut parts = line.splitn(2, ' ');
                if let Some(name) = parts.next() {
                    let desc = parts.next().unwrap_or("").trim().to_string();
                    results.push(SearchResult {
                        name: name.to_string(),
                        version: String::from("?"),
                        description: desc,
                        source: crate::core::Source::Pacman, // TODO: add Debian source
                    });
                }
            }
        }
        results
    }
    async fn install<'a>(&'a self, package: &'a str) {
        let _ = std::process::Command::new("sudo")
            .arg("apt")
            .arg("install")
            .arg("-y")
            .arg(package)
            .status();
    }
    async fn upgrade<'a>(&'a self) {
        let _ = std::process::Command::new("sudo")
            .arg("apt")
            .arg("update")
            .status();
        let _ = std::process::Command::new("sudo")
            .arg("apt")
            .arg("upgrade")
            .arg("-y")
            .status();
    }
    async fn audit<'a>(&'a self, package: &'a str) {
        println!("[reap] Apt audit for {} (not implemented)", package);
    }
    async fn gpg_check<'a>(&'a self, package: &'a str) {
        println!("[reap] Apt GPG check for {} (not implemented)", package);
    }
}

pub enum BackendImpl {
    Aur(AurBackend),
    Flatpak(FlatpakBackend),
    Pacman(PacmanBackend),
    Apt(AptBackend),
}

impl BackendImpl {
    pub async fn search(&self, query: &str) -> Vec<SearchResult> {
        match self {
            BackendImpl::Aur(b) => b.search(query).await,
            BackendImpl::Flatpak(b) => b.search(query).await,
            BackendImpl::Pacman(b) => b.search(query).await,
            BackendImpl::Apt(b) => b.search(query).await,
        }
    }
    pub async fn install(&self, pkg: &str) {
        match self {
            BackendImpl::Aur(b) => b.install(pkg).await,
            BackendImpl::Flatpak(b) => b.install(pkg).await,
            BackendImpl::Pacman(b) => b.install(pkg).await,
            BackendImpl::Apt(b) => b.install(pkg).await,
        }
    }
    pub async fn upgrade(&self) {
        match self {
            BackendImpl::Aur(b) => b.upgrade().await,
            BackendImpl::Flatpak(b) => b.upgrade().await,
            BackendImpl::Pacman(b) => b.upgrade().await,
            BackendImpl::Apt(b) => b.upgrade().await,
        }
    }
    pub async fn audit(&self, pkg: &str) {
        match self {
            BackendImpl::Aur(b) => b.audit(pkg).await,
            BackendImpl::Flatpak(b) => b.audit(pkg).await,
            BackendImpl::Pacman(b) => b.audit(pkg).await,
            BackendImpl::Apt(b) => b.audit(pkg).await,
        }
    }
    pub async fn gpg_check(&self, pkg: &str) {
        match self {
            BackendImpl::Aur(b) => b.gpg_check(pkg).await,
            BackendImpl::Flatpak(b) => b.gpg_check(pkg).await,
            BackendImpl::Pacman(b) => b.gpg_check(pkg).await,
            BackendImpl::Apt(b) => b.gpg_check(pkg).await,
        }
    }
}
