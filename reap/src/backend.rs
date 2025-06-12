use std::path::Path;
use std::process::Command;
use std::error::Error;
use futures::FutureExt;
use crate::aur::SearchResult;

pub trait Backend {
    fn name(&self) -> &'static str;
    fn is_available(&self) -> bool;
    fn search(&self, query: &str) -> Vec<SearchResult>;
    fn install(&self, package: &str);
    fn upgrade(&self);
    fn audit(&self, package: &str);
    fn gpg_check(&self, package: &str);
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
impl Backend for AurBackend {
    fn name(&self) -> &'static str { "AUR" }
    fn is_available(&self) -> bool { true }
    fn search(&self, query: &str) -> Vec<SearchResult> {
        crate::aur::aur_search_results(query)
            .into_iter()
            .map(|r| SearchResult {
                name: r.name,
                version: r.version,
                description: r.description.unwrap_or_default(),
                source: crate::core::Source::Aur,
            })
            .collect()
    }
    fn install(&self, package: &str) { crate::aur::install(package); }
    fn upgrade(&self) { crate::aur::upgrade(); }
    fn audit(&self, package: &str) {
        let pkgb = crate::aur::get_pkgbuild_preview(package);
        crate::utils::audit_pkgbuild(&pkgb, None);
    }
    fn gpg_check(&self, _package: &str) { /* AUR GPG check is handled in gpg.rs */ }
}

pub struct PacmanBackend;
impl Backend for PacmanBackend {
    fn name(&self) -> &'static str { "Pacman" }
    fn is_available(&self) -> bool {
        std::process::Command::new("which").arg("pacman").output().map(|o| o.status.success()).unwrap_or(false)
    }
    fn search(&self, query: &str) -> Vec<SearchResult> {
        crate::core::unified_search(query)
            .now_or_never()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.source == crate::core::Source::Pacman)
            .collect()
    }
    fn install(&self, package: &str) { crate::pacman::install(package); }
    fn upgrade(&self) { /* handled in aur::upgrade for now */ }
    fn audit(&self, package: &str) {
        println!("[reap] Pacman audit for {} (not implemented)", package);
    }
    fn gpg_check(&self, package: &str) {
        println!("[reap] Pacman GPG check for {} (not implemented)", package);
    }
}

pub struct FlatpakBackend;
impl Backend for FlatpakBackend {
    fn name(&self) -> &'static str { "Flatpak" }
    fn is_available(&self) -> bool {
        std::process::Command::new("which").arg("flatpak").output().map(|o| o.status.success()).unwrap_or(false)
    }
    fn search(&self, query: &str) -> Vec<SearchResult> {
        crate::core::unified_search(query)
            .now_or_never()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.source == crate::core::Source::Flatpak)
            .collect()
    }
    fn install(&self, package: &str) { crate::flatpak::install(package); }
    fn upgrade(&self) { crate::flatpak::upgrade(); }
    fn audit(&self, package: &str) {
        println!("[reap] Flatpak audit for {} (not implemented)", package);
    }
    fn gpg_check(&self, package: &str) {
        println!("[reap] Flatpak GPG check for {} (not implemented)", package);
    }
}

pub struct AptBackend;
impl Backend for AptBackend {
    fn name(&self) -> &'static str { "Apt" }
    fn is_available(&self) -> bool {
        std::process::Command::new("which").arg("apt").output().map(|o| o.status.success()).unwrap_or(false)
    }
    fn search(&self, query: &str) -> Vec<SearchResult> {
        let output = std::process::Command::new("apt-cache").arg("search").arg(query).output();
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
    fn install(&self, package: &str) {
        let _ = std::process::Command::new("sudo").arg("apt").arg("install").arg("-y").arg(package).status();
    }
    fn upgrade(&self) {
        let _ = std::process::Command::new("sudo").arg("apt").arg("update").status();
        let _ = std::process::Command::new("sudo").arg("apt").arg("upgrade").arg("-y").status();
    }
    fn audit(&self, package: &str) {
        println!("[reap] Apt audit for {} (not implemented)", package);
    }
    fn gpg_check(&self, package: &str) {
        println!("[reap] Apt GPG check for {} (not implemented)", package);
    }
}

