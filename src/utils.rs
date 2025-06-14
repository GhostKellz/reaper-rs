use anyhow::Result;
use diff::lines;
use std::fs;
use std::os::unix::process::ExitStatusExt;
use toml::Value;

#[cfg(feature = "cache")]
use once_cell::sync::Lazy;

#[cfg(feature = "cache")]
use serde_json;
#[cfg(feature = "cache")]
use std::fs::{self, create_dir_all, read_to_string, write};

#[cfg(feature = "cache")]
static PKGBUILD_CACHE: Lazy<std::sync::Mutex<std::collections::HashMap<String, String>>> =
    Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

#[cfg(feature = "cache")]
static SEARCH_CACHE: Lazy<std::sync::Mutex<std::collections::HashMap<String, Vec<SearchResult>>>> =
    Lazy::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

#[cfg(feature = "cache")]
static PKGBUILD_CACHE_DIR: Lazy<std::path::PathBuf> = Lazy::new(|| {
    dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("reap/pkgbuilds")
});
#[cfg(feature = "cache")]
static SEARCH_CACHE_DIR: Lazy<std::path::PathBuf> = Lazy::new(|| {
    dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("reap/search")
});

/// Caches and returns AUR search results for a query
#[cfg(feature = "cache")]
pub fn get_cached_search(query: &str) -> Option<Vec<SearchResult>> {
    let path = SEARCH_CACHE_DIR.join(format!("{}.json", query));
    if let Ok(data) = read_to_string(&path) {
        serde_json::from_str(&data).ok()
    } else {
        None
    }
}

#[cfg(feature = "cache")]
pub fn cache_search_result(query: &str, results: &[SearchResult]) {
    let path = SEARCH_CACHE_DIR.join(format!("{}.json", query));
    if let Ok(json) = serde_json::to_string(results) {
        let _ = create_dir_all(SEARCH_CACHE_DIR.as_path());
        let _ = write(path, json);
    }
}

#[cfg(feature = "cache")]
pub async fn async_get_pkgbuild_cached(pkg: &str) -> String {
    let path = PKGBUILD_CACHE_DIR.join(format!("{}.PKGBUILD", pkg));
    if let Ok(data) = read_to_string(&path) {
        return data;
    }
    let pkgb_preview = crate::aur::get_pkgbuild_preview(pkg);
    println!("[utils] PKGBUILD preview for {}:\n{}", pkg, pkgb_preview);
    for line in pkgb_preview.lines() {
        if line.trim_start().starts_with("pkgname=") {
            println!("[utils] Parsed pkgname for {}: {}", pkg, line.trim_start());
        }
    }
    if pkgb_preview.contains("pkgname") {
        println!("[utils] PKGBUILD for {} contains a pkgname field.", pkg);
    }
    let _ = create_dir_all(PKGBUILD_CACHE_DIR.as_path());
    let _ = write(&path, &_pkgb);
    _pkgb
}

#[cfg(feature = "cache")]
pub fn ensure_cache_dirs() {
    let _ = create_dir_all(PKGBUILD_CACHE_DIR.as_path());
    let _ = create_dir_all(SEARCH_CACHE_DIR.as_path());
}

