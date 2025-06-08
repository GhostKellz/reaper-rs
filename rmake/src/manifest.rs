use std::fs;
use std::path::Path;

pub enum ManifestType {
    PKGBUILD,
    ForgeToml,
    ForgeLua,
    AutoRust,
}

pub struct Manifest {
    pub manifest_type: ManifestType,
    pub path: String,
    pub data: Option<ManifestData>,
}

#[derive(Debug, Clone)]
pub struct ManifestData {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub build: Option<String>,
    pub install: Option<String>,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub depends: Option<Vec<String>>,
    pub makedepends: Option<Vec<String>>,
    pub optdepends: Option<Vec<String>>,
}

impl Manifest {
    pub fn detect() -> Option<Manifest> {
        // Prefer PKGBUILD in root
        if Path::new("PKGBUILD").exists() {
            let data = Manifest::parse_pkgbuild("PKGBUILD");
            return Some(Manifest {
                manifest_type: ManifestType::PKGBUILD,
                path: "PKGBUILD".to_string(),
                data,
            });
        }
        // Recursively search for forge.toml or Cargo.toml in subdirs
        fn search_dir(dir: &Path) -> Option<Manifest> {
            for entry in std::fs::read_dir(dir).ok()? {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(m) = search_dir(&path) {
                        return Some(m);
                    }
                } else if let Some(fname) = path.file_name().and_then(|n| n.to_str()) {
                    if fname == "forge.toml" {
                        let data = Manifest::parse_forge_toml(path.to_str().unwrap());
                        return Some(Manifest {
                            manifest_type: ManifestType::ForgeToml,
                            path: path.to_string_lossy().to_string(),
                            data,
                        });
                    } else if fname == "Cargo.toml" {
                        return Some(Manifest {
                            manifest_type: ManifestType::AutoRust,
                            path: path.to_string_lossy().to_string(),
                            data: None,
                        });
                    }
                }
            }
            None
        }
        if let Some(found) = search_dir(Path::new(".")) {
            return Some(found);
        }
        // Fallback: forge.lua in root
        if Path::new("forge.lua").exists() {
            return Some(Manifest {
                manifest_type: ManifestType::ForgeLua,
                path: "forge.lua".to_string(),
                data: None,
            });
        }
        None
    }

    pub fn detect_with_path(path: Option<&str>) -> Option<Manifest> {
        if let Some(path) = path {
            if Path::new(path).exists() {
                if path.ends_with("forge.lua") {
                    return Some(Manifest {
                        manifest_type: ManifestType::ForgeLua,
                        path: path.to_string(),
                        data: None, // Will be handled by Lua runtime
                    });
                } else if path.ends_with("forge.toml") {
                    let data = Manifest::parse_forge_toml(path);
                    return Some(Manifest {
                        manifest_type: ManifestType::ForgeToml,
                        path: path.to_string(),
                        data,
                    });
                } else if path.ends_with("PKGBUILD") {
                    let data = Manifest::parse_pkgbuild(path);
                    return Some(Manifest {
                        manifest_type: ManifestType::PKGBUILD,
                        path: path.to_string(),
                        data,
                    });
                } else if path.ends_with("Cargo.toml") {
                    return Some(Manifest {
                        manifest_type: ManifestType::AutoRust,
                        path: path.to_string(),
                        data: None,
                    });
                }
            }
        }
        Manifest::detect()
    }

    pub fn describe(&self) -> &'static str {
        match self.manifest_type {
            ManifestType::PKGBUILD => "PKGBUILD",
            ManifestType::ForgeToml => "forge.toml",
            ManifestType::ForgeLua => "forge.lua",
            ManifestType::AutoRust => "Rust/Cargo.toml (auto)",
        }
    }

    pub fn parse_forge_toml(path: &str) -> Option<ManifestData> {
        let content = fs::read_to_string(path).ok()?;
        let value: toml::Value = toml::from_str(&content).ok()?;
        Some(ManifestData {
            name: value.get("name")?.as_str()?.to_string(),
            version: value.get("version")?.as_str()?.to_string(),
            author: value
                .get("author")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            license: value
                .get("license")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            build: value
                .get("build")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            install: value
                .get("install")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            source: value
                .get("source")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            checksum: value
                .get("checksum")
                .and_then(|v| v.as_str().map(|s| s.to_string())),
            depends: None,
            makedepends: None,
            optdepends: None,
        })
    }

    pub fn parse_pkgbuild(path: &str) -> Option<ManifestData> {
        let content = fs::read_to_string(path).ok()?;
        let name = regex_extract(&content, r#"pkgname=(['"]?)([^'"\s]+)\1"#)?;
        let version = regex_extract(&content, r#"pkgver=(['"]?)([^'"\s]+)\1"#)?;
        let license = regex_extract(&content, r#"license=(['"]?)([^'"\s]+)\1"#);
        let build = if content.contains("build()") {
            Some("build".to_string())
        } else {
            None
        };
        let install = if content.contains("package()") {
            Some("package".to_string())
        } else {
            None
        };
        let depends = extract_array(&content, "depends");
        let makedepends = extract_array(&content, "makedepends");
        let optdepends = extract_array(&content, "optdepends");
        Some(ManifestData {
            name,
            version,
            author: None,
            license,
            build,
            install,
            source: None,
            checksum: None,
            depends,
            makedepends,
            optdepends,
        })
    }
}

fn regex_extract(content: &str, pat: &str) -> Option<String> {
    let re = regex::Regex::new(pat).ok()?;
    re.captures(content)
        .and_then(|cap| cap.get(2).map(|m| m.as_str().to_string()))
}

fn extract_array(content: &str, var: &str) -> Option<Vec<String>> {
    let re = regex::Regex::new(&format!(r#"{}=\(([^)]*)\)"#, var)).ok()?;
    let caps = re.captures(content)?;
    let arr = caps.get(1)?.as_str();
    let vals: Vec<String> = arr
        .split_whitespace()
        .map(|s| s.trim_matches(|c| c == '"' || c == '\''))
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    if vals.is_empty() { None } else { Some(vals) }
}

