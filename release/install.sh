#!/bin/bash
# Reaper v0.6.0 Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/GhostKellz/reaper/main/release/install.sh | bash

set -e

REPO_URL="https://github.com/GhostKellz/reaper"
RELEASE_URL="$REPO_URL/releases/latest/download"
INSTALL_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/reap"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running on Arch Linux
check_arch() {
    if [[ ! -f /etc/arch-release ]]; then
        error "This installer is designed for Arch Linux. For other distributions, please build from source."
        exit 1
    fi
}

# Detect architecture
detect_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64) echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *) 
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Check dependencies
check_dependencies() {
    info "Checking dependencies..."
    
    local missing_deps=()
    
    # Core dependencies
    command -v pacman >/dev/null 2>&1 || missing_deps+=("pacman")
    command -v git >/dev/null 2>&1 || missing_deps+=("git")
    
    # Optional but recommended
    if ! command -v flatpak >/dev/null 2>&1; then
        warning "Flatpak not found. Install for Flatpak support: sudo pacman -S flatpak"
    fi
    
    if ! command -v gpg >/dev/null 2>&1; then
        warning "GPG not found. Install for signature verification: sudo pacman -S gnupg"
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        error "Missing required dependencies: ${missing_deps[*]}"
        info "Install with: sudo pacman -S ${missing_deps[*]}"
        exit 1
    fi
    
    success "All required dependencies found"
}

# Download and install binary
install_binary() {
    local arch=$(detect_arch)
    local binary_url="$RELEASE_URL/reap-${arch}"
    local temp_file="/tmp/reap-${arch}"
    
    info "Downloading Reaper binary for $arch..."
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$binary_url" -o "$temp_file"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$binary_url" -O "$temp_file"
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
    
    # Make executable and install
    chmod +x "$temp_file"
    
    if [[ -w "$INSTALL_DIR" ]]; then
        mv "$temp_file" "$INSTALL_DIR/reap"
    else
        info "Installing to system directory (requires sudo)..."
        sudo mv "$temp_file" "$INSTALL_DIR/reap"
    fi
    
    success "Binary installed to $INSTALL_DIR/reap"
}

# Install shell completions
install_completions() {
    info "Installing shell completions..."
    
    local completion_dir="/tmp/reap-completions"
    mkdir -p "$completion_dir"
    
    # Download completion files
    curl -fsSL "$REPO_URL/raw/main/completions/bash/reap" -o "$completion_dir/reap.bash"
    curl -fsSL "$REPO_URL/raw/main/completions/zsh/_reap" -o "$completion_dir/_reap"
    curl -fsSL "$REPO_URL/raw/main/completions/fish/reap.fish" -o "$completion_dir/reap.fish"
    
    # Install completions
    if [[ -d "/usr/share/bash-completion/completions" ]]; then
        sudo cp "$completion_dir/reap.bash" "/usr/share/bash-completion/completions/reap"
        success "Bash completion installed"
    fi
    
    if [[ -d "/usr/share/zsh/site-functions" ]]; then
        sudo cp "$completion_dir/_reap" "/usr/share/zsh/site-functions/_reap"
        success "Zsh completion installed"
    fi
    
    if [[ -d "/usr/share/fish/vendor_completions.d" ]]; then
        sudo cp "$completion_dir/reap.fish" "/usr/share/fish/vendor_completions.d/reap.fish"
        success "Fish completion installed"
    fi
    
    rm -rf "$completion_dir"
}

