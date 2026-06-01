#!/usr/bin/env bash
set -euo pipefail

THREADS_TOOLCHAIN="nightly-2024-08-02"

cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo check --locked --target wasm32-unknown-unknown

if ! rustup toolchain list | grep -q "^${THREADS_TOOLCHAIN}"; then
  echo "Missing ${THREADS_TOOLCHAIN}. Install it with:"
  echo "rustup toolchain install ${THREADS_TOOLCHAIN} --component rust-src --target wasm32-unknown-unknown"
  exit 1
fi

RUSTFLAGS="-C target-feature=+atomics,+bulk-memory" \
  RUSTUP_TOOLCHAIN="${THREADS_TOOLCHAIN}" \
  cargo check --locked --target wasm32-unknown-unknown --features wasm-bindgen-rayon -Z build-std=panic_abort,std

node scripts/check-diagrams.mjs
