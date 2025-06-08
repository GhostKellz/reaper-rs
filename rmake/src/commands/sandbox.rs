use std::env;
use std::process::Command;

pub fn run() {
    let build_dir = env::current_dir().unwrap();
    
    // Check if bwrap (Bubblewrap) is available
    let bwrap_available = Command::new("which")
        .arg("bwrap")
        .output()
        .ok()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if bwrap_available {
        println!("Running build in sandbox using bubblewrap...");
        
        // Running the build command inside a sandbox
        let status = Command::new("bwrap")
            .arg("--ro-bind")
            .arg(&build_dir)
            .arg("/src")
            .arg("--dev")
            .arg("/dev")
            .arg("--proc")
            .arg("/proc")
            .arg("--unshare-all")
            .arg("--die-with-parent")
            .arg("--chdir")
            .arg("/src")
            .arg("rmake")
            .arg("build")
            .status();
        
        match status {
            Ok(s) if s.success() => println!("Sandboxed build succeeded."),
            Ok(s) => println!("Sandboxed build failed with status: {}", s),
            Err(e) => println!("Failed to run bubblewrap: {}", e),
        }
    } else {
        println!("Bubblewrap not found. Running build without sandbox...");
        
        // Running the build command without the sandbox
        let status = Command::new("rmake")
            .arg("build")
            .status();
        
        match status {
            Ok(s) if s.success() => println!("Build succeeded."),
            Ok(s) => println!("Build failed with status: {}", s),
            Err(e) => println!("Failed to run build: {}", e),
        }
    }

    // Compatibility with makepkg flags (e.g., -si)
    let args: Vec<String> = env::args().collect();
    if args.contains(&"-si".to_string()) {
        println!("Running build and install (makepkg -si compatibility)...");

        // Handle build and install in a single step (if requested by the user)
        let build_status = Command::new("rmake")
            .arg("build")
            .status();
        
        match build_status {
            Ok(s) if s.success() => {
                let install_status = Command::new("rmake")
                    .arg("install")
                    .status();
                
                match install_status {
                    Ok(s) if s.success() => println!("Build and install succeeded."),
                    Ok(s) => println!("Install failed with status: {}", s),
                    Err(e) => println!("Failed to run install: {}", e),
                }
            }
            Ok(s) => println!("Build failed with status: {}", s),
            Err(e) => println!("Failed to run build: {}", e),
        }
    }
}

