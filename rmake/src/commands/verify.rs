use std::fs;
use std::process::Command;

pub fn run() {
    // Find the first .pkg.tar.zst file in the current directory
    let pkg = match fs::read_dir(".") {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().ends_with(".pkg.tar.zst")),
        Err(_) => None,
    };
    if let Some(pkg) = pkg {
        let pkg_path = pkg.file_name();
        let sig_path = format!("{}.sig", pkg_path.to_string_lossy());
        println!("Verifying signature for: {}", pkg_path.to_string_lossy());
        let status = Command::new("gpg")
            .arg("--verify")
            .arg(&sig_path)
            .arg(&pkg_path)
            .status();
        match status {
            Ok(s) if s.success() => println!("Signature is valid."),
            Ok(s) => println!("Signature verification failed with status: {}", s),
            Err(e) => println!("Failed to run gpg: {}", e),
        }
        // Optionally verify checksum if .sha256 exists
        let sha_path = format!("{}.sha256", pkg_path.to_string_lossy());
        if fs::metadata(&sha_path).is_ok() {
            println!("Verifying checksum: {}", sha_path);
            let status = Command::new("sha256sum").arg("-c").arg(&sha_path).status();
            match status {
                Ok(s) if s.success() => println!("Checksum is valid."),
                Ok(s) => println!("Checksum verification failed with status: {}", s),
                Err(e) => println!("Failed to run sha256sum: {}", e),
            }
        }
    } else {
        println!("No .pkg.tar.zst file found to verify.");
    }
}
