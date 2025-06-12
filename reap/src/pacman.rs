// Pacman repo logic
use std::process::Command;

/// Install a package from the official repositories using pacman
pub fn install(package: &str) {
    println!("[pacman] Installing package: {}", package);
    let status = Command::new("sudo")
        .arg("pacman")
        .arg("-S")
        .arg("--noconfirm")
        .arg(package)
        .status();
    if let Ok(s) = status {
        if s.success() {
            println!("[pacman] {} installed successfully!", package);
        } else {
            eprintln!("[pacman] pacman failed for {}", package);
        }
    } else {
        eprintln!("[pacman] failed to run pacman for {}", package);
    }
}

pub fn is_installed(pkg: &str) -> bool {
    Command::new("pacman")
        .arg("-Q")
        .arg(pkg)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_version(pkg: &str) -> Option<String> {
    let output = Command::new("pacman").arg("-Qi").arg(pkg).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.starts_with("Version") {
            return line.split_whitespace().nth(1).map(|s| s.to_string());
        }
    }
    None
}

pub fn list_installed_aur() -> Vec<String> {
    // TODO: Implement real list
    vec![]
}
