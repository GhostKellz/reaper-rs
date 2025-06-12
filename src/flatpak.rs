use crate::aur::SearchResult;
use crate::core::Source;
use std::process::Command;

// Flatpak integration (scaffold)
pub fn search(query: &str) -> Vec<SearchResult> {
    let output = Command::new("flatpak").arg("search").arg(query).output();
    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut results = Vec::new();
            for line in stdout.lines() {
                // Flatpak search output is typically: <app-id> <summary>
                // Some versions may have columns: Name	App ID	Version	Branch	Remotes	Description
                // We'll try to parse tab-separated, else fallback to space split
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() >= 2 {
                    let name = fields.get(1).unwrap_or(&"").to_string();
                    let version = fields.get(2).unwrap_or(&"").to_string();
                    let description = fields.get(5).unwrap_or(&"").to_string();
                    results.push(SearchResult {
                        name,
                        version,
                        description,
                        source: Source::Flatpak,
                    });
                } else {
                    // Fallback: try space split
                    let mut parts = line.splitn(2, ' ');
                    let name = parts.next().unwrap_or("").to_string();
                    let description = parts.next().unwrap_or("").to_string();
                    if !name.is_empty() && !description.is_empty() {
                        results.push(SearchResult {
                            name,
                            version: String::new(),
                            description,
                            source: Source::Flatpak,
                        });
                    }
                }
            }
            return results;
        }
    }
    vec![]
}

// Example usage: call flatpak::search from CLI or TUI for Flatpak search

pub fn install(pkg: &str) {
    let _ = Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg(pkg)
        .status();
}

pub async fn install_flatpak(pkg: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[reap][flatpak] Installing {}...", pkg);
    let status = Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg(pkg)
        .status()?;
    if status.success() {
        println!("[reap][flatpak] Installed {}!", pkg);
        Ok(())
    } else {
        Err(format!("[reap][flatpak] install failed for {}", pkg).into())
    }
}

pub fn upgrade() {
    println!("[reap] flatpak :: Upgrading all flatpak packages...");
    let status = Command::new("flatpak").arg("update").arg("-y").status();
    match status {
        Ok(s) if s.success() => println!("[reap] flatpak :: All packages upgraded!"),
        Ok(_) | Err(_) => println!("[reap] flatpak :: upgrade failed."),
    }
}

pub async fn upgrade_flatpak() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("[reap][flatpak] Upgrading all flatpak packages...");
    let status = Command::new("flatpak").arg("update").arg("-y").status()?;
    if status.success() {
        println!("[reap][flatpak] All packages upgraded!");
        Ok(())
    } else {
        Err("[reap][flatpak] upgrade failed.".into())
    }
}
// Example usage: call flatpak::upgrade from CLI or TUI for Flatpak upgrade

pub fn print_flatpak_sandbox_info(pkg: &str) {
    let output = Command::new("flatpak").arg("info").arg(pkg).output();
    if let Ok(out) = output {
        let info = String::from_utf8_lossy(&out.stdout);
        if info.contains("sandbox: none") {
            println!(
                "[reap] flatpak :: Warning: Flatpak {} is NOT sandboxed!",
                pkg
            );
        } else {
            println!("[reap] flatpak :: sandbox info for {}:\n{}", pkg, info);
        }
    }
}
// Example usage: call print_flatpak_sandbox_info in TUI/CLI details
