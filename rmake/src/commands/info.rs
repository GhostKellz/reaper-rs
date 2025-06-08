use crate::commands::ui;
use crate::manifest::Manifest;
use std::env;

pub fn run() {
    let args: Vec<String> = env::args().collect();
    let manifest_path = args
        .iter()
        .skip(2)
        .find(|a| a.ends_with(".toml") || a.ends_with("PKGBUILD"));
    let manifest = if let Some(path) = manifest_path {
        Manifest::detect_with_path(Some(path))
    } else {
        Manifest::detect()
    };
    match manifest {
        Some(manifest) => {
            if let Some(data) = &manifest.data {
                ui::print_info(&format!("Manifest info ({}):", manifest.describe()));
                ui::print_info(&format!("  Name: {}", data.name));
                ui::print_info(&format!("  Version: {}", data.version));
                if let Some(author) = &data.author {
                    ui::print_info(&format!("  Author: {}", author));
                }
                if let Some(license) = &data.license {
                    ui::print_info(&format!("  License: {}", license));
                }
                if let Some(source) = &data.source {
                    ui::print_info(&format!("  Source: {}", source));
                }
                if let Some(checksum) = &data.checksum {
                    ui::print_info(&format!("  Checksum: {}", checksum));
                }
                if let Some(build) = &data.build {
                    ui::print_info(&format!("  Build: {}", build));
                }
                if let Some(install) = &data.install {
                    ui::print_info(&format!("  Install: {}", install));
                }
            } else {
                ui::print_info("  (No manifest data parsed)");
            }
        }
        None => {
            ui::print_error(
                "No PKGBUILD or forge.toml found in the current directory or specified path.",
            );
        }
    }
}
