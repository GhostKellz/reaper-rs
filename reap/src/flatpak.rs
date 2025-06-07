use std::process::Command;

// Flatpak integration (scaffold)
pub fn search(query: &str) {
    println!("[reap] flatpak :: Searching for: {}", query);
    let output = Command::new("flatpak").arg("search").arg(query).output();
    match output {
        Ok(out) => {
            let results = String::from_utf8_lossy(&out.stdout);
            println!("{}", results);
        }
        Err(e) => println!("[reap] flatpak :: search failed: {}", e),
    }
}
// Example usage: call flatpak::search from CLI or TUI for Flatpak search

pub fn install(package: &str) {
    println!("[reap] flatpak :: Installing: {}", package);
    let status = Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg(package)
        .status();
    match status {
        Ok(s) if s.success() => println!("[reap] flatpak :: {} installed successfully!", package),
        Ok(_) | Err(_) => println!("[reap] flatpak :: install failed for {}", package),
    }
}

pub fn upgrade() {
    println!("[reap] flatpak :: Upgrading all flatpak packages...");
    let status = Command::new("flatpak").arg("update").arg("-y").status();
    match status {
        Ok(s) if s.success() => println!("[reap] flatpak :: All packages upgraded!"),
        Ok(_) | Err(_) => println!("[reap] flatpak :: upgrade failed."),
    }
}
// Example usage: call flatpak::upgrade from CLI or TUI for Flatpak upgrade

pub fn print_flatpak_sandbox_info(pkg: &str) {
    let output = Command::new("flatpak").arg("info").arg(pkg).output();
    if let Ok(out) = output {
        let info = String::from_utf8_lossy(&out.stdout);
        if info.contains("sandbox: none") {
            println!("[reap] flatpak :: Warning: Flatpak {} is NOT sandboxed!", pkg);
        } else {
            println!("[reap] flatpak :: sandbox info for {}:\n{}", pkg, info);
        }
    }
}
// Example usage: call print_flatpak_sandbox_info in TUI/CLI details

