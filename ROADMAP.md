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

---

## 🔧 Near-Term Enhancements

More control, fewer dependencies:

- [ ] Drop reliance on any external AUR helpers (done)
- [ ] Manual PKGBUILD retrieval from AUR
- [ ] Makepkg integration: `makepkg -si` (**TODO**)
- [ ] Support Flatpak as backend (implemented)
- [ ] `.deb` backend temporarily removed
- [ ] Dependency resolution and conflict detection
- [ ] Interactive prompts: confirm removals, edit PKGBUILDs

---

## 🧰 Intermediate Features

Modular design, performance improvements:

- [ ] Pluggable backends (`reap --backend aur`, `--backend flatpak`) (**TODO**)
- [ ] Caching: PKGBUILDs, metadata, search results
- [ ] Persistent config (TOML/YAML under `~/.config/reap`)
- [ ] Logging and audit mode (`--log`, `--audit`)
- [x] Async install queues with progress bars

---

## 🎨 User Experience (TUI)

Optional terminal UI using `ratatui` or similar:

- [ ] `reap tui` – Full TUI interface
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

---

## 🧪 Stretch & Experimental Ideas

Long-term exploration:

- [ ] Multi-distro support (e.g. `reap` inside containers for Debian/Fedora)
- [ ] AUR diff audit (compare PKGBUILD changes)
- [ ] Reap script mode: install from JSON manifest
- [ ] Headless mode for CI/CD systems
- [ ] WASM-based sandboxing for PKGBUILD parsing

---

## 💬 Community & Contribution

- [ ] `reap --version` and `--about` with repo link
- [ ] CONTRIBUTING.md for onboarding devs
- [ ] Plugin system for power users
- [ ] Discord community

---

## 📅 Status

Current focus: **MVP completion and transition away from yay/paru dependencies.**  
Target: **Self-contained, reliable AUR helper with fast, async-native execution.**

---

> Built with 🦀 Rust, 💻 by @ghostkellz
