use std::fs;
use std::process::Command;

pub fn run() {
    let remote = std::env::var("FORGE_UPLOAD_REMOTE").ok();
    if let Some(remote) = remote {
        let entries = match fs::read_dir(".") {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut found = false;
        for entry in entries.flatten() {
            let fname = entry.file_name();
            if fname.to_string_lossy().ends_with(".pkg.tar.zst") {
                println!("Uploading {} to {}", fname.to_string_lossy(), remote);
                let status = Command::new("scp").arg(entry.path()).arg(&remote).status();
                match status {
                    Ok(s) if s.success() => println!("Upload succeeded."),
                    Ok(s) => println!("Upload failed with status: {}", s),
                    Err(e) => println!("Failed to run scp: {}", e),
                }
                found = true;
            }
        }
        if !found {
            println!("No .pkg.tar.zst files found to upload.");
        }
    } else {
        println!("No FORGE_UPLOAD_REMOTE set. To upload, set FORGE_UPLOAD_REMOTE=user@host:/path");
    }
}