# Setup configuration
setup_config() {
    info "Setting up configuration..."
    
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$CONFIG_DIR/profiles"
    mkdir -p "$CONFIG_DIR/hooks"
    mkdir -p "$CONFIG_DIR/taps"
    
    # Create default config if it doesn't exist
    if [[ ! -f "$CONFIG_DIR/reap.toml" ]]; then
        cat > "$CONFIG_DIR/reap.toml" << 'EOF'
# Reaper Configuration
backend_order = ["tap", "aur", "pacman", "flatpak"]
auto_resolve_deps = true
noconfirm = false
log_verbose = true
theme = "dark"
show_tips = false
enable_cache = true
enable_lua_hooks = false

[security]
strict_signatures = false
allow_insecure = false
gpg_keyserver = "hkps://keys.openpgp.org"

[performance]
parallel_jobs = 4
fast_mode = false
max_parallel = 8

[ui]
show_progress = true
colored_output = true
trust_badges = true
rating_stars = true
EOF
        success "Default configuration created"
    else
        info "Configuration already exists, skipping"
    fi
    
    # Create example hook
    if [[ ! -f "$CONFIG_DIR/hooks/post_install.sh" ]]; then
        cat > "$CONFIG_DIR/hooks/post_install.sh" << 'EOF'
#!/bin/bash
# Example post-install hook
echo "[HOOK] Package $REAP_PKG installed from $REAP_SOURCE"
EOF
        chmod +x "$CONFIG_DIR/hooks/post_install.sh"
        success "Example hook created"
    fi
}

# Create default profiles
create_profiles() {
    info "Creating default profiles..."
    
    # Developer profile
    cat > "$CONFIG_DIR/profiles/developer.toml" << 'EOF'
name = "developer"
backend_order = ["tap", "aur", "flatpak"]
auto_install_deps = ["base-devel", "git", "rust", "nodejs", "python"]
pinned_packages = ["linux-lts"]
ignored_packages = []
parallel_jobs = 8
fast_mode = false
strict_signatures = true
auto_resolve_deps = true
EOF

    # Gaming profile
    cat > "$CONFIG_DIR/profiles/gaming.toml" << 'EOF'
name = "gaming"
backend_order = ["flatpak", "aur", "chaotic-aur"]
auto_install_deps = ["steam", "lutris", "wine", "gamemode"]
pinned_packages = []
ignored_packages = []
parallel_jobs = 6
fast_mode = true
strict_signatures = false
auto_resolve_deps = true
EOF

    # Minimal profile
    cat > "$CONFIG_DIR/profiles/minimal.toml" << 'EOF'
name = "minimal"
backend_order = ["pacman", "aur"]
auto_install_deps = []
pinned_packages = []
ignored_packages = []
parallel_jobs = 2
fast_mode = true
strict_signatures = false
auto_resolve_deps = false
EOF

    success "Default profiles created (developer, gaming, minimal)"
}

# Verify installation
verify_installation() {
    info "Verifying installation..."
    
    if command -v reap >/dev/null 2>&1; then
        local version=$(reap --version 2>/dev/null || echo "unknown")
        success "Reaper installed successfully: $version"
        
        info "Testing basic functionality..."
        if reap --help >/dev/null 2>&1; then
            success "Basic functionality test passed"
        else
            warning "Basic functionality test failed"
        fi
    else
        error "Installation verification failed. Reaper not found in PATH."
        info "You may need to restart your shell or run: export PATH=\"$INSTALL_DIR:\$PATH\""
        exit 1
    fi
}

# Main installation flow
main() {
    echo "🔥 Reaper v0.6.0 Installer"
    echo "=========================="
    
    check_arch
    check_dependencies
    install_binary
    install_completions
    setup_config
    create_profiles
    verify_installation
    
    echo ""
    success "🎉 Reaper installation completed successfully!"
    echo ""
    info "Next steps:"
    echo "  1. Restart your shell or run: source ~/.bashrc"
    echo "  2. Run 'reap --help' to see available commands"
    echo "  3. Run 'reap profile list' to see available profiles"
    echo "  4. Run 'reap doctor' to verify system health"
    echo "  5. Run 'reap tui' for the interactive interface"
    echo ""
    info "Documentation: https://github.com/GhostKellz/reaper"
    info "Configuration: $CONFIG_DIR/reap.toml"
    echo ""
}

# Run installer
main "$@"