## 📦 Unified Package Management

### CLI Commands

* `reap -S <pkg>`         - Install AUR or Flatpak package (auto-detects source)
* `reap -R <pkg>`         - Remove a package
* `reap -Ss <term>`       - Search AUR (JSON-RPC)
* `reap -Syu`             - Sync and upgrade all packages (AUR + Flatpak)
* `reap -U <file>`        - Install local `.zst` or `.pkg.tar.zst` package
* `reap tui`              - Launch interactive TUI
* `reap clean`            - Clean package cache
* `reap doctor`           - Run system diagnostics
* `reap gpg refresh`      - Refresh GPG keys
* `reap rollback <pkg>`   - Rollback a package to previous version

---

### Config Example

* `~/.config/reaper/brew.lua` – Lua config for ignored packages, parallelism, etc.

---

### See the README for more details.

---

☠ Built with paranoia by **GhostKellz**

