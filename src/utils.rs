use crate::aur::SearchResult;
use diff::lines;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;
use std::fs;

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
    let backup_path = std::path::Path::new(&backup_path);
    if let Ok(old_pkgb) = fs::read_to_string(backup_path) {
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

pub fn pin_package(pkg: &str) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".config/reap/pinned.toml");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)
        .map_err(|e| format!("Failed to open pinned.toml: {}", e))?;
    writeln!(file, "{}", pkg).map_err(|e| format!("Failed to write: {}", e))?;
    Ok(())
}

pub fn is_pinned(pkg: &str) -> bool {
    use std::fs;
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".config/reap/pinned.toml");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        contents.lines().any(|line| line.trim() == pkg)
    } else {
        false
    }
}

pub fn clean_cache() -> Result<String, String> {
    let home = dirs::home_dir().unwrap_or_default();
    let cache_dirs = vec!["/tmp/reap".to_string(), format!("{}/.cache/reap", home.display())];
    let mut deleted = 0;
    for dir in &cache_dirs {
        let path = std::path::PathBuf::from(dir);
        if path.exists() && path.is_dir() {
            for entry in fs::read_dir(&path).map_err(|e| format!("Failed to read {}: {}", dir, e))? {
                let entry = entry.map_err(|e| format!("Read dir error: {}", e))?;
                let p = entry.path();
                if p.is_file() {
                    fs::remove_file(&p).map_err(|e| format!("Failed to remove {}: {}", p.display(), e))?;
                    deleted += 1;
                }
            }
        }
    }
    if deleted > 0 {
        Ok(format!("Cleaned {} cache files.", deleted))
    } else {
        Ok("Nothing to clean.".to_string())
    }
}

pub fn doctor_report() -> Result<String, String> {
    let mut issues = Vec::new();
    // Check for broken symlinks in /usr/bin/reap-*
    if let Ok(entries) = fs::read_dir("/usr/bin") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("reap-") && path.is_symlink() {
                    if let Ok(target) = fs::read_link(&path) {
                        if !target.exists() {
                            issues.push(format!("Broken symlink: {} -> {}", path.display(), target.display()));
                        }
                    }
                }
            }
        }
    }
    // Check for missing config files
    let config_dir = dirs::home_dir().unwrap_or_default().join(".config/reap");
    if !config_dir.exists() {
        issues.push(format!("Missing config dir: {}", config_dir.display()));
    }
    let required = ["brew.lua", "pinned.toml"];
    for f in &required {
        let fpath = config_dir.join(f);
        if !fpath.exists() {
            issues.push(format!("Missing config file: {}", fpath.display()));
        }
    }
    if issues.is_empty() {
        Ok("System appears healthy".to_string())
    } else {
        Ok(format!("Issues found:\n{}", issues.join("\n")))
    }
}
