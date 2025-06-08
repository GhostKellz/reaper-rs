use crate::manifest::Manifest;
use std::fs;
use std::path::Path;
use toml::Value;

pub fn run() {
    match Manifest::detect() {
        Some(manifest) => {
            if let Some(data) = &manifest.data {
                // First try to extract dependencies from the brew.toml file
                if let Some(source) = &data.source {
                    println!("Source: {}", source);
                }

                // Load brew.toml for dependencies
                let config_path = dirs::home_dir()
                    .unwrap_or_default()
                    .join(".config/reaper/brew.toml");
                if config_path.exists() {
                    let content = fs::read_to_string(config_path).unwrap_or_default();
                    if let Ok(toml_value) = toml::de::from_str::<Value>(&content) {
                        if let Some(deps) = toml_value.get("dependencies") {
                            if let Some(dep_array) = deps.as_array() {
                                for dep in dep_array {
                                    if let Some(dep_name) = dep.as_str() {
                                        println!("Dependency from brew.toml: {}", dep_name);
                                    }
                                }
                            }
                        }
                    }
                }

                // For PKGBUILD, try to extract the depends array
                if manifest.path == "PKGBUILD" {
                    let content = fs::read_to_string("PKGBUILD").unwrap_or_default();
                    for line in content.lines() {
                        if line.trim_start().starts_with("depends=") {
                            println!("{}", line.trim());
                        }
                    }
                }
            } else if manifest.path == "PKGBUILD" {
                let content = fs::read_to_string("PKGBUILD").unwrap_or_default();
                for line in content.lines() {
                    if line.trim_start().starts_with("depends=") {
                        println!("{}", line.trim());
                    }
                }
            } else {
                println!("No dependency information found.");
            }
        }
        None => {
            println!("No PKGBUILD or brew.toml found in the current directory.");
        }
    }
}






