use crate::manifest::Manifest;

pub fn run() {
    match Manifest::detect() {
        Some(manifest) => {
            if let Some(data) = &manifest.data {
                if let Some(license) = &data.license {
                    if license.to_lowercase().contains("gpl")
                        || license.to_lowercase().contains("mit")
                    {
                        println!("License is open source: {}", license);
                    } else {
                        println!("License may not be open source: {}", license);
                    }
                } else {
                    println!("No license field found in manifest.");
                }
            } else {
                println!("No manifest data available (PKGBUILD audit not implemented).");
            }
        }
        None => {
            println!("No PKGBUILD or brew.toml found in the current directory.");
        }
    }
}
