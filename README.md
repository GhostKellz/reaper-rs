[![Arch Linux](https://img.shields.io/badge/platform-Arch%20Linux-1793d1?logo=arch-linux&logoColor=white)](https://archlinux.org)
[![Made with Rust](https://img.shields.io/badge/made%20with-Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Status](https://img.shields.io/badge/status-active-success?style=flat-square)](https://github.com/ghostkellz/reaper)
[![Build](https://img.shields.io/github/actions/workflow/status/ghostkellz/reaper/main.yml?branch=main)](https://github.com/ghostkellz/reaper/actions)
![Built with Clap](https://img.shields.io/badge/built%20with-clap-orange)
![License](https://img.shields.io/github/license/ghostkellz/reaper)

# ☠️ Reaper Package Manager

---

## 📄 Overview

**Reaper** is a blazing-fast, Rust-powered **AUR helper and meta package manager** for Arch Linux. It is designed for paranoid Arch users, power packagers, and automation-first workflows.

---

## 🔧 Capabilities

* Unified search: AUR, Pacman, Flatpak, ChaoticAUR
* Interactive TUI installer
* GPG key importing + PKGBUILD auditing
* Rollback system & multi-package upgrades
* Flatpak integration with metadata
* Lua-configurable backend logic

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

```bash
reap search <pkg>
reap install <pkg>
reap upgrade
reap rollback <pkg>
reap tui
```

---

## 📂 Config Example

### `~/.config/reaper/brew.lua`

```lua
ignored_packages = {"spotify", "google-chrome"}
parallel = 8
```

---

## 📚 Documentation

* [Command Reference](COMMANDS.md)
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

