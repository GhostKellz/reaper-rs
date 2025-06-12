use crate::aur::SearchResult;
use futures::FutureExt;
use std::error::Error;
use std::path::Path;
use std::process::Command;

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
    fn name(&self) -> &'static str {
        "AUR"
    }
    fn is_available(&self) -> bool {
        which::which("yay").is_ok() || which::which("paru").is_ok()
    }
    fn search(&self, query: &str) -> Vec<SearchResult> {
        // Use aurweb RPC API (blocking for now)
        let url = format!(
            "https://aur.archlinux.org/rpc/?v=5&type=search&arg={}",
            query
        );
        let client = reqwest::blocking::Client::new();
        let resp = client.get(&url).send();
        if let Ok(resp) = resp {
            if let Ok(json) = resp.json::<serde_json::Value>() {
                if let Some(results) = json.get("results").and_then(|v| v.as_array()) {
                    return results
                        .iter()
                        .filter_map(|r| {
                            Some(SearchResult {
                                name: r.get("Name")?.as_str()?.to_string(),
                                version: r.get("Version")?.as_str()?.to_string(),
                                description: r
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
        vec![]
    }
    fn install(&self, package: &str) {
        let yay = which::which("yay").is_ok();
        let paru = which::which("paru").is_ok();
        let bin = if yay {
            "yay"
        } else if paru {
            "paru"
        } else {
            "yay"
        };
        let _ = Command::new(bin).arg("-S").arg(package).status();
    }
    fn upgrade(&self) {
        // Not implemented for now
    }
    fn audit(&self, package: &str) {
        // PKGBUILD audit: check for URL, license, maintainer anomalies
        let pkgb = crate::aur::get_pkgbuild_preview(package);
        if !pkgb.contains("url=") {
            println!("[audit] PKGBUILD missing url field for {}", package);
        }
        if !pkgb.contains("license=") {
            println!("[audit] PKGBUILD missing license field for {}", package);
        }
        if !pkgb.contains("maintainer=") {
            println!("[audit] PKGBUILD missing maintainer field for {}", package);
        }
        crate::utils::audit_pkgbuild(&pkgb, None);
    }
    fn gpg_check(&self, package: &str) {
        // Use pacman -Si or AUR RPC to get PGP key info
        let output = std::process::Command::new("pacman")
            .arg("-Si")
            .arg(package)
            .output();
        let mut keyid = None;
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.contains("PGP Signature") {
                    if let Some(start) = line.find(':') {
                        keyid = Some(line[start + 1..].trim().to_string());
                    }
                }
            }
        }
        if let Some(key) = keyid {
            let _ = std::process::Command::new("gpg")
                .arg("--recv-keys")
                .arg(&key)
                .status();
            let sig_path = format!("/var/cache/pacman/pkg/{}.sig", package);
            let pkg_path = format!("/var/cache/pacman/pkg/{}.pkg.tar.zst", package);
            let _ = std::process::Command::new("gpg")
                .arg("--verify")
                .arg(&sig_path)
                .arg(&pkg_path)
                .status();
        } else {
            println!("[gpg] No PGP key found for {}", package);
        }
    }
}

pub struct PacmanBackend;
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
    fn search(&self, query: &str) -> Vec<SearchResult> {
        crate::core::unified_search(query)
            .now_or_never()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.source == crate::core::Source::Pacman)
            .collect()
    }
    fn install(&self, package: &str) {
        crate::pacman::install(package);
    }
    fn upgrade(&self) { /* handled in aur::upgrade for now */
    }
    fn audit(&self, package: &str) {
        println!("[reap] Pacman audit for {} (not implemented)", package);
    }
    fn gpg_check(&self, package: &str) {
        println!("[reap] Pacman GPG check for {} (not implemented)", package);
    }
}

pub struct FlatpakBackend;
impl Backend for FlatpakBackend {
    fn name(&self) -> &'static str {
        "Flatpak"
    }
    fn is_available(&self) -> bool {
        std::process::Command::new("which")
            .arg("flatpak")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    fn search(&self, query: &str) -> Vec<SearchResult> {
        crate::core::unified_search(query)
            .now_or_never()
            .unwrap_or_default()
            .into_iter()
            .filter(|r| r.source == crate::core::Source::Flatpak)
            .collect()
    }
    fn install(&self, package: &str) {
        crate::flatpak::install(package);
    }
    fn upgrade(&self) {
        crate::flatpak::upgrade();
    }
    fn audit(&self, package: &str) {
        println!("[reap] Flatpak audit for {} (not implemented)", package);
    }
    fn gpg_check(&self, package: &str) {
        println!("[reap] Flatpak GPG check for {} (not implemented)", package);
    }
}

pub struct AptBackend;
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
    fn search(&self, query: &str) -> Vec<SearchResult> {
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
    fn install(&self, package: &str) {
        let _ = std::process::Command::new("sudo")
            .arg("apt")
            .arg("install")
            .arg("-y")
            .arg(package)
            .status();
    }
    fn upgrade(&self) {
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
    fn audit(&self, package: &str) {
        println!("[reap] Apt audit for {} (not implemented)", package);
    }
    fn gpg_check(&self, package: &str) {
        println!("[reap] Apt GPG check for {} (not implemented)", package);
    }
}
