use crate::commands::ui;
use crate::manifest::Manifest;

pub fn run() {
    match Manifest::detect() {
        Some(manifest) => {
            if let Some(data) = &manifest.data {
                let mut errors = Vec::new();
                // Check if required fields are present in forge.toml
                if data.name.trim().is_empty() {
                    errors.push("Missing 'name' field");
                }
                if data.version.trim().is_empty() {
                    errors.push("Missing 'version' field");
                }
                if data.build.is_none() {
                    errors.push("Missing 'build' command");
                }
                if data.install.is_none() {
                    errors.push("Missing 'install' command");
                }
                if errors.is_empty() {
                    ui::print_success("Manifest is valid.");
                } else {
                    ui::print_error("Manifest has errors:");
                    for e in errors {
                        ui::print_error(&format!("- {}", e));
                    }
                }
            } else if manifest.path == "PKGBUILD" {
                // Lint for PKGBUILD if it's found
                let content = std::fs::read_to_string("PKGBUILD").unwrap_or_default();
                let mut errors = Vec::new();
                if !content.contains("pkgname=") {
                    errors.push("Missing 'pkgname' field");
                }
                if !content.contains("pkgver=") {
                    errors.push("Missing 'pkgver' field");
                }
                if !content.contains("build()") {
                    errors.push("Missing build() function");
                }
                if !content.contains("package()") {
                    errors.push("Missing package() function");
                }
                if errors.is_empty() {
                    ui::print_success("PKGBUILD is valid.");
                } else {
                    ui::print_error("PKGBUILD has errors:");
                    for e in errors {
                        ui::print_error(&format!("- {}", e));
                    }
                }
            } else {
                ui::print_error(
                    "No manifest data available (PKGBUILD linting not implemented).\nIf this is a Rust project, ensure you have a forge.toml or PKGBUILD in the root or subdirectory.",
                );
            }
        }
        None => {
            ui::print_error(
                "No PKGBUILD or forge.toml found in the current directory or subdirectories.",
            );
        }
    }
}

