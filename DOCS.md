# 📘 Reap Documentation

> **Reap** is a secure, modular package manager and local build system for Arch Linux, designed to safely install, audit, sandbox, and rollback packages across the AUR, Flatpak, Pacman, and ChaoticAUR ecosystems.
> It replaces `ghostbrew` with a hardened and extensible CLI-first toolset.

---

## 🔧 Overview

Reap unifies:

* **System package management**
* **AUR and custom repo support**
* **Flatpak integration**
* **Sandboxed test environments (pre-install)**
* **PKGBUILD auditing and linting**
* **Rollback and snapshot support**
* **TUI-based batch installs**
* **Developer-focused local build tooling via `rmake`**

Reap is ideal for developers, tinkerers, and IT professionals seeking a hardened Arch ecosystem with repeatable, reversible, and inspectable operations.

---

## 🛠 Architecture

### Core Components

* **`reap` CLI** — Main interface (search/install/upgrade/sandbox/test)
* **`reap tui`** — Interactive fuzzy-powered installer
* **`rmake`** — Local PKGBUILD builder for projects and repos
* **Sandbox Daemon** — Ephemeral VM/container to test builds (LXC, nspawn, firejail, or bwrap)
* **Lua Hooks Engine** — Custom lifecycle logic for install/build/test

### Sources Supported

* `pacman`
* `aur` (via `yay`-style logic)
* `chaotic-aur` (if enabled)
* `flatpak`
* `custom` (via `reap tap`)

---

## 🔐 Security Features

* **PKGBUILD audit**: Shows diffs before install
* **GPG verification**: Verifies sources and signatures
* **Pre-install sandbox**: All packages can be test-installed into an ephemeral environment
* **Rollback support**: Every install/upgrade creates a fallback snapshot
* **Dependency graphing**: Full resolution display for transparency

---

## ⚙️ Configuration

Reap reads config from:

* `~/.config/reap/config.lua`
* Global defaults in `/etc/reap/`

### Example config.lua:

```lua
return {
  prefer = { "pacman", "chaotic-aur", "aur", "flatpak" },
  sandbox = {
    enable = true,
    backend = "lxc", -- or "bwrap", "nspawn", "firejail"
  },
  hooks = {
    pre_install = "lua ./hooks/pre.lua",
    post_build = "lua ./hooks/post.lua",
  },
  ignored_packages = {
    "steam",
    "nvidia-beta",
  }
}
```

---

## 📦 Local PKGBUILD Projects

Reap includes `rmake` to simplify working with custom packages:

* `rmake init` scaffolds a build folder
* `rmake build` uses `makepkg` and captures logs
* `rmake release` signs and publishes to Git-based or file-based repos
* `rmake graph` visualizes complex dependency chains

---

## 🧪 Sandbox Modes

Before installing, `reap` can test a package in a:

* `lxc` container (default if LXC is installed)
* `systemd-nspawn` container
* `bubblewrap` rootless sandbox
* `firejail` (fallback if others unavailable)

Useful for catching:

* Install-time breakage
* File conflicts
* Suspicious scripts
* Networking or filesystem access

---

## 📚 Related Files

| File          | Purpose                            |
| ------------- | ---------------------------------- |
| `COMMANDS.md` | CLI reference and keybindings      |
| `README.md`   | Intro, goals, install instructions |
| `config.lua`  | Main user configuration file       |
| `PKGBUILD`    | For building `reap` itself         |
| `rmake.toml`  | Optional overrides for `rmake`     |
| `sandbox.log` | Last test install log              |

---

## 🧠 Philosophy

* **Security-first**: Don’t trust—verify.
* **Reversible**: Every action should be rollable.
* **Modular**: All features configurable or replaceable.
* **Unified UX**: One tool, one UX, all sources.
* **Scriptable**: Lua hooks power most of the internal automation.

---

## 📡 Contributing

Want to improve or fork `reap`? Use `rmake` and clone from:

```bash
git clone https://github.com/ghostkellz/reap
cd reap
rmake build
```

Contributions welcome via PR, patch, or issue!

---

© 2025 — CK Technology LLC • MIT License

