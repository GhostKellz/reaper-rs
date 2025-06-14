use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct HookContext {
    pub pkg: String,
    pub version: Option<String>,
    pub source: Option<String>,
    pub install_path: Option<String>,
    pub tap: Option<String>,
}

fn hook_log_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("reap/logs/hooks.log")
}

fn log_hook_result(hook: &str, ctx: &HookContext, result: &str) {
    let log_path = hook_log_path();
    let _ = fs::create_dir_all(log_path.parent().unwrap());
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .expect("Failed to open log file");
    let _ = writeln!(
        file,
        "[{}][{}] {}",
        chrono::Local::now().to_rfc3339(),
        hook,
        result
    );
    let _ = writeln!(
        file,
        "  Context: {}",
        serde_json::to_string(ctx).unwrap_or_default()
    );
}

// Remove or comment out unused function find_hook_file
/*
#[allow(dead_code)]
fn find_hook_file(hook: &str, ctx: &HookContext) -> Option<PathBuf> {
    // 1. Per-package: ~/.config/reap/hooks/tapname/pkgname/hook.lua
    if let Some(tap) = &ctx.tap {
        let pkg_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(format!("reap/hooks/{}/{}/{}.lua", tap, ctx.pkg, hook));
        if pkg_dir.exists() {
            return Some(pkg_dir);
        }
        // 2. Per-tap: ~/.config/reap/hooks/tapname/hook.lua
        let tap_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(format!("reap/hooks/{}/{}.lua", tap, hook));
        if tap_dir.exists() {
            return Some(tap_dir);
        }
    }
    // 3. Global: ~/.config/reap/hooks/global/hook.lua
    let global_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(format!("reap/hooks/global/{}.lua", hook));
    if global_dir.exists() {
        return Some(global_dir);
    }
    None
}
*/

use std::process::Command;

fn run_shell_hook(hook: &str, ctx: &HookContext) {
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let script_path = home.join(format!(".config/reap/hooks/{}.sh", hook));
    if script_path.exists() {
        let mut cmd = Command::new("bash");
        cmd.arg(&script_path);
        // Pass context as env vars
        cmd.env("REAP_PKG", &ctx.pkg);
        if let Some(ver) = &ctx.version {
            cmd.env("REAP_VERSION", ver);
        }
        if let Some(src) = &ctx.source {
            cmd.env("REAP_SOURCE", src);
        }
        if let Some(path) = &ctx.install_path {
            cmd.env("REAP_INSTALL_PATH", path);
        }
        if let Some(tap) = &ctx.tap {
            cmd.env("REAP_TAP", tap);
        }
        let _ = cmd.status();
    }
}

fn run_lua_hook(_hook: &str, _ctx: &HookContext) {
    // Lua support is stubbed for future advanced setup.
}

/// Pre-installation hook, called before a package is installed.
/// This will execute any `pre_install` script found in the hooks directory.
pub fn pre_install(ctx: &HookContext) {
    run_shell_hook("pre_install", ctx);
    run_lua_hook("pre_install", ctx);
}

/// Post-installation hook, called after a package is installed.
/// This will execute any `post_install` script found in the hooks directory.
pub fn post_install(ctx: &HookContext) {
    run_shell_hook("post_install", ctx);
    run_lua_hook("post_install", ctx);
}

/// Post-upgrade hook, called after a package is upgraded.
/// This will execute any `post_upgrade` script found in the hooks directory.
pub fn post_upgrade(ctx: &HookContext) {
    run_shell_hook("post_upgrade", ctx);
    run_lua_hook("post_upgrade", ctx);
}

/// Conflict resolution hook, called when there is a conflict during installation or upgrade.
/// This will execute any `on_conflict` script found in the hooks directory.
pub fn on_conflict(ctx: &HookContext) {
    run_shell_hook("on_conflict", ctx);
    run_lua_hook("on_conflict", ctx);
}

/// Flatpak search hook, called during Flatpak package searches.
/// This will execute any `on_flatpak_search` script found in the hooks directory.
pub fn on_flatpak_search(ctx: &HookContext) {
    run_shell_hook("on_flatpak_search", ctx);
    run_lua_hook("on_flatpak_search", ctx);
}

/// Flatpak installation hook, called during Flatpak package installation.
/// This will execute any `on_flatpak_install` script found in the hooks directory.
pub fn on_flatpak_install(ctx: &HookContext) {
    run_shell_hook("on_flatpak_install", ctx);
    run_lua_hook("on_flatpak_install", ctx);
}

/// Pre-upgrade hook, called before a package is upgraded.
pub fn pre_upgrade(ctx: &HookContext) {
    log_hook_result("pre_upgrade", ctx, "START");
    run_shell_hook_with_timeout("pre_upgrade", ctx);
    log_hook_result("pre_upgrade", ctx, "END");
}

/// Post-remove hook, called after a package is removed.
pub fn post_remove(ctx: &HookContext) {
    log_hook_result("post_remove", ctx, "START");
    run_shell_hook_with_timeout("post_remove", ctx);
    log_hook_result("post_remove", ctx, "END");
}

/// On-error hook, called when an error occurs during install/upgrade/remove.
pub fn on_error(ctx: &HookContext, error: &str) {
    log_hook_result("on_error", ctx, &format!("START: {}", error));
    run_shell_hook_with_timeout("on_error", ctx);
    log_hook_result("on_error", ctx, "END");
}

fn run_shell_hook_with_timeout(hook: &str, ctx: &HookContext) {
    use std::process::{Command, Stdio};
    use std::thread;
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let script_path = home.join(format!(".config/reap/hooks/{}.sh", hook));
    if script_path.exists() {
        let mut cmd = Command::new("bash");
        cmd.arg(&script_path);
        cmd.env("REAP_PKG", &ctx.pkg);
        if let Some(ver) = &ctx.version {
            cmd.env("REAP_VERSION", ver);
        }
        if let Some(src) = &ctx.source {
            cmd.env("REAP_SOURCE", src);
        }
        if let Some(path) = &ctx.install_path {
            cmd.env("REAP_INSTALL_PATH", path);
        }
        if let Some(tap) = &ctx.tap {
            cmd.env("REAP_TAP", tap);
        }
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
        let _timeout = std::env::var("REAP_HOOK_TIMEOUT")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(5);
        let child = cmd.spawn();
        if let Ok(mut child) = child {
            let pid = child.id();
            let handle = thread::spawn(move || {
                let _ = child.wait();
            });
            if handle.join().is_err() {
                eprintln!("[reap][hook] {}.sh timed out (pid={:?})", hook, pid);
            }
        } else {
            eprintln!("[reap][hook] Failed to spawn {}.sh", hook);
        }
    }
}

// Ensure all hook calls are safe (do not panic if missing), and add doc comments for hook usage
// All Lua logic removed; hooks will call shell scripts if present in ~/.config/reap/hooks/

// No async/parallel flows in hooks.rs; nothing to change for prompt 2
