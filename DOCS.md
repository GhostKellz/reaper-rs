# 📘 Reaper Documentation

> **Reaper** is a secure, modular package manager and AUR helper for Arch Linux, designed to safely install, audit, sandbox, and rollback packages across the AUR, Flatpak, Pacman, and ChaoticAUR ecosystems.

---

## 🔧 Overview

Reaper unifies:

* **System package management**
* **AUR and custom Git repo support**
* **Flatpak integration**
* **Sandboxed testing environments**
* **PKGBUILD auditing and linting**
* **Rollback and snapshot support**
* **TUI-based batch installs**

Reaper is built for developers, hackers, and sysadmins who demand verifiability, repeatability, and modularity.

---

## 🛠 Architecture

### Core Binary

* `reap` — Meta package manager (search/install/upgrade/sandbox/test)

### Components

* **TUI** — `reap tui` provides a fuzzy-powered interactive interface
* **Sandbox Engine** — Ephemeral VMs/containers for isolated testing
* **Lua Hook Engine** — Lifecycle scripting for custom behavior

### Supported Sources

* `pacman`
* `aur` (via internal or `yay`-style logic)
* `chaotic-aur` (optional)
* `flatpak`
* `custom` (via `reap tap`)

---

## 🔐 Security Features

* 💾 PKGBUILD diff viewer
* 🔐 GPG key auto-fetch + verification
* 🧪 Pre-install sandbox testing
* 🕵️ File/network access tracing
* 📀 Rollback support via snapshotting
* 🔍 Dependency graph auditing

---

## ⚙️ Configuration

Reaper reads configuration from:

* `~/.config/reap/config.lua`
* System-wide defaults from `/etc/reap/`

### Example `config.lua`:

```lua
return {
  prefer = { "pacman", "aur", "flatpak" },
  sandbox = {
    enable = true,
    backend = "lxc",
  },
  hooks = {
    pre_install = "lua ./hooks/pre.lua",
    post_build = "lua ./hooks/post.lua"
  },
  ignored_packages = {
    "nvidia-beta",
    "steam"
  }
}
```

---

## 🧪 Sandbox Testing

Reaper uses secure sandboxes to test packages **before** installation:

Supported backends:

* `lxc` (default, if available)
* `systemd-nspawn`
* `bubblewrap`
* `firejail`

Features:

* Snapshot state
* File/network tracing
* Diff against clean root
* Ephemeral installs

---

## 📚 Related Files

* [README.md](./README.md) – Project overview
* [COMMANDS.md](./COMMANDS.md) – Full CLI reference
* [CONTRIBUTING.md](./CONTRIBUTING.md)

---

☠️ Built with paranoia by **GhostKellz**

