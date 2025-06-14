# 📝 CHANGELOG

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
