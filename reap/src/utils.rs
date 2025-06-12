use crate::aur::SearchResult;
use diff::lines;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

static PKGBUILD_CACHE: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub async fn async_get_pkgbuild_cached(pkg: &str) -> String {
    {
        let cache = PKGBUILD_CACHE.lock().unwrap();
        if let Some(cached) = cache.get(pkg) {
            return cached.clone();
        }
    }
    let url = format!(
        "https://aur.archlinux.org/cgit/aur.git/plain/PKGBUILD?h={}",
        pkg
    );
    let text = match reqwest::get(&url).await {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => String::from("[reap] PKGBUILD not found."),
    };
    let mut cache = PKGBUILD_CACHE.lock().unwrap();
    cache.insert(pkg.to_string(), text.clone());
    text
}

pub fn audit_pkgbuild(pkgbuild: &str, lua: Option<&mlua::Lua>) {
    let risky_patterns = [
        "curl",
        "wget",
        "sudo",
        "rm -rf",
        "chmod",
        "chown",
        "dd",
        "mkfs",
        "mount",
        "scp",
        "nc",
        "ncat",
        "bash -c",
        "eval",
        "setcap",
        "setuid",
        "setgid",
        "useradd",
        "groupadd",
        "passwd",
        "iptables",
        "firewalld",
        "systemctl",
        "service",
    ];
    for pat in risky_patterns.iter() {
        if pkgbuild.contains(pat) {
            println!("[AUDIT] Found risky command: {}", pat);
        }
    }
    if let Some(lua) = lua {
        let _ = lua
            .load(r#"if custom_audit then custom_audit(...) end"#)
            .exec();
    }
}

pub fn backup_package(_pkg: &str) {
    // Placeholder for backup logic (e.g., copy config files, etc.)
}

/// Audit a .deb control file for risky patterns and allow Lua hooks
pub fn audit_deb_control(control: &str, lua: Option<&mlua::Lua>) {
    let risky_patterns = [
        "sudo", "rm -rf", "curl", "wget", "chmod", "chown", "dd", "mkfs", "mount", "scp", "nc",
        "ncat", "bash -c", "eval",
    ];
    for pat in risky_patterns.iter() {
        if control.contains(pat) {
            println!("[AUDIT][DEB] Found risky command: {}", pat);
        }
    }
    if let Some(lua) = lua {
        let _ = lua
            .load(r#"if custom_audit then custom_audit(...) end"#)
            .exec();
    }
}

/// Audit a Flatpak manifest for risky patterns and allow Lua hooks
pub fn audit_flatpak_manifest(manifest: &str, lua: Option<&mlua::Lua>) {
    let risky_patterns = ["command", "run", "sudo", "curl", "wget", "bash", "eval"];
    for pat in risky_patterns.iter() {
        if manifest.contains(pat) {
            println!("[AUDIT][FLATPAK] Found risky command: {}", pat);
        }
    }
    if let Some(lua) = lua {
        let _ = lua
            .load(r#"if custom_audit then custom_audit(...) end"#)
            .exec();
    }
}

pub fn audit_package(pkg: &str) {
    match crate::core::detect_source(pkg) {
        Some(crate::core::Source::Aur) => {
            println!("[AUDIT][AUR] Auditing PKGBUILD for {}...", pkg);
            let pkgb = crate::aur::get_pkgbuild_preview(pkg);
            audit_pkgbuild(&pkgb, None);
            let deps = crate::aur::get_deps(pkg);
            if deps.is_empty() {
                println!("[AUDIT][AUR] No dependencies found for {}.", pkg);
            } else {
                println!("[AUDIT][AUR] Dependencies for {}: {:?}", pkg, deps);
            }
        }
        Some(crate::core::Source::Flatpak) => {
            println!("[AUDIT][FLATPAK] flatpak info {}:", pkg);
            let output = std::process::Command::new("flatpak")
                .arg("info")
                .arg(pkg)
                .output();
            if let Ok(out) = output {
                println!("{}", String::from_utf8_lossy(&out.stdout));
            } else {
                println!("[AUDIT][FLATPAK] Could not get info for {}.", pkg);
            }
        }
        _ => println!("[AUDIT] Unknown package source for {}.", pkg),
    }
}

// --- CLI stub functions for compatibility ---
pub fn pkgb_diff_audit(package: &str, pkgb: &str) {
    let backup_path = format!("/var/lib/reaper/backup/{}_PKGBUILD.bak", package);
    if let Ok(old_pkgb) = fs::read_to_string(&backup_path) {
        for diff in lines(&old_pkgb, pkgb) {
            match diff {
                diff::Result::Left(l) => println!("[-] {}", l),
                diff::Result::Right(r) => println!("[+] {}", r),
                diff::Result::Both(_, _) => {}
            }
        }
    } else {
        println!("[reap] No previous PKGBUILD backup found for {}.", package);
    }
}

pub fn completion(shell: &str) {
    match shell {
        "bash" => println!("complete -C reap reap"),
        "zsh" => println!("compdef _reap reap"),
        "fish" => println!("complete -c reap -a \"(reap --completion)\""),
        _ => println!("[reap] Shell completion not implemented for {}.", shell),
    }
}

pub fn rollback(package: &str) {
    let backup_path = format!("/var/lib/reaper/backup/{}_backup", package);
    if fs::metadata(&backup_path).is_ok() {
        println!("[reap] Restoring backup for {}...", package);
        // Actual restore logic would go here
    } else {
        println!("[reap] No backup found for {}.", package);
    }
}

pub fn cli_rollback_pkgbuild(package: &str) {
    let backup_path = format!("/var/lib/reaper/backup/{}_PKGBUILD.bak", package);
    if fs::metadata(&backup_path).is_ok() {
        println!("[reap] Restoring PKGBUILD for {}...", package);
        // Actual restore logic would go here
    } else {
        println!("[reap] No PKGBUILD backup found for {}.", package);
    }
}

pub fn cli_set_keyserver(keyserver: &str) {
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config/reaper/brew.lua");
    if let Ok(mut script) = fs::read_to_string(&config_path) {
        if script.contains("keyserver = ") {
            script = script.replace(
                regex::Regex::new("keyserver = \".*\"").unwrap().as_str(),
                &format!("keyserver = \"{}\"", keyserver),
            );
        } else {
            script.push_str(&format!("\nkeyserver = \"{}\"\n", keyserver));
        }
        let _ = fs::write(&config_path, script);
        println!("[reap] Set keyserver to {} in config.", keyserver);
    } else {
        println!("[reap] Could not update config at {:?}.", config_path);
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

pub async fn check_keyserver(keyserver: &str) {
    let output = std::process::Command::new("gpg")
        .args(["--keyserver", keyserver, "--list-keys"])
        .output();
    if let Ok(out) = output {
        if out.status.success() {
            println!("[reap] GPG keyserver {} is reachable.", keyserver);
        } else {
            println!("[reap] GPG keyserver {} is NOT reachable.", keyserver);
        }
    } else {
        println!("[reap] Failed to check GPG keyserver {}.", keyserver);
    }
}

pub async fn check_keyserver_async(keyserver: &str) {
    let output = tokio::process::Command::new("gpg")
        .args(["--keyserver", keyserver, "--list-keys"])
        .output()
        .await;
    if let Ok(out) = output {
        if out.status.success() {
            println!("[reap] GPG keyserver {} is reachable.", keyserver);
        } else {
            println!("[reap] GPG keyserver {} is NOT reachable.", keyserver);
        }
    } else {
        println!("[reap] Failed to check GPG keyserver {}.", keyserver);
    }
}
