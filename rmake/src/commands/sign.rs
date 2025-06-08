use std::fs;
use std::process::Command;

pub fn run() {
    // Find the first .pkg.tar.zst file in the current directory
    let pkg = match fs::read_dir(".") {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().ends_with(".pkg.tar.zst")),
        Err(e) => {
            println!("Failed to read the directory: {}", e); // Improved error message
            return;
        }
    };

    if let Some(pkg) = pkg {
        let pkg_path = pkg.file_name();
        println!("Signing package: {}", pkg_path.to_string_lossy());
        
        // Output the exact command being run for debugging
        let status = Command::new("gpg")
            .arg("--detach-sign")
            .arg(&pkg_path)
            .status();
        
        match status {
            Ok(s) if s.success() => {
                println!("Package signed: {}.sig", pkg_path.to_string_lossy())
            }
            Ok(s) => {
                println!("GPG sign failed with status: {}", s);
            }
            Err(e) => {
                println!("Failed to run gpg: {}", e);
            }
        }
    } else {
        println!("No .pkg.tar.zst file found to sign.");
    }
}

