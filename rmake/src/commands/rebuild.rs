use crate::commands::package;
use crate::commands::ui;
use std::fs;
use std::process::Command;

pub fn run() {
    // Read current directory for files
    let entries = match fs::read_dir(".") {
        Ok(e) => e,
        Err(_) => return,
    };

    // Iterate over entries and check for PKGBUILD or forge.toml
    for entry in entries.flatten() {
        let fname = entry.file_name();
        let fname_str = fname.to_string_lossy();

        // If PKGBUILD or forge.toml is found, proceed with rebuild
        if fname_str == "PKGBUILD" || fname_str == "forge.toml" {
            ui::print_info(&format!("Rebuilding package: {}", fname_str));

            // Run forge build for rebuild
            let status = Command::new("forge").arg("build").status();
            match status {
                Ok(s) if s.success() => {
                    ui::print_success(&format!("Rebuild succeeded for {}", fname_str));

                    // After successful rebuild, create package
                    let _ = package::create_package(".", &format!("{}.pkg.tar.zst", fname_str));
                }
                Ok(s) => ui::print_error(&format!(
                    "Rebuild failed for {} with status: {}",
                    fname_str, s
                )),
                Err(e) => ui::print_error(&format!(
                    "Failed to run forge build for {}: {}",
                    fname_str, e
                )),
            }
        }
    }
}