/// Audit a package by checking its source and dependencies
pub fn audit_package(pkg: &str) {
    match crate::core::detect_source(pkg, None, false) {
        Some(crate::core::Source::Aur) => {
            println!("[AUDIT][AUR] Auditing PKGBUILD for {}...", pkg);
            let pkgb = crate::aur::get_pkgbuild_preview(pkg);
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

type PkgMeta = (
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
    Vec<String>,
);

/// Parse PKGBUILD for depends, makedepends, conflicts and check system state
pub fn resolve_deps(pkgb: &str) -> PkgMeta {
    let mut depends = Vec::new();
    let mut makedepends = Vec::new();
    let mut conflicts = Vec::new();
    let mut missing = Vec::new();
    let mut conflicting = Vec::new();
    for line in pkgb.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("depends=") {
            let dep_line = trimmed.split_once('=').map(|x| x.1).unwrap_or("").trim();
            depends.extend(
                dep_line
                    .trim_matches(&['(', ')', '"', '\'', ' '] as &[_])
                    .split_whitespace()
                    .map(|s| s.trim_matches(&['"', '\'', ' '] as &[_]))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
        } else if trimmed.starts_with("makedepends=") {
            let dep_line = trimmed.split_once('=').map(|x| x.1).unwrap_or("").trim();
            makedepends.extend(
                dep_line
                    .trim_matches(&['(', ')', '"', '\'', ' '] as &[_])
                    .split_whitespace()
                    .map(|s| s.trim_matches(&['"', '\'', ' '] as &[_]))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
        } else if trimmed.starts_with("conflicts=") {
            let dep_line = trimmed.split_once('=').map(|x| x.1).unwrap_or("").trim();
            conflicts.extend(
                dep_line
                    .trim_matches(&['(', ')', '"', '\'', ' '] as &[_])
                    .split_whitespace()
                    .map(|s| s.trim_matches(&['"', '\'', ' '] as &[_]))
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string()),
            );
        }
    }
    // Check installed packages
    let installed: Vec<String> = {
        let output = std::process::Command::new("pacman").arg("-Q").output();
        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
                .collect()
        } else {
            Vec::new()
        }
    };
    for dep in depends.iter().chain(makedepends.iter()) {
        if !installed.contains(dep) {
            missing.push(dep.clone());
        }
    }
    for c in &conflicts {
        if installed.contains(c) {
            conflicting.push(c.clone());
        }
    }
    (depends, makedepends, conflicts, missing, conflicting)
}

// --- CLI stub functions for compatibility ---
pub fn pkgb_diff_audit(package: &str, pkgb: &str) {
    let backup_path =
        std::path::PathBuf::from(format!("/var/lib/reaper/backup/{}_PKGBUILD.bak", package));
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

#[cfg(feature = "cache")]
#[allow(dead_code)]
pub fn compare_pkgbuilds(pkg: &str, new_pkgb: &str) {
    let backup_path =
        std::path::PathBuf::from(format!("/var/lib/reaper/backups/{}/PKGBUILD.bak", pkg));
    if let Ok(old_pkgb) = fs::read_to_string(&backup_path) {
        println!("[reap][diff] Diff for {}:", pkg);
        for diff in lines(&old_pkgb, new_pkgb) {
            match diff {
                diff::Result::Left(l) => println!("- {}", l),
                diff::Result::Right(r) => println!("+ {}", r),
                diff::Result::Both(l, _) => println!("  {}", l),
            }
        }
    } else {
        println!(
            "[reap][diff] No previous PKGBUILD backup found for {}.",
            pkg
        );
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

pub async fn check_keyserver_async(keyserver: &str) {
    let output = tokio::process::Command::new("gpg")
        .args(["--keyserver", keyserver, "--list-keys"])
        .output()
        .await
        .map_err(|e| {
            eprintln!("[reap] keyserver check failed: {}", e);
            e
        })
        .unwrap_or_else(|e| {
            eprintln!("[reap] keyserver check failed: {}", e);
            std::process::Output {
                status: std::process::ExitStatus::from_raw(1),
                stdout: Vec::new(),
                stderr: Vec::new(),
            }
        });
    if output.status.success() {
        println!("[reap] GPG keyserver {} is reachable.", keyserver);
    } else {
        println!("[reap] GPG keyserver {} is NOT reachable.", keyserver);
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
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".config/reap/pinned.toml");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(toml) = contents.parse::<Value>() {
            if let Some(table) = toml.as_table() {
                return table.contains_key(pkg);
            }
        }
        // Fallback: check for simple line pin
        contents.lines().any(|line| line.trim() == pkg)
    } else {
        false
    }
}

#[cfg(feature = "cache")]
#[allow(dead_code)]
pub fn pinned_version(pkg: &str) -> Option<String> {
    let config_path = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join(".config/reap/pinned.toml");
    if let Ok(contents) = fs::read_to_string(&config_path) {
        if let Ok(toml) = contents.parse::<Value>() {
            if let Some(table) = toml.as_table() {
                if let Some(Value::String(ver)) = table.get(pkg) {
                    return Some(ver.clone());
                }
            }
        }
    }
    None
}

/// Clean the cache directories used by reap
pub fn clean_cache() -> Result<String, String> {
    let home = dirs::home_dir().unwrap_or_default();
    let cache_dirs = vec![
        "/tmp/reap".to_string(),
        format!("{}/.cache/reap", home.display()),
    ];
    let mut deleted = 0;
    for dir in &cache_dirs {
        let path = std::path::PathBuf::from(dir);
        if path.exists() && path.is_dir() {
            for entry in
                fs::read_dir(&path).map_err(|e| format!("Failed to read {}: {}", dir, e))?
            {
                let entry = entry.map_err(|e| format!("Read dir error: {}", e))?;
                let p = entry.path();
                if p.is_file() {
                    fs::remove_file(&p).map_err(|e| {
                        eprintln!("[reap] Failed to remove {}: {}", p.display(), e);
                        e.to_string()
                    })?;
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
                            issues.push(format!(
                                "Broken symlink: {} -> {}",
                                path.display(),
                                target.display()
                            ));
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

pub fn build_pkg(pkgdir: &std::path::Path, edit: bool) -> Result<(), String> {
    use std::env;
    use std::process::Command;
    let pkgb_path = pkgdir.join("PKGBUILD");
    if edit {
        let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
        let status = Command::new(editor).arg(&pkgb_path).status();
        if let Ok(s) = status {
            if !s.success() {
                return Err(format!("[reap] Editor exited with status: {}", s));
            }
        } else {
            return Err("[reap] Failed to launch editor".to_string());
        }
    }
    let output = Command::new("makepkg")
        .arg("-si")
        .arg("--noconfirm")
        .current_dir(pkgdir)
        .output();
    match output {
        Ok(out) => {
            if out.status.success() {
                println!(
                    "[reap] makepkg succeeded:\n{}",
                    String::from_utf8_lossy(&out.stdout)
                );
                Ok(())
            } else {
                eprintln!(
                    "[reap] makepkg failed:\n{}",
                    String::from_utf8_lossy(&out.stderr)
                );
                Err(format!("[reap] makepkg failed with status: {}", out.status))
            }
        }
        Err(e) => Err(format!("[reap] Failed to run makepkg: {}", e)),
    }
}

pub fn backup_config() -> Result<(), String> {
    use std::fs;
    use std::path::PathBuf;
    let config_dir = dirs::home_dir().unwrap_or_default().join(".config/reap");
    let backup_dir = PathBuf::from("/var/lib/reaper/backups/config");
    if !config_dir.exists() {
        return Err(format!(
            "[reap] Config dir not found: {}",
            config_dir.display()
        ));
    }
    if let Err(e) = fs::create_dir_all(&backup_dir) {
        return Err(format!("[reap] Failed to create backup dir: {}", e));
    }
    let options = fs_extra::dir::CopyOptions::new()
        .overwrite(true)
        .content_only(false);
    match fs_extra::dir::copy(&config_dir, &backup_dir, &options) {
        Ok(_) => {
            println!("[reap] Backed up config to {}", backup_dir.display());
            Ok(())
        }
        Err(e) => Err(format!("[reap] Failed to backup config: {}", e)),
    }
}

#[cfg(feature = "cache")]
pub mod cache {
    use crate::aur::SearchResult;
    use once_cell::sync::Lazy;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    static PKGBUILD_CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("reap/pkgbuilds")
    });
    static SEARCH_CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("reap/search")
    });

    pub fn save_pkgbuild(pkg: &str, content: &str) {
        let path = PKGBUILD_CACHE_DIR.join(pkg);
        let _ = fs::create_dir_all(PKGBUILD_CACHE_DIR.as_path());
        let _ = fs::write(path, content);
    }

    pub fn load_pkgbuild(pkg: &str) -> Option<String> {
        let path = PKGBUILD_CACHE_DIR.join(pkg);
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(Duration::from_secs(0))
                    < Duration::from_secs(60 * 60 * 24)
                {
                    return fs::read_to_string(&path).ok();
                }
            }
        }
        None
    }

    pub fn expire_cache() {
        let _ = fs::remove_dir_all(PKGBUILD_CACHE_DIR.as_path());
        let _ = fs::remove_dir_all(SEARCH_CACHE_DIR.as_path());
    }

    pub fn save_search(query: &str, results: &[SearchResult]) {
        let path = SEARCH_CACHE_DIR.join(query);
        let _ = fs::create_dir_all(SEARCH_CACHE_DIR.as_path());
        let _ = fs::write(path, serde_json::to_string(results).unwrap_or_default());
    }

    pub fn load_search(query: &str) -> Option<Vec<SearchResult>> {
        let path = SEARCH_CACHE_DIR.join(query);
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if SystemTime::now()
                    .duration_since(modified)
                    .unwrap_or(Duration::from_secs(0))
                    < Duration::from_secs(60 * 60 * 24)
                {
                    if let Ok(data) = fs::read_to_string(&path) {
                        return serde_json::from_str(&data).ok();
                    }
                }
            }
        }
        None
    }
}

/// Audit a PKGBUILD for risky patterns
pub fn audit_pkgbuild(pkgbuild: &str) {
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
            println!(
                "[AUDIT] Warning: PKGBUILD contains potentially risky pattern: {}",
                pat
            );
        }
    }
}

