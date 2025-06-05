# ☠️ reap — Secure Arch Build, Install, and Sandbox System

## 🧩 Unified Package Management

### CLI Commands

- `reap search <pkg>` — Unified search (Pacman, AUR, ChaoticAUR, Flatpak)
- `reap install <pkg>` — Secure install with PKGBUILD diff, sandbox test, and GPG validation
- `reap upgrade` — Upgrade all sources (Pacman, AUR, Flatpak) with rollback + parallel jobs
- `reap rollback <pkg>` — Roll back to previous package state (backup snapshot)
- `reap tap <repo>` — Add custom Git-based PKGBUILD repo
- `reap tui` — Launch interactive TUI interface
- `reap doctor` — Run environment diagnostics for AUR, sandbox, GPG, and missing deps
- `reap completion <shell>` — Generate shell completions for bash/zsh/fish

---

## 🧪 Sandbox Testing & Security

### CLI Commands

- `reap test <pkg>` — Install + verify a package inside an ephemeral container
- `reap start` — Boot sandbox VM/container manually
- `reap shell` — Drop into live sandbox shell
- `reap trace` — Monitor file/network access during test
- `reap snapshot` — Save current sandbox state for later reuse or rollback
- `reap diff` — Show file diff from clean state vs after package install
- `reap logs` — Show test output/logs from sandbox
- `reap stop` — Stop and remove sandbox

### Sandbox Backends

- 🧱 LXC or systemd-nspawn (default)
- 🧼 Bubblewrap / Firejail fallback
- 👤 Works rootless or as root
- 🔐 Used automatically before installs (configurable)

---

## 🛠 Local Build System (rmake)

### CLI Commands

- `rmake init` — Scaffold a new PKGBUILD project layout
- `rmake build` — Build with `makepkg` + custom hooks
- `rmake install` — Install local package (`pacman -U`)
- `rmake release` — Sign, compress, and publish to repo
- `rmake clean` — Clean dist + work dirs
- `rmake lint` — Run PKGBUILD validator
- `rmake graph` — Generate package dependency graph

---

## 💻 TUI Mode

### Keybindings

- `/` — New search
- `↑ / ↓` — Navigate results
- `Space` — Select for install
- `Enter` — Install selected or focused
- `d` — Show PKGBUILD and deps
- `l` — Toggle log pane
- `h` — Help popup
- `q` — Quit

### Features

- 🔍 Unified view (Pacman, AUR, Flatpak, ChaoticAUR)
- ✅ GPG + PKGBUILD audit with diff preview
- 🧪 Pre-install sandbox testing per policy
- ⚡ Parallel upgrades with logging + rollback
- 📦 Flatpak sandbox visibility, votes, maintainer info
- 📜 Lua-configurable logic, hooks, and source priority
- 🧾 Real-time logs + error stream in TUI

---

## 🔧 See Also

- `README.md` for config options, Lua API, backend setup
- `reap doctor` to verify your system is ready
- `rmake` for local development, testing, and release automation

---

💀 Built with paranoia by **GhostKellz**

