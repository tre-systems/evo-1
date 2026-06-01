#!/usr/bin/env bash
set -euo pipefail

THREADS_TOOLCHAIN="nightly-2024-08-02"

if ! rustup toolchain list | grep -q "^${THREADS_TOOLCHAIN}"; then
  echo "Missing ${THREADS_TOOLCHAIN}. Install it with:"
  echo "rustup toolchain install ${THREADS_TOOLCHAIN} --component rust-src --target wasm32-unknown-unknown"
  exit 1
fi

if ! rustup component list --toolchain "${THREADS_TOOLCHAIN}" --installed | grep -q '^rust-src'; then
  echo "Missing rust-src for ${THREADS_TOOLCHAIN}. Install it with:"
  echo "rustup component add rust-src --toolchain ${THREADS_TOOLCHAIN}"
  exit 1
fi

if ! rustup target list --toolchain "${THREADS_TOOLCHAIN}" --installed | grep -q '^wasm32-unknown-unknown'; then
  echo "Missing wasm32-unknown-unknown for ${THREADS_TOOLCHAIN}. Install it with:"
  echo "rustup target add wasm32-unknown-unknown --toolchain ${THREADS_TOOLCHAIN}"
  exit 1
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "Missing wasm-pack. Install it with: cargo install wasm-pack --locked --version 0.13.1"
  exit 1
fi

RUSTFLAGS="-C target-feature=+atomics,+bulk-memory" \
  RUSTUP_TOOLCHAIN="${THREADS_TOOLCHAIN}" \
  wasm-pack build --target web --out-dir pkg . --features wasm-bindgen-rayon
