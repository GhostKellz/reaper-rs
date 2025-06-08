use crate::commands::ui;
use std::fs;
use std::path::Path;

pub fn run() {
    // Remove common build artifacts and temp directories
    let paths = [
        "target",
        "build",
        "pkg",
        "*.tar.zst",
        "*.tar",
        "*.pkg.tar.zst",
    ];
    for p in &paths {
        if p.contains("*") {
            // Glob pattern
            if let Ok(paths) = glob::glob(p) {
                for entry in paths.flatten() {
                    let _ = fs::remove_file(&entry);
                }
            }
        } else if Path::new(p).exists() {
            let _ = fs::remove_dir_all(p);
            let _ = fs::remove_file(p);
        }
    }
    ui::print_success("Cleaned build artifacts and temp files.");
}





