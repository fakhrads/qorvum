#!/usr/bin/env bash
# ============================================================
# Qorvum — Enterprise Post-Quantum Blockchain
# Setup script for local development
# ============================================================
set -euo pipefail

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; CYAN='\033[0;36m'; NC='\033[0m'
info()    { echo -e "${GREEN}[qorvum]${NC} $*"; }
warn()    { echo -e "${YELLOW}[warn]${NC}  $*"; }
error()   { echo -e "${RED}[error]${NC} $*"; exit 1; }
step()    { echo -e "\n${CYAN}▶ $*${NC}"; }

RUST_MIN="1.85.0"

echo -e "${GREEN}"
echo "  ╔═══════════════════════════════════════════════════╗"
echo "  ║   Qorvum Enterprise Post-Quantum Blockchain       ║"
echo "  ║   Setup Script                                    ║"
echo "  ╚═══════════════════════════════════════════════════╝"
echo -e "${NC}"

# ── 1. Cek Rust ───────────────────────────────────────────────────────────────
step "Checking Rust installation"

if ! command -v rustc &>/dev/null; then
    warn "Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
fi

RUST_VER=$(rustc --version | awk '{print $2}')
info "Rust version: $RUST_VER"

if [[ "$(printf '%s\n' "$RUST_MIN" "$RUST_VER" | sort -V | head -1)" != "$RUST_MIN" ]]; then
    warn "Rust $RUST_VER < $RUST_MIN. Updating..."
    rustup update stable
    source "$HOME/.cargo/env"
fi

# ── 2. WASM target ────────────────────────────────────────────────────────────
step "Adding wasm32 target"
rustup target add wasm32-unknown-unknown

# ── 3. System dependencies ────────────────────────────────────────────────────
step "Checking system dependencies"

if command -v apt-get &>/dev/null; then
    sudo apt-get install -y \
        pkg-config libclang-dev clang cmake \
        build-essential libssl-dev protobuf-compiler \
        2>/dev/null || warn "apt install failed — some deps may be missing"
elif command -v brew &>/dev/null; then
    brew install llvm cmake protobuf 2>/dev/null || true
fi

# ── 4. Build workspace ────────────────────────────────────────────────────────
step "Building workspace"
cargo build --workspace
info "Build complete"

# ── 5. Install qv CLI ────────────────────────────────────────────────────────
step "Installing qv CLI"

cargo install --path crates/qorvum-cli --quiet

# Verifikasi
if command -v qv &>/dev/null; then
    QV_VER=$(qv --version 2>/dev/null || echo "installed")
    info "qv installed: $QV_VER"
    info "Location    : $(which qv)"
else
    # Fallback: ~/.cargo/bin mungkin belum di PATH di shell ini
    CARGO_BIN="$HOME/.cargo/bin"
    if [ -f "$CARGO_BIN/qv" ]; then
        info "qv installed at $CARGO_BIN/qv"
        warn "Tambahkan ke PATH jika belum ada:"
        warn "  echo 'export PATH=\"\$HOME/.cargo/bin:\$PATH\"' >> ~/.bashrc"
        warn "  source ~/.bashrc"
    else
        warn "qv install mungkin gagal — coba manual: cargo install --path crates/qorvum-cli"
    fi
fi

# ── 6. Run tests ──────────────────────────────────────────────────────────────
step "Running tests"
cargo test --workspace --lib
info "All tests passed"

# ── 7. Tampilkan direktori struktur Qorvum ────────────────────────────────────
QORVUM_HOME="$HOME/.qorvum"
if [ ! -d "$QORVUM_HOME" ]; then
    mkdir -p "$QORVUM_HOME/ca"
    mkdir -p "$QORVUM_HOME/identities"
    info "Created Qorvum home: $QORVUM_HOME"
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Setup selesai!                                          ║${NC}"
echo -e "${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${CYAN}Quick start (dev mode):${NC}"
echo "    cargo run -p qorvum-node"
echo "    curl http://localhost:8080/api/v1/health"
echo ""
echo -e "  ${CYAN}Setup PKI (production):${NC}"
echo "    qv ca init --org MyOrg"
echo "    qv ca issue --org MyOrg --name admin --roles ADMIN,HR_MANAGER"
echo "    qv identity use ~/.qorvum/identities/admin.cert ~/.qorvum/identities/admin.key"
echo ""
echo -e "  ${CYAN}Node info & peer id:${NC}"
echo "    qv node peer-id --data-dir ./data"
echo "    qv node info    --data-dir ./data"
echo ""
echo -e "  ${CYAN}Direktori Qorvum:${NC}"
echo "    CA          : ~/.qorvum/ca/<org-name>/"
echo "    Identities  : ~/.qorvum/identities/"
echo "    Active ID   : ~/.qorvum/active.profile"
echo ""