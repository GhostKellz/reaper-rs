## 📦 Unified Package Management Commands

### Core
- `reap install <pkg>` / `-S <pkg>`: Install package (AUR, Flatpak, or tap)
- `reap remove <pkg>` / `-R <pkg>`: Remove package
- `reap upgradeall` / `-Syu`: Upgrade all packages
- `reap local <file>` / `-U <file>`: Install local package
- `reap search <term>` / `-Q <term>`: Search for packages
- `reap pin <pkg>`: Pin package
- `reap clean`: Clean cache
- `reap doctor`: System health check
- `reap tui`: Interactive TUI
- `reap backup`: Backup config

### Flatpak
- `reap flatpak search <query>`: Search Flatpak
- `reap flatpak install <pkg>`: Install Flatpak
- `reap flatpak upgrade`: Upgrade Flatpak
- `reap flatpak audit <pkg>`: Audit Flatpak

### GPG
- `reap gpg import <keyid>`: Import GPG key
- `reap gpg show <keyid>`: Show GPG key info
- `reap gpg check <keyid>`: Check GPG key
- `reap gpg verify <pkgdir>`: Verify PKGBUILD signature
- `reap gpg set-keyserver <url>`: Set GPG keyserver
- `reap gpg check-keyserver <url>`: Check GPG keyserver

### Tap
- `reap tap add <name> <url>`: Add tap repo
- `reap tap list`: List tap repos

### Misc
- `reap completion <shell>`: Shell completion

---

See [DOCS.md](./DOCS.md) for full documentation and usage details.

