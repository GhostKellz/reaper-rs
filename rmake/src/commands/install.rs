use crate::commands::ui;
use crate::manifest::Manifest;
use std::env;
use std::fs;
use std::process::Command;

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let manifest_path = args
        .iter()
        .skip(2)
        .find(|a| a.ends_with(".toml") || a.ends_with("PKGBUILD") || a.ends_with("forge.toml"));
    let manifest = if let Some(path) = manifest_path {
        Manifest::detect_with_path(Some(path))
    } else {
        Manifest::detect()
    };

    match manifest {
        Some(manifest) => {
            ui::print_info(&format!(
                "Detected manifest: {} (at {})",
                manifest.describe(),
                manifest.path
            ));

            match manifest.manifest_type {
                crate::manifest::ManifestType::ForgeToml => {
                    // Handling for forge.toml (relevant for rmake)
                    if let Some(data) = &manifest.data {
                        if let Some(install_cmd) = &data.install {
                            ui::print_info(&format!("Running install command: {}", install_cmd));
                            let status = if cfg!(target_os = "windows") {
                                Command::new("cmd").arg("/C").arg(install_cmd).status()
                            } else {
                                Command::new("sh").arg("-c").arg(install_cmd).status()
                            };
                            match status {
                                Ok(s) if s.success() => ui::print_success("Install succeeded."),
                                Ok(s) => {
                                    ui::print_error(&format!("Install failed with status: {}", s))
                                }
                                Err(e) => ui::print_error(&format!(
                                    "Failed to run install command: {}",
                                    e
                                )),
                            }
                        } else {
                            ui::print_error("No install command found in manifest.");
                        }
                    }
                }
                crate::manifest::ManifestType::PKGBUILD => {
                    // Handling for PKGBUILD
                    ui::print_info("Running PKGBUILD package() function...");
                    let status = Command::new("bash")
                        .arg("-c")
                        .arg("source PKGBUILD; package")
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            ui::print_success("PKGBUILD package() succeeded.");
                            // Try to find and install the built .pkg.tar.zst
                            let pkg = fs::read_dir(".").ok().and_then(|entries| {
                                entries.filter_map(|e| e.ok()).find(|e| {
                                    e.file_name().to_string_lossy().ends_with(".pkg.tar.zst")
                                })
                            });
                            if let Some(pkg) = pkg {
                                let pkg_path = pkg.file_name();
                                println!(
                                    "Install {} with pacman -U? [Y/n]",
                                    pkg_path.to_string_lossy()
                                );
                                use std::io::{self, Write};
                                let mut input = String::new();
                                io::stdout().flush().unwrap();
                                io::stdin().read_line(&mut input).unwrap();
                                if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y")
                                {
                                    let status = Command::new("sudo")
                                        .arg("pacman")
                                        .arg("-U")
                                        .arg(&pkg_path)
                                        .status();
                                    match status {
                                        Ok(s) if s.success() => {
                                            ui::print_success("Package installed with pacman.")
                                        }
                                        Ok(s) => ui::print_error(&format!(
                                            "pacman -U failed with status: {}",
                                            s
                                        )),
                                        Err(e) => {
                                            ui::print_error(&format!("Failed to run pacman: {}", e))
                                        }
                                    }
                                } else {
                                    println!("Install skipped.");
                                }
                            } else {
                                ui::print_error(
                                    "No .pkg.tar.zst file found after packaging. Check your PKGBUILD's package() output.",
                                );
                            }
                        }
                        Ok(s) => ui::print_error(&format!(
                            "PKGBUILD package() failed with status: {}",
                            s
                        )),
                        Err(e) => {
                            ui::print_error(&format!("Failed to run PKGBUILD package(): {}", e))
                        }
                    }
                }
                crate::manifest::ManifestType::AutoRust => {
                    // Handling for AutoRust (Cargo.toml)
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
                    let bin_path = format!("target/release/{}", bin_name);
                    if fs::metadata(&bin_path).is_err() {
                        ui::print_error(&format!(
                            "Binary {} not found. Run 'rmake build' first.",
                            bin_path
                        ));
                        return;
                    }
                    println!("Install {} to /usr/bin/{}? [Y/n]", bin_path, bin_name);
                    use std::io::{self, Write};
                    let mut input = String::new();
                    io::stdout().flush().unwrap();
                    io::stdin().read_line(&mut input).unwrap();
                    if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y") {
                        let status = Command::new("sudo")
                            .arg("install")
                            .arg("-Dm755")
                            .arg(&bin_path)
                            .arg(format!("/usr/bin/{}", bin_name))
                            .status();
                        match status {
                            Ok(s) if s.success() => ui::print_success("Install succeeded."),
                            Ok(s) => ui::print_error(&format!("Install failed with status: {}", s)),
                            Err(e) => ui::print_error(&format!("Failed to run install: {}", e)),
                        }
                    } else {
                        println!("Install skipped.");
                    }
                }
                crate::manifest::ManifestType::ForgeLua => {
                    ui::print_info(
                        "Install is not supported for this manifest type. Use a forge.toml or PKGBUILD manifest for install logic.",
                    );
                }
            }
        }
        None => {
            ui::print_error(
                "No PKGBUILD, forge.toml, or Cargo.toml found in the current directory or specified path. Try 'rmake init' to scaffold a manifest.",
            );
        }
    }
}

