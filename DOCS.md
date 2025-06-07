# 📘 Reap Documentation

> **Reap** is a secure, modular package manager and local build system for Arch Linux, designed to safely install, audit, sandbox, and rollback packages across the AUR, Flatpak, Pacman, and ChaoticAUR ecosystems.
> It replaces `ghostbrew` and `ghostforge` with a hardened and extensible CLI-first toolset.

---

## 🔧 Overview

Reap unifies:

* **System package management**
* **AUR and custom Git repo support**
* **Flatpak integration**
* **Sandboxed testing environments**
* **PKGBUILD auditing and linting**
* **Rollback and snapshot support**
* **TUI-based batch installs**
* **Rust-native developer tooling (`rmake`, `grim`)**

Reap is built for developers, hackers, and sysadmins who demand verifiability, repeatability, and modularity.

---

## 🛠 Architecture

### Core Binaries

* `reap` — Meta package manager (search/install/upgrade/sandbox/test)
* `rmake` — PKGBUILD + TOML builder with CI hooks
* `grim` — Secure Rust package bootstrapper (Cargo++)

### Components

* **TUI** — `reap tui` provides a fuzzy-powered interactive interface
* **Sandbox Engine** — Ephemeral VMs/containers for isolated testing
* **Lua Hook Engine** — Lifecycle scripting for custom behavior
* **Forge Layer** — Optional `reaping.toml` support for modern builds

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

Reap reads configuration from:

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

## 📆 Local Projects (`rmake`)

Reap includes `rmake` to streamline package development:

* `rmake init` – Scaffold a new package
* `rmake build` – Build using makepkg-compatible logic
* `rmake install` – Local install with `pacman -U`
* `rmake lint` – Check PKGBUILD or reaping.toml
* `rmake release` – Sign + publish to repo
* `rmake graph` – Visualize dependencies

---

## 🧪 Sandbox Testing

Reap uses secure sandboxes to test packages **before** installation:

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

## 🧰 Rust Project Support (`grim`)

`grim` is a secure wrapper for managing and deploying Rust projects.

### Key Features

* 🧱 Sandboxed builds + testing
* 🔐 Audit crates and dependencies
* 📆 Secure install of Rust binaries (system-wide or user)
* 🌐 Offline fetch + lockfile verification
* 🧪 Compatible with Reap’s sandbox engine

### CLI Commands

```bash
grim build       # Compile crate with checks
grim install     # Install binary + metadata
grim test        # Run test suite in sandbox
grim audit       # Check versions, licenses, vulnerabilities
grim update      # Update dependencies & lockfile
grim fetch       # Fetch sources for offline build
grim shell       # Drop into dev shell with toolchain
```

---

## 📁 Project Example: `reaping.toml`

```toml
name = "ghostctl"
version = "0.3.0"
author = "GhostKellz"
license = "MIT"
build = "cargo build --release"
install = "install -Dm755 target/release/ghostctl /usr/bin/ghostctl"
source = "https://github.com/ghostkellz/ghostctl/archive/v0.3.0.tar.gz"
checksum = "sha256:abcd1234..."
```

---

## 📚 Related Files

* [README.md](./README.md) – Project overview
* [COMMANDS.md](./COMMANDS.md) – Full CLI reference
* [forge.toml Spec](https://github.com/ghostkellz/ghostforge/wiki/forge.toml-Spec)
* [Migrating from makepkg](https://github.com/ghostkellz/ghostforge/wiki/Migrating-from-Makepkg)
* [CONTRIBUTING.md](./CONTRIBUTING.md)

---

☠️ Built with paranoia by **GhostKellz**

