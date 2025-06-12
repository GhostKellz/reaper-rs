# 🚀 REAP Roadmap

`reap` is a modern, async-first AUR helper written in Rust, built to replace tools like `yay` and `paru` — with better performance, extensibility, and a clean CLI/TUI experience.

---

## ✅ Minimum Viable Product (MVP)

Basic CLI functionality powered by `std::process::Command`:

- [x] `reap -S <pkg>` – Install AUR or repo package (AUR + Flatpak install via detect_source())
- [x] `reap -R <pkg>` – Remove a package
- [x] `reap -Syu` – Sync and upgrade packages
- [x] `reap -U <file>` – Install local `.zst` or `.pkg.tar.zst`
- [x] `reap -Ss <term>` – Search AUR (via JSON-RPC)
- [x] Async execution using `tokio` (parallel_install implemented)
- [x] GPG verification with PKGBUILD.sig
- [x] Basic error handling and logging
- [x] No longer relies on yay/paru (fallback removed)
- [x] `reap --rollback` – now wired to `hooks::on_rollback()` for tracking reversions.
- [x] `reap --upgradeall` – upgraded to call `aur::upgrade_all()` with summary reporting.
- [x] Flatpak support – install + upgrade fully integrated.
- [x] `reap --tui` – launches the async TUI (stub or basic UI).
- [x] `reap --pin` – pins packages to exclude from upgrades.
- [x] `reap --clean` – cleans cache or temp files.
- [x] `reap doctor` – performs basic health check and config audit.
- [x] CLI integration for all GPG subcommands (import, show, verify, check)
- [x] Tap-based source support (add/list remote AUR repos)
- [x] Full CLI wiring for Flatpak backend (search, install, upgrade, audit)
- [x] Implemented `handle_search()` and wired AUR search subcommand
- [x] `reap doctor` now provides async config validation

---

## 🔧 Near-Term Enhancements

More control, fewer dependencies:

- [x] Flatpak backend CLI fully wired
- [ ] Add interactive `--edit` and `--diff` for PKGBUILDs
- [ ] Remove or finalize legacy `.deb` stubs
- [ ] Move hooks to support Lua/custom external scripts (stretch)
- [ ] Manual PKGBUILD retrieval from AUR
- [ ] Makepkg integration: `makepkg -si` (**TODO**)
- [ ] Dependency resolution and conflict detection
- [ ] Interactive prompts: confirm removals, edit PKGBUILDs

---

## 🧰 Intermediate Features

Modular design, performance improvements:

- [ ] Pluggable backends (`reap --backend aur`, `--backend flatpak`) (**TODO**)
- [ ] Caching system for AUR search results and PKGBUILDs (partial scaffolding exists)
- [ ] Persistent config (TOML/YAML under `~/.config/reap`)
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
- [ ] Still in early stage; UI will be extended with package list, queue, and diff viewer
- [ ] Search, install, queue, review updates interactively
- [ ] Real-time log pane, diff viewer for PKGBUILDs

---

## 🔐 Security & Validation

Built-in trust and transparency:

- [x] GPG key management (`--import-gpg`, `--check-gpg`)
- [x] Package rollback (`--rollback`)
- [ ] Audit for GPG trust level, optional deps (**TODO**)
- [ ] Keyserver validation
- [ ] Audit mode to show upstream changes
- [ ] Async keyserver health check (`check_keyserver`)
- [ ] Wire GPG fallback key import on missing sigs

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

- [ ] `reap --version` and `--about` with repo link
- [ ] CONTRIBUTING.md for onboarding devs
- [ ] Plugin system for power users
- [ ] Discord community

---

## 📅 Status

Current focus: Final GPG integration, Flatpak/AUR parity, config polish, and search result caching. TUI will evolve next.

---

> Built with 🦀 Rust, 💻 by @ghostkellz
