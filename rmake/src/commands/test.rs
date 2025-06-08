use crate::manifest::Manifest;
use std::process::Command;

pub fn run() {
    match Manifest::detect() {
        Some(manifest) => {
            if let Some(data) = &manifest.data {
                // Check if a "test" command exists in the manifest
                if let Some(test_cmd) = data.build.as_ref().filter(|s| s.contains("test")) {
                    println!("Running test command: {}", test_cmd);
                    
                    // Execute the test command depending on the OS
                    let status = if cfg!(target_os = "windows") {
                        Command::new("cmd").arg("/C").arg(test_cmd).status()
                    } else {
                        Command::new("sh").arg("-c").arg(test_cmd).status()
                    };

                    match status {
                        Ok(s) if s.success() => println!("Test succeeded."),
                        Ok(s) => println!("Test failed with status: {}", s),
                        Err(e) => println!("Failed to run test command: {}", e),
                    }
                } else {
                    // Provide a message when no test command is found in the manifest
                    println!(
                        "No test command found in manifest. (Example convention: build = 'cargo test')"
                    );
                }
            } else {
                // If manifest data is missing or incomplete
                println!("No manifest data available (PKGBUILD test not implemented).");
            }
        }
        None => {
            // Handle the case where no valid manifest is found
            println!("No PKGBUILD or forge.toml found in the current directory.");
        }
    }
}

