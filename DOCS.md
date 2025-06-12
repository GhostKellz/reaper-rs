# 📘 Reaper Documentation

Reaper (`reap`) is a modern, async-first AUR helper and universal package manager for Arch Linux, written in Rust.

---

## Features

- Unified install: AUR, Flatpak (auto-detects source)
- Parallel async installs and upgrades
- GPG signature verification for PKGBUILDs
- Interactive TUI (search, install, logs, rollback)
- Rollback and audit support
- Lua config for custom hooks and ignored packages
- No yay/paru fallback: fully self-contained

---

## CLI Usage

- `reap -S <pkg>`         Install AUR or Flatpak package
- `reap -R <pkg>`         Remove a package
- `reap -Ss <term>`       Search AUR
- `reap -Syu`             Sync and upgrade all packages
- `reap -U <file>`        Install local .zst or .pkg.tar.zst
- `reap tui`              Launch TUI
- `reap clean`            Clean package cache
- `reap doctor`           Run diagnostics
- `reap gpg refresh`      Refresh GPG keys
- `reap rollback <pkg>`   Rollback a package

---

## TUI Features

- Real-time log pane
- Search and install interactively
- Parallel install/upgrade
- PKGBUILD diff/preview
- Scrollable results and logs

---

## Security

- GPG key management and PKGBUILD signature checks
- Rollback support
- Audit mode for PKGBUILD and Flatpak manifests

---

## Configuration

- `~/.config/reaper/brew.lua` for ignored packages, parallelism, and custom hooks

---

## Roadmap

- Makepkg integration
- Backend switching (`--backend flatpak`)
- Audit for GPG trust level, optional deps

---

See the ROADMAP.md for more details and planned features.

