# 📘 Reaper Documentation

## Overview

Reaper (`reap`) is a modern, async-first AUR helper and unified package manager written in Rust. It supports AUR, Flatpak, and tap-based sources, with a focus on security, extensibility, and a clean CLI/TUI experience.

## Installation

See the README for installation instructions.

## CLI Usage

### Core Commands

- `reap -S <pkg>` / `reap install <pkg>`: Install AUR or repo package (supports --parallel, --edit)
- `reap -R <pkg>` / `reap remove <pkg>`: Remove a package
- `reap -Syu` / `reap upgradeall`: Sync and upgrade all packages
- `reap -U <file>` / `reap local <file>`: Install local package file
- `reap -Q <term>` / `reap search <term>`: Search AUR (and Flatpak)
- `reap pin <pkg>`: Pin a package to exclude from upgrades
- `reap clean`: Clean cache and temp files
- `reap doctor`: Run system health check and config audit
- `reap tui`: Launch the interactive TUI
- `reap backup`: Backup current config to backup directory

### Flatpak Commands

- `reap flatpak search <query>`: Search Flatpak packages
- `reap flatpak install <pkg>`: Install a Flatpak package
- `reap flatpak upgrade`: Upgrade Flatpak packages
- `reap flatpak audit <pkg>`: Audit Flatpak sandbox info

### GPG Commands

- `reap gpg import <keyid>`: Import a GPG key
- `reap gpg show <keyid>`: Show GPG key info and trust level
- `reap gpg check <keyid>`: Check GPG key
- `reap gpg verify <pkgdir>`: Verify PKGBUILD signature in a directory
- `reap gpg set-keyserver <url>`: Set the GPG keyserver
- `reap gpg check-keyserver <url>`: Check if a GPG keyserver is reachable

### Tap Commands

- `reap tap add <name> <url>`: Add a tap repository
- `reap tap list`: List configured tap repositories

### Other

- `reap completion <shell>`: Generate shell completion for bash, zsh, or fish

## Configuration

- Config files are stored in `~/.config/reap/`
- Backups are stored in `/var/lib/reaper/backups/`
- Pinning: `~/.config/reap/pinned.toml`
- Tap repos: `~/.config/reap/taps.json`

## Hooks

- Post-install and rollback hooks are supported via `hooks.rs`

## Security

- GPG verification is integrated for PKGBUILD signatures
- Keyserver fallback and trust level reporting

## TUI

- Run `reap tui` for an interactive terminal UI (early stage)

## See also

- [ROADMAP.md](./ROADMAP.md) for planned features and status
- [COMMANDS.md](./COMMANDS.md) for a concise command reference

