## 📦 Unified Package Management

### CLI Commands

* `reap search <pkg>`        - Unified search (Pacman, AUR, ChaoticAUR, Flatpak)
* `reap install <pkg>`       - Secure install (with PKGBUILD diff, sandbox test, and GPG validation)
* `reap upgrade`             - Upgrade all AUR/Flatpak packages
* `reap rollback <pkg>`      - Rollback a package to previous version
* `reap tui`                 - Launch interactive TUI
* `reap clean`               - Clean package cache
* `reap doctor`              - Run system diagnostics
* `reap gpg refresh`         - Refresh GPG keys

---

### Config Example

* `~/.config/reaper/brew.lua` – Lua config for ignored packages, parallelism, etc.

---

### See the README for more details.

---

☠ Built with paranoia by **GhostKellz**