pub fn audit_flatpak_manifest(_pkg: &str) {
    // Stub: Flatpak manifest audit not yet implemented
}

pub fn rollback(pkg: &str) {
    use std::fs;
    use std::path::PathBuf;
    let backup_dir = PathBuf::from(format!("/var/lib/reaper/backups/{}/", pkg));
    let pkgbuild_bak = backup_dir.join("PKGBUILD.bak");
    let pkgbuild = backup_dir.join("PKGBUILD");
    if pkgbuild_bak.exists() {
        if let Err(e) = fs::copy(&pkgbuild_bak, &pkgbuild) {
            eprintln!("[reap][rollback] Failed to restore PKGBUILD: {}", e);
        } else {
            println!("[reap][rollback] Restored PKGBUILD for {}.", pkg);
        }
    } else {
        eprintln!("[reap][rollback] No PKGBUILD backup found for {}.", pkg);
    }
    // Optionally clean up failed build dirs
    let tmp_dir = std::env::temp_dir().join(format!("reap-aur-{}", pkg));
    if tmp_dir.exists() {
        let _ = fs::remove_dir_all(&tmp_dir);
        println!("[reap][rollback] Cleaned up temp dir for {}.", pkg);
    }
}

/// Plugin loader: scan ~/.config/reap/plugins/ for executable .sh/.rs plugins
pub fn load_plugins() -> Vec<std::path::PathBuf> {
    let plugin_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
        .join("reap/plugins");
    let mut plugins = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&plugin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && (path.extension() == Some("sh".as_ref())
                    || path.extension() == Some("rs".as_ref()))
                && is_executable(&path)
            {
                plugins.push(path);
            }
        }
    }
    plugins
}

fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

// Ensure all async/parallel flows use owned values or Arc<T> in async blocks
// Use Arc::clone for shared state if needed
// Add explicit return types for async blocks using ?
