use std::fs;
use std::path::Path;

pub fn run() {
    let repo_dir = "repo";
    
    // Create the repo directory if it doesn't exist
    if !Path::new(repo_dir).exists() {
        if let Err(e) = fs::create_dir(repo_dir) {
            println!("Failed to create repo directory: {}", e);
            return;
        }
    }

    // Read current directory for .pkg.tar.zst files
    let entries = match fs::read_dir(".") {
        Ok(e) => e,
        Err(_) => {
            println!("Failed to read current directory.");
            return;
        }
    };

    let mut found = false;
    // Iterate over the files and check for .pkg.tar.zst files
    for entry in entries.flatten() {
        let fname = entry.file_name();
        
        // Only handle .pkg.tar.zst files
        if fname.to_string_lossy().ends_with(".pkg.tar.zst") {
            let dest = Path::new(repo_dir).join(&fname);
            
            // Attempt to copy the file to the repo directory
            match fs::copy(entry.path(), &dest) {
                Ok(_) => println!("Published {} to repo/", fname.to_string_lossy()),
                Err(e) => println!("Failed to publish {}: {}", fname.to_string_lossy(), e),
            }
            found = true;
        }
    }

    if !found {
        println!("No .pkg.tar.zst files found to publish.");
    }
}

