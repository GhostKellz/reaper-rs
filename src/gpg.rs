use std::path::Path;
use std::process::Command;
use tokio::process::Command as TokioCommand;

/// Show GPG key info (sync)
pub fn show_gpg_key_info(keyid: &str) {
    let output = Command::new("gpg")
        .args(["--list-keys", keyid, "--with-colons"])
        .output();
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if line.starts_with("pub:") {
                let fields: Vec<&str> = line.split(':').collect();
                if fields.len() > 6 {
                    println!(
                        "[reap] gpg :: Key trust: {} Expiry: {}",
                        fields[1], fields[6]
                    );
                }
            }
        }
    }
}

/// Verify PKGBUILD signature in a directory
pub fn verify_pkgbuild(pkgdir: &Path) -> bool {
    let sig_path = pkgdir.join("PKGBUILD.sig");
    let pkgb_path = pkgdir.join("PKGBUILD");
    if !sig_path.exists() || !pkgb_path.exists() {
        eprintln!("[reap] gpg :: PKGBUILD or signature missing");
        return false;
    }
    let status = Command::new("gpg")
        .arg("--verify")
        .arg(sig_path)
        .arg(pkgb_path)
        .status();
    if let Ok(s) = status {
        if s.success() {
            println!("[reap] gpg :: PKGBUILD signature verified");
            return true;
        }
    }
    eprintln!("[reap] gpg :: PKGBUILD signature verification failed");
    false
}

#[allow(dead_code)]
/// Refresh all GPG keys
pub fn refresh_keys() {
    // TODO: Wire this into CLI flow in core::handle_cli()
    let status = Command::new("gpg").arg("--refresh-keys").status();
    if let Ok(s) = status {
        if s.success() {
            println!("[reap] gpg :: Refreshed all keys");
        } else {
            eprintln!("[reap] gpg :: Failed to refresh keys");
        }
    }
}

/// Async GPG key import from multiple keyservers
pub async fn import_gpg_key_async(keyid: &str) -> Result<(), String> {
    let keyservers = [
        "hkps://keyserver.ubuntu.com",
        "hkps://keys.openpgp.org",
        "hkps://pgp.mit.edu",
    ];
    let mut last_err = None;
    for server in &keyservers {
        match TokioCommand::new("gpg")
            .args(["--keyserver", server, "--recv-keys", keyid])
            .status()
            .await
        {
            Ok(s) if s.success() => {
                println!("[reap] gpg :: Imported key {} from {}", keyid, server);
                return Ok(());
            }
            Ok(_) => {
                last_err = Some(format!("Failed to import key from {}", server));
            }
            Err(e) => {
                last_err = Some(format!("TokioCommand error for {}: {}", server, e));
            }
        }
    }
    Err(last_err.unwrap_or_else(|| "All keyserver attempts failed".to_string()))
}

/// Async GPG key presence check
pub async fn check_key(keyid: &str) {
    let output = TokioCommand::new("gpg")
        .args(["--list-keys", keyid])
        .output()
        .await;
    if let Ok(out) = output {
        if out.status.success() {
            println!("[reap] gpg :: GPG key {} is present.", keyid);
        } else {
            println!("[reap] gpg :: GPG key {} is NOT present.", keyid);
        }
    } else {
        println!("[reap] gpg :: Failed to check GPG key {}.", keyid);
    }
}
