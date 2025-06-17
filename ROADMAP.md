# 🚀 REAP Roadmap

`reap` is a modern, async-first AUR helper written in Rust, built to replace tools like `yay` and `paru` — with better performance, extensibility, and a clean CLI/TUI experience.

---

## ✅ v0.5.0 Completed Features
- ✅ **Multi-Profile Package Management**: Switch between developer, gaming, minimal profiles with different backend priorities and settings
- ✅ **Package Verification & Trust Scoring**: Real-time security analysis with trust badges (🛡️ TRUSTED, ⚠️ CAUTION, ❌ UNSAFE)
- ✅ **Enhanced TUI with Live Monitoring**: Real-time build progress, trust scores in search results, system stats dashboard
- ✅ **Profile-Aware Installation**: Install behavior adapts to active profile (strict signatures, parallel jobs, backend order)
- ✅ **Security Analytics**: Vulnerability scanning, PKGBUILD analysis, publisher verification
- ✅ **Advanced UI Components**: Progress bars, trust badges, profile switcher, system monitoring
- ✅ **Manual PKGBUILD retrieval from AUR**: Fetch, parse, and analyze PKGBUILDs
- ✅ **Dependency resolution and conflict detection**: Advanced circular dependency and conflict analysis
- ✅ **Interactive prompts**: Confirm removals, edit PKGBUILDs with safety checks
- ✅ **Interactive `--diff` for PKGBUILDs**: TUI/CLI diff viewer with colored output
- ✅ **Package list, queue manager, and PKGBUILD diff viewer in TUI**: Comprehensive package management interface
- ✅ **Search, install, queue, review updates interactively**: Full interactive workflow
- ✅ **Real-time log pane, diff viewer for PKGBUILDs**: Live monitoring and analysis
- ✅ **AUR rating system with star emojis**: ⭐⭐⭐⭐⭐ ratings and community reviews
- ✅ **Publisher verification badge in TUI queue**: Security indicators in package queue
- ✅ **Audit mode to show upstream changes**: Enhanced security analysis
- ✅ **Clean build without warnings**: All code warnings resolved for v0.5.0 release
- ✅ **Complete release package**: PKGBUILD, install script, build automation ready

## 🆕 v0.4.0 Highlights  
- Refactored `resolve_and_install_deps` to use dynamic package lists and proper async return types
- Fully implemented recursive AUR + repo dependency resolution with deduplication
- `pkgb` now parsed and printed via `parse_pkgname_ver` to eliminate unused variable warnings
- Fixed Clippy-critical errors (E0308, E0271) blocking build; reduced total warnings significantly
- Updated core.rs to use clean `Box::pin(async move { ... })` with correct `Result<(), ()>` wrapping

## 🆕 v0.3.0-rc Highlights
- End-to-end async/parallel install and upgrade flows (no yay/paru fallback)
- GPG workflows: refresh, import, verify, check key, set keyserver (with clear feedback)
- Flatpak install/upgrade fully integrated
- Shell-based hooks for all lifecycle events (pre_install, post_install, etc.)
- Minimal rollback support: restores PKGBUILD or cleans up failed install dirs
- Improved error handling and logging throughout
- Docs and tests for config, GPG, hooks, Flatpak, rollback

---

## ✅ Minimum Viable Product (MVP)

Basic CLI functionality powered by `std::process::Command`:

- [x] `reap -S <pkg>` – Install AUR or repo package (AUR + Flatpak install via detect_source())
- [x] `reap -R <pkg>` – Remove a package
- [x] `reap -Syu` – Sync and upgrade packages
- [x] `reap -U <file>` – Install local `.zst` or `.pkg.tar.zst`
- [x] `reap -Ss <term>` – Search AUR (via JSON-RPC)
- [x] Async execution using `tokio` (parallel_install implemented)
- [x] GPG verification with PKGBUILD.sig and publisher.toml (secure tap installs)
- [x] --insecure and --gpg-keyserver CLI options for tap installs
- [x] Publisher verification and CLI/TUI log output for tap installs
- [x] Basic error handling and logging
- [x] No longer relies on yay/paru (fallback removed)
- [x] `reap --rollback` – now wired to `hooks::on_rollback()` for tracking reversions.
- [x] `reap --upgradeall` – upgraded to call `aur::upgrade_all()` with summary reporting.
- [x] Flatpak support – install + upgrade fully integrated.
- [x] `reap --tui` – launches the async TUI (stub or basic UI).
- [x] `reap --pin` – pins packages to exclude from upgrades.
- [x] `reap --clean` – cleans cache or temp files.
- [x] `reap doctor` – performs basic health check and config audit.
- [x] CLI integration for all GPG subcommands (import, show, verify, check, set-keyserver)
- [x] Tap-based source support (add/list remote AUR repos)
- [x] Full CLI wiring for Flatpak backend (search, install, upgrade, audit)
- [x] Implemented `handle_search()` and wired AUR search subcommand
- [x] `reap doctor` now provides async config validation
- [x] `reap backup` – backs up config from `~/.config/reap` to `/var/lib/reaper/backups/config/`
- [x] Short flag cleanup – resolved CLI flag conflicts for `-S`, `-Q`, `-U`, etc.
- [x] Backup and rollback scaffolding (`utils::backup_config`, `rollback_pkgbuild`)
- [x] Documentation for secure tap publishing (PUBLISHING.md)

---

