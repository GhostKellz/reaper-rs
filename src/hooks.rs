pub fn on_install(pkg: &str) {
    // TODO: Wire this into CLI flow in core::handle_cli()
    println!("[reap][hook] Installed {}", pkg);
}

pub fn on_rollback(pkg: &str) {
    // TODO: Wire this into CLI flow in core::handle_cli()
    println!("[reap][hook] Rollback for {}", pkg);
}
