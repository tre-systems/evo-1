#!/usr/bin/env bash
set -euo pipefail

THREADS_TOOLCHAIN="nightly-2024-08-02"

if ! rustup toolchain list | grep -q "^${THREADS_TOOLCHAIN}"; then
  echo "Missing ${THREADS_TOOLCHAIN}. Install it with:"
  echo "rustup toolchain install ${THREADS_TOOLCHAIN} --component rust-src --target wasm32-unknown-unknown"
  exit 1
fi

RUSTUP_TOOLCHAIN="${THREADS_TOOLCHAIN}" \
  wasm-pack build --target web --out-dir pkg . --features wasm-bindgen-rayon
