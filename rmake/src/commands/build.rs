use crate::commands::ui;
use crate::manifest::Manifest;
use atty;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Recursively search for a manifest file in the current directory and subdirectories
fn find_manifest() -> Option<PathBuf> {
    let manifest_names = ["forge.lua", "forge.toml", "PKGBUILD", "Cargo.toml"];
    fn search_dir(dir: &Path, names: &[&str]) -> Option<PathBuf> {
        for entry in fs::read_dir(dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = search_dir(&path, names) {
                    return Some(found);
                }
            } else if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                if names.contains(&fname) {
                    return Some(path);
                }
            }
        }
        None
    }
    search_dir(Path::new("."), &manifest_names)
}

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let manifest_path = args
        .iter()
        .skip(2)
        .find(|a| a.ends_with(".toml") || a.ends_with("PKGBUILD") || a.ends_with("forge.lua"));
    let manifest = if let Some(path) = manifest_path {
        Manifest::detect_with_path(Some(path))
    } else {
        // Try current dir, then recursively search
        Manifest::detect()
            .or_else(|| find_manifest().and_then(|p| Manifest::detect_with_path(p.to_str())))
    };

    // Source/artifact caching: use .forge-cache/ for built binaries and sources
    let cache_dir = ".forge-cache";
    let _ = fs::create_dir_all(cache_dir);

    match manifest {
        Some(manifest) => {
            ui::print_info(&format!(
                "Detected manifest: {} (at {})",
                manifest.describe(),
                manifest.path
            ));
            match manifest.manifest_type {
                crate::manifest::ManifestType::ForgeLua => {
                    let status = Command::new("lua").arg("forge.lua").arg("build").status();
                    match status {
                        Ok(s) if s.success() => ui::print_success("forge.lua build() succeeded."),
                        Ok(s) => {
                            ui::print_error(&format!("forge.lua build() failed with status: {}", s))
                        }
                        Err(e) => ui::print_error(&format!("Failed to run forge.lua: {}", e)),
                    }
                }
                crate::manifest::ManifestType::AutoRust => {
                    // Zero-config Rust build
                    let cargo_toml = fs::read_to_string("Cargo.toml").unwrap_or_default();
                    let bin_name = cargo_toml
                        .lines()
                        .find_map(|l| {
                            if l.trim().starts_with("name") {
                                l.split('=')
                                    .nth(1)
                                    .map(|s| s.trim().trim_matches('"').to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "main".to_string());
                    ui::print_info(&format!(
                        "Auto-detected Rust project (Cargo.toml). Building {}...",
                        bin_name
                    ));
                    // Check cache
                    let bin_path = format!("target/release/{}", bin_name);
                    let cache_bin = format!("{}/{}", cache_dir, bin_name);
                    if fs::metadata(&cache_bin).is_ok() {
                        ui::print_success(&format!("Using cached binary: {}", cache_bin));
                        if atty::is(atty::Stream::Stdout) {
                            println!("Install cached binary to /usr/bin/{}? [Y/n]", bin_name);
                            use std::io::{self, Write};
                            let mut input = String::new();
                            io::stdout().flush().unwrap();
                            io::stdin().read_line(&mut input).unwrap();
                            if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y") {
                                let status = Command::new("sudo")
                                    .arg("install")
                                    .arg("-Dm755")
                                    .arg(&cache_bin)
                                    .arg(format!("/usr/bin/{}", bin_name))
                                    .status();
                                match status {
                                    Ok(s) if s.success() => ui::print_success("Install succeeded."),
                                    Ok(s) => ui::print_error(&format!(
                                        "Install failed with status: {}",
                                        s
                                    )),
                                    Err(e) => {
                                        ui::print_error(&format!("Failed to run install: {}", e))
                                    }
                                }
                            } else {
                                println!("Install skipped.");
                            }
                        }
                        return;
                    }
                    let status = Command::new("cargo").arg("build").arg("--release").status();
                    match status {
                        Ok(s) if s.success() => {
                            ui::print_success(&format!(
                                "Rust build succeeded. Binary: {}",
                                bin_path
                            ));
                            // Cache the binary
                            let _ = fs::copy(&bin_path, &cache_bin);
                            ui::print_info(&format!("Cached binary at {}", cache_bin));
                            // Offer to install
                            if atty::is(atty::Stream::Stdout) {
                                println!("Install to /usr/bin/{}? [Y/n]", bin_name);
                                use std::io::{self, Write};
                                let mut input = String::new();
                                io::stdout().flush().unwrap();
                                io::stdin().read_line(&mut input).unwrap();
                                if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y")
                                {
                                    let status = Command::new("sudo")
                                        .arg("install")
                                        .arg("-Dm755")
                                        .arg(&bin_path)
                                        .arg(format!("/usr/bin/{}", bin_name))
                                        .status();
                                    match status {
                                        Ok(s) if s.success() => {
                                            ui::print_success("Install succeeded.")
                                        }
                                        Ok(s) => ui::print_error(&format!(
                                            "Install failed with status: {}",
                                            s
                                        )),
                                        Err(e) => ui::print_error(&format!(
                                            "Failed to run install: {}",
                                            e
                                        )),
                                    }
                                } else {
                                    println!("Install skipped.");
                                }
                            }
                        }
                        Ok(s) => ui::print_error(&format!("Rust build failed with status: {}", s)),
                        Err(e) => ui::print_error(&format!("Failed to run cargo build: {}", e)),
                    }
                }
                crate::manifest::ManifestType::ForgeToml => {
                    if let Some(data) = &manifest.data {
                        if data.build.is_none() {
                            ui::print_error("Lint: No build command found in manifest.");
                        }
                        if let Some(build_cmd) = &data.build {
                            ui::print_info(&format!("Running build command: {}", build_cmd));
                            let status = if cfg!(target_os = "windows") {
                                Command::new("cmd").arg("/C").arg(build_cmd).status()
                            } else {
                                Command::new("sh").arg("-c").arg(build_cmd).status()
                            };
                            match status {
                                Ok(s) if s.success() => ui::print_success("Build succeeded."),
                                Ok(s) => {
                                    ui::print_error(&format!("Build failed with status: {}", s))
                                }
                                Err(e) => {
                                    ui::print_error(&format!("Failed to run build command: {}", e))
                                }
                            }
                        }
                    }
                }
                crate::manifest::ManifestType::PKGBUILD => {
                    ui::print_info("Running PKGBUILD build() function...");
                    let status = Command::new("bash")
                        .arg("-c")
                        .arg("source PKGBUILD; build")
                        .status();
                    match status {
                        Ok(s) if s.success() => ui::print_success("PKGBUILD build() succeeded."),
                        Ok(s) => {
                            ui::print_error(&format!("PKGBUILD build() failed with status: {}", s))
                        }
                        Err(e) => {
                            ui::print_error(&format!("Failed to run PKGBUILD build(): {}", e))
                        }
                    }
                }
            }
        }
        None => {
            ui::print_error(
                "No forge.lua, forge.toml, PKGBUILD, or Cargo.toml found in the current directory or any subdirectory. Try 'rmake init' to scaffold a manifest.",
            );
        }
    }
}


