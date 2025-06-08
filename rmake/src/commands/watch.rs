use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::channel;

pub fn run() {
    println!("Watching for file changes. Press Ctrl+C to stop.");
    let (tx, rx) = channel();
    let mut watcher =
        RecommendedWatcher::new(tx, Config::default()).expect("Failed to create watcher");
    watcher
        .watch(Path::new("."), RecursiveMode::Recursive)
        .expect("Failed to watch .");
    loop {
        match rx.recv() {
            Ok(Ok(_event)) => {
                println!("Change detected. Rebuilding...");
                let status = Command::new("rmake").arg("build").status(); // Change 'forge' to 'rmake'
                match status {
                    Ok(s) if s.success() => println!("Auto-build succeeded."),
                    Ok(s) => println!("Auto-build failed with status: {}", s),
                    Err(e) => println!("Failed to run build: {}", e),
                }
            }
            Ok(Err(e)) => println!("watch error: {:?}", e),
            Err(e) => println!("channel error: {:?}", e),
        }
    }
}

