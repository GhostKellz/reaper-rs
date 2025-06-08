use std::fs;

pub fn run() {
    let repo_dir = "repo";
    let entries = match fs::read_dir(repo_dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut pkgs: Vec<_> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".pkg.tar.zst"))
        .collect();
    pkgs.sort_by_key(|e| e.file_name());
    if pkgs.len() < 2 {
        println!("Need at least two packages in repo/ to diff.");
        return;
    }
    let a = &pkgs[pkgs.len() - 2];
    let b = &pkgs[pkgs.len() - 1];
    println!(
        "Diffing {} and {}",
        a.file_name().to_string_lossy(),
        b.file_name().to_string_lossy()
    );
    let status = std::process::Command::new("diffoscope")
        .arg(a.path())
        .arg(b.path())
        .status();
    match status {
        Ok(s) if s.success() => println!("Diff complete."),
        Ok(s) => println!("Diffoscope exited with status: {}", s),
        Err(e) => println!("Failed to run diffoscope: {}", e),
    }
}
