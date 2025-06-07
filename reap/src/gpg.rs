use tokio::process::Command as TokioCommand;
use std::process::Command;

/// Import a GPG key from multiple keyservers (sync)
pub fn import_gpg_key(keyid: &str) {
    let keyservers = [
        "hkps://keyserver.ubuntu.com",
        "hkps://keys.openpgp.org",
        "hkps://pgp.mit.edu"
    ];
    for server in &keyservers {
        let status = Command::new("gpg")
            .args(["--keyserver", server, "--recv-keys", keyid])
            .status();
        if let Ok(s) = status {
            if s.success() {
                println!("[reap] gpg :: Imported key {} from {}", keyid, server);
                break;
            }
        }
    }
}

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
                    println!("[reap] gpg :: Key trust: {} Expiry: {}", fields[1], fields[6]);
                }
            }
        }
    }
}

/// Async GPG key import from multiple keyservers
pub async fn import_gpg_key_async(keyid: &str) {
    let keyservers = [
        "hkps://keyserver.ubuntu.com",
        "hkps://keys.openpgp.org",
        "hkps://pgp.mit.edu"
    ];
    for server in &keyservers {
        let status = TokioCommand::new("gpg")
            .args(["--keyserver", server, "--recv-keys", keyid])
            .status()
            .await;
        if let Ok(s) = status {
            if s.success() {
                println!("[reap] gpg :: Imported key {} from {}", keyid, server);
                break;
            }
        }
    }
}

/// Async GPG key info display
pub async fn show_gpg_key_info_async(keyid: &str) {
    let output = TokioCommand::new("gpg")
        .args(["--list-keys", keyid, "--with-colons"])
        .output()
        .await;
    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        for line in stdout.lines() {
            if line.starts_with("pub:") {
                let fields: Vec<&str> = line.split(':').collect();
                if fields.len() > 6 {
                    println!("[reap] gpg :: Key trust: {} Expiry: {}", fields[1], fields[6]);
                }
            }
        }
    }
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

