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

* Unified search: AUR, Pacman, Flatpak, ChaoticAUR, ghostctl-aur, custom binary repos
* Interactive TUI installer with multi-source search and install queue
* GPG key importing, PKGBUILD diff and auditing, trust level reporting
* Rollback system, backup/restore, and multi-package upgrades
* Flatpak integration with metadata and audit
* Orphan detection and removal (AUR/pacman)
* Dependency/conflict resolution with --resolve-deps
* Binary-only and repo selection (e.g. --repo=ghostctl-aur)
* Lua-configurable backend logic and plugin/hook support
* Search and PKGBUILD caching for speed

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
reap search <pkg>           # Search AUR, Flatpak, Pacman, ChaoticAUR, ghostctl-aur
reap install <pkg>          # Install from any source (auto-detect)
reap install <pkg> --repo=ghostctl-aur  # Force binary repo
reap install <pkg> --binary-only        # Only install from binary repo
reap upgrade                # Upgrade all packages
reap rollback <pkg>         # Rollback a package
reap orphan [--remove]      # List/remove orphaned AUR/pacman packages
reap backup                 # Backup config
reap diff <pkg>             # Show PKGBUILD diff before install/upgrade
reap pin <pkg>              # Pin a package/version
reap clean                  # Clean cache
reap doctor                 # System/config health check
reap tui                    # Interactive TUI
reap gpg ...                # GPG key management
reap flatpak ...            # Flatpak management
reap tap ...                # Tap repo management
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
* [Full Docs](DOCS.md)
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

