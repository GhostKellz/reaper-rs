[![Arch Linux](https://img.shields.io/badge/platform-Arch%20Linux-1793d1?logo=arch-linux\&logoColor=white)](https://archlinux.org)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-000000?logo=rust\&logoColor=white)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-active-success?style=flat-square)](https://github.com/ghostkellz/reaper)
[![Build](https://img.shields.io/github/actions/workflow/status/ghostkellz/reaper/main.yml?branch=main)](https://github.com/ghostkellz/reaper/actions)
![Built with Clap](https://img.shields.io/badge/built%20with-clap-orange)
![License](https://img.shields.io/github/license/ghostkellz/reaper)

# ☠️ Reaper Package Manager

---

## 📄 Overview

**Reaper** is a blazing-fast, Rust-powered **meta package management toolkit** for Arch Linux. It merges two specialized tools:

* **Ghostbrew**: A secure AUR helper with rollback, GPG verification, sandboxing, and Flatpak support.
* **GhostForge**: A modern `makepkg` replacement with TOML config, CI integration, and dependency control.

Now bundled together in one modular CLI toolkit, **Reaper** is designed for paranoid Arch users, power packagers, and automation-first workflows.

---

## 🔧 Capabilities

### `reap`: Secure AUR & Meta Package Manager (Ghostbrew)

* Unified search: AUR, Pacman, Flatpak, ChaoticAUR
* Interactive TUI installer
* GPG key importing + PKGBUILD auditing
* Rollback system & multi-package upgrades
* Flatpak integration with metadata
* Lua-configurable backend logic

### `rmake`: Makepkg Replacement (GhostForge)

* Drop-in alternative to `makepkg`
* Supports both `PKGBUILD` and `reaping.toml`
* Fully written in Rust, zero runtime dependencies
* Hookable lifecycle with custom scripts
* Reproducible, signed builds
* Auto-release, packaging, and CI hooks

### `grim`: Rust Crate Manager (Cargo++ Concept)

* Secure wrapper for `cargo` workflows
* Sandboxed test & install
* Offline dependency fetch & integrity validation
* Audit toolchain and dependency graph
* Integrates directly into Reaper CLI family

---

## 📦 Installation

### Build from Source

```bash
cargo install --path .
```

### AUR (planned)

```bash
yay -S reaper-bin
```

---

## 🚀 Usage

### AUR & Package Management (reap)

```bash
reap search <pkg>
reap install <pkg>
reap upgrade
reap rollback <pkg>
reap tui
```

### PKGBUILD Build System (rmake)

```bash
rmake init
rmake build
rmake install
rmake lint
rmake release
```

### Secure Rust Tooling (grim)

```bash
grim build
grim install
grim audit
grim shell
```

---

## 📂 Config Examples

### `~/.config/reaper/brew.lua`

```lua
ignored_packages = {"spotify", "google-chrome"}
parallel = 8
```

### `reaping.toml`

```toml
name = "phantomdns"
version = "0.1.0"
author = "GhostKellz"
license = "MIT"
source = "https://github.com/ghostkellz/phantomdns/archive/v0.1.0.tar.gz"
checksum = "sha256:abc123..."
build = "cargo build --release"
install = "install -Dm755 target/release/phantomdns /usr/bin/phantomdns"
```

---

## 📚 Documentation

* [Command Reference](COMMANDS.md)
* [forge.toml Spec](https://github.com/ghostkellz/ghostforge/wiki/forge.toml-Spec)
* [Migration from ](https://github.com/ghostkellz/ghostforge/wiki/Migrating-from-Makepkg)[`makepkg`](https://github.com/ghostkellz/ghostforge/wiki/Migrating-from-Makepkg)
* `reap doctor` – validate your environment

---

## 😎 Contributing

Open to PRs, bugs, ideas, and flames. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for style and module conventions.

---

## 📜 License

MIT License © 2025 [CK Technology LLC](https://github.com/ghostkellz)
See [`LICENSE`](LICENSE) for full terms.

---

☠️ Built with paranoia by **GhostKellz**