## 🔧 Near-Term Enhancements

More control, fewer dependencies:

- [x] Flatpak backend CLI fully wired
- [x] Add interactive `--edit` flow for PKGBUILDs
- [x] Makepkg integration: `makepkg -si` (via `utils::build_pkg`)
- [x] Secure tap install UX/logging improvements
- [x] Publisher verification badge in CLI/TUI (log output)
- [ ] Add interactive `--diff` for PKGBUILDs (TUI/CLI diff viewer)
- [ ] Move hooks to support Lua/custom external scripts (stretch)
- [ ] Manual PKGBUILD retrieval from AUR
- [ ] Dependency resolution and conflict detection
- [ ] Interactive prompts: confirm removals, edit PKGBUILDs

---

## 🧰 Intermediate Features

Modular design, performance improvements:

- [ ] Pluggable backends (`reap --backend aur`, `--backend flatpak`, `--backend tap`)
- [x] Caching system for AUR search results and PKGBUILDs (partial scaffolding exists)
- [x] Persistent config (TOML/YAML under `~/.config/reap`)
- [ ] Logging and audit mode (`--log`, `--audit`)
- [x] Async install queues with progress bars
- [ ] Integrate `run_hook()` for user-defined lifecycle scripting (pre/post install)
- [ ] Modular `utils::print_search_results()` across CLI and TUI

### 🔁 Rollback & Pinning

- [x] Rollback hook support (triggered post-install)
- [x] Configurable pinned packages (`~/.config/reap/pinned.toml`)

---

## 🎨 User Experience (TUI)

Optional terminal UI using `ratatui` or similar:

- [x] TUI mode (`reap tui`) now launches an interactive CLI menu
- [ ] Add package list, queue manager, and PKGBUILD diff viewer to TUI
- [ ] Search, install, queue, review updates interactively
- [ ] Real-time log pane, diff viewer for PKGBUILDs
- [ ] Publisher verification badge in TUI queue (planned)

---

## 🔐 Security & Validation

Built-in trust and transparency:

- [x] GPG key management (`--import-gpg`, `--check-gpg`, `--set-keyserver`)
- [x] Package rollback (`--rollback`)
- [x] Audit for GPG trust level, optional deps (via `get_trust_level`)
- [x] GPG fallback key import if PKGBUILD signature is missing key
- [x] Async keyserver health check (`check_keyserver`) (via CLI scaffold)
- [x] Keyserver validation and override
- [ ] Audit mode to show upstream changes

---

## 🧪 Stretch & Experimental Ideas

Long-term exploration:

- [ ] Multi-distro support (e.g. `reap` inside containers for Debian/Fedora)
- [ ] AUR diff audit (compare PKGBUILD changes)
- [ ] Reap script mode: install from JSON manifest
- [ ] Headless mode for CI/CD systems
- [ ] WASM-based sandboxing for PKGBUILD parsing
- [ ] Lua scripting support for install hooks
- [ ] PKGBUILD linting / schema validation

---

## 💬 Community & Contribution

- [x] `reap --version` and `--about` with repo link
- [x] CONTRIBUTING.md for onboarding devs
- [ ] Plugin system for power users
- [ ] Discord community

---

## 📅 Status

Current focus: TUI install queue, PKGBUILD diff viewer, plugin/hook system, audit/logging mode, and further UX improvements for secure tap installs.

---

> Built with 🦀 Rust, 💻 by @ghostkellz

---

## TODO

- [x] TODO(v0.3): Minimal rollback support (PKGBUILD restore, temp cleanup)
- [ ] TODO(v0.4): TUI colored status for verification, [fast]/[strict] badges, source highlighting
- [ ] TODO(v0.4): Remove or refactor dead code, legacy hooks, and unused cache logic
- [ ] TODO: Advanced Lua scripting for hooks

## v0.4 / v1.0 TODOs
- Tap publishing (CLI + docs)
- Flatpak audit/signing
- Multi-profile config support
- Plugin/hook system
- TUI colored status, badges, source highlighting
- Audit/logging mode
- Benchmarks and performance tracking

## 🎯 v0.6.0 Planned Features
- [ ] **Package snapshots and rollback**: System-level package state management
- [ ] **Cross-distro package translation**: Translate package names between distributions
- [ ] **Plugin system**: Extensible architecture with Rust/WASM plugins
- [ ] **AI-powered package recommendations**: Smart suggestions based on usage patterns
- [ ] **Container environment support**: Reproducible development environments
- [ ] **Real-time vulnerability database**: CVE integration and security alerts
- [ ] **Package marketplace**: Community ratings, reviews, and recommendations
- [ ] **Advanced caching system**: Intelligent build cache and binary cache
- [ ] **Network optimization**: Parallel downloads and mirror selection
- [ ] **Mobile TUI**: Responsive interface for smaller terminals

## 🔮 Future Vision (v0.7.0+)
- [ ] **Nix/Guix backend support**: Integration with functional package managers
- [ ] **Distributed package building**: Community build farm
- [ ] **Package signing infrastructure**: Enhanced security with signing
- [ ] **Integration with system package managers**: apt, dnf, zypper support
- [ ] **Package analytics and telemetry**: Usage statistics and optimization
- [ ] **Advanced dependency solver**: SAT-based dependency resolution
- [ ] **Package virtualization**: Isolated package environments
- [ ] **Cloud synchronization**: Profile and settings sync across devices
