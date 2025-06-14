# 📝 CHANGELOG
## v0.4.0 (2025-06-12)
- Refactored `resolve_and_install_deps` to use dynamic package lists and proper async return types
- Fully implemented recursive AUR + repo dependency resolution with deduplication
- `pkgb` now parsed and printed via `parse_pkgname_ver` to eliminate unused variable warnings
- Fixed Clippy-critical errors (E0308, E0271) blocking build; reduced total warnings to 56
- Updated core.rs to use clean `Box::pin(async move { ... })` with correct `Result<(), ()>` wrapping
- Verified successful full build and runtime execution for initial dynamic dep resolution
- Prep for 0.4.x line to focus on cleanup, lint zeroing, and extended config/testing support

## v0.3.0 (2025-06-12)
- Async/parallel install and upgrade flows (tokio, progress bars, no yay/paru fallback)
- Tap GPG verification, publisher.toml, key fetch
- CLI/TUI log output for install/upgrade
- Flatpak backend integration
- Rollback and backup system
- Config CLI, pinning, orphan detection
- Search/PKGBUILD caching (feature flag)
- Lua hook support (feature flag, planned)
- Improved error handling and diagnostics
- Docs: INSTALL.md, PUBLISHING.md, RELEASE_TESTS.md
- CI: clippy, fmt, test, build matrix
- Expanded CLI: audit, rollback, pin, TUI, tap management, and more

## v0.2.x and earlier
- Initial MVP, CLI install/remove/search, basic TUI stub
