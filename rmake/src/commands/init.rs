use crate::commands::ui;
use std::fs;

pub fn run() {
    let template = r#"name = "rmake"
version = "0.1.0"
author = "CK Technology LLC"
license = "MIT"
build = "cargo build --release"
install = "install -Dm755 target/release/rmake /usr/bin/rmake"
"#;

    // Check if 'forge.toml' already exists, not ghostforge.toml
    if fs::metadata("forge.toml").is_ok() {
        ui::print_error("forge.toml already exists in this directory.");
        return;
    }

    // Write the template to forge.toml
    match fs::write("forge.toml", template) {
        Ok(_) => ui::print_success("Created starter forge.toml."),
        Err(e) => ui::print_error(&format!("Failed to create forge.toml: {}", e)),
    }
}

