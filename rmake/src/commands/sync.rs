use std::process::Command;

pub fn run() {
    let repo_dir = "repo"; // Directory containing the local repository
    let remote = std::env::var("FORGE_SYNC_REMOTE").ok(); // Fetch remote destination from environment variable

    if let Some(remote) = remote {
        println!("Syncing 'repo/' to remote destination: {}", remote);
        // Run rsync to sync local repository to remote destination
        let status = Command::new("rsync")
            .arg("-avz") // Archive, verbose, and compress options for rsync
            .arg(repo_dir) // Source directory
            .arg(&remote) // Remote destination
            .status();

        match status {
            Ok(s) if s.success() => println!("Sync completed successfully."),
            Ok(s) => println!("Sync failed with exit status: {}", s),
            Err(e) => println!("Failed to run rsync: {}", e),
        }
    } else {
        // Provide feedback if FORGE_SYNC_REMOTE is not set
        println!(
            "No FORGE_SYNC_REMOTE set. Please set the environment variable to the remote destination, e.g.:\n\
            FORGE_SYNC_REMOTE=user@host:/path/to/remote"
        );
    }
}

