#!/usr/bin/env bash
# Netlify CI build script for Uplink MVP
# Installs Rust + wasm-pack (if absent), builds the Wasm bundle, then the web app.
set -euo pipefail

# ── 1. Rust toolchain ─────────────────────────────────────────────────────
if ! command -v cargo &>/dev/null; then
  echo ">>> Installing Rust toolchain (minimal profile)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal --quiet
fi

# shellcheck source=/dev/null
source "${HOME}/.cargo/env"

echo ">>> Rust $(rustc --version)"
rustup target add wasm32-unknown-unknown

# ── 2. wasm-pack ──────────────────────────────────────────────────────────
if ! command -v wasm-pack &>/dev/null; then
  echo ">>> Installing wasm-pack..."
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

echo ">>> wasm-pack $(wasm-pack --version)"

# ── 3. npm deps + Wasm bundle + web build ─────────────────────────────────
echo ">>> Building web app..."
cd web
npm ci
npm run wasm:build
npm run build

echo ">>> Build complete — dist ready at web/dist"
