# ☠️ Reaper Package Manager

[![Arch Linux](https://img.shields.io/badge/platform-Arch%20Linux-1793d1?logo=arch-linux\&logoColor=white)](https://archlinux.org)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-000000?logo=rust\&logoColor=white)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-active-success?style=flat-square)](https://github.com/ghostkellz/reaper)
[![Build](https://img.shields.io/github/actions/workflow/status/ghostkellz/reaper/main.yml?branch=main)](https://github.com/ghostkellz/reaper/actions)
![Built with Clap](https://img.shields.io/badge/built%20with-clap-orange)
![License](https://img.shields.io/github/license/ghostkellz/reaper)

---

## Overview

Reaper is a blazing-fast, Rust-powered **meta package management toolkit** for Linux. It's a hybrid of two powerful tools:

* **Ghostbrew**: An AUR helper with TUI, rollback, GPG handling, and Flatpak integration.
* **GhostForge**: A modern makepkg replacement with TOML config support and CI-friendly features.

**Reaper unifies them into a single cross-distro CLI-first toolkit.**

---

## 🔧 Capabilities

### `reap`: AUR + Meta Package Manager (Ghostbrew module)

* Unified search across AUR, Pacman, and Flatpak
* Interactive TUI search and selection
* GPG key import & verification
* Rollback, batch installs
* Lua-based configuration
* Flatpak metadata integration

### `rmake`: The Makepkg Replacement (GhostForge module)

* Drop-in `makepkg` alternative
* Supports legacy PKGBUILD **and** modern `reaping.toml`
* Native Rust-based backend with no runtime deps
* Optional signature verification, hash checking
* Pluggable build backends (cargo, cmake, just, etc.)
* CLI audit/lint of PKGBUILD or TOML files
* Full dependency and hook management system

---

## 📦 Installation

### From Source

```bash
cargo install --path . # or reaper, depending on workspace layout
```

### From AUR (planned)

```bash
yay -S reaper-bin
```

---

## 🚀 Usage

### `reap` CLI

```bash
reap search <query>
reap install <pkg>
reap upgrade
reap rollback <pkg>
reap tui         # Launch interactive UI
```

### `rmake` (PKGBUILD or forge.toml)

```bash
rmake build
rmake install
rmake lint
rmake sign
rmake publish
```

---

## 📁 Configuration

### AUR Helper (Lua)

Path: `~/.config/reaper/brew.lua`

```lua
ignored_packages = {"package1", "package2"}
parallel = 4
```

### reaping.toml Example

```toml
name = "ghostctl"
version = "0.3.0"
author = "CK Technology LLC"
license = "MIT"
build = "cargo build --release"
install = "install -Dm755 target/release/ghostctl /usr/bin/ghostctl"
source = "https://github.com/ghostkellz/ghostctl/archive/v0.3.0.tar.gz"
checksum = "sha256:abcd1234..."
```

---

## 📚 Docs

* Full CLI Reference: `reap --help`, `rmake --help`
* [forge.toml Specification](https://github.com/ghostkellz/ghostforge/wiki/forge.toml-Spec)
* Migration Guide: [Switch from `makepkg`](https://github.com/ghostkellz/ghostforge/wiki/Migrating-from-Makepkg)
* See [`DOCS.md`](DOCS.md) for more

---

## 🤝 Contributing

PRs, issues, and feedback welcome.
See [`CONTRIBUTING.md`](CONTRIBUTING.md)

---

## 📜 License

MIT License © 2025 CK Technology LLC
See [`LICENSE`](LICENSE) for full details.

