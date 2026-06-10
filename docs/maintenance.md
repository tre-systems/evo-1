# Maintenance Notes

evo-1 is a Rust evolutionary simulation with two supported execution paths:

- Native headless runs for experiments and smoke tests.
- WebAssembly browser runs with WebGPU rendering and WebGL/Canvas2D fallbacks.

The intended live site is `https://evo-1.tre.systems` on a Cloudflare Pages project named `evo-1`. Until that Pages project and custom domain are active, browser smoke tests should use the local development server.

## Required Checks

Run these before pushing code changes:

```bash
./scripts/check.sh
```

`scripts/check.sh` runs formatting, locked clippy/tests, the stable WASM check, the pinned nightly threaded-WASM check, and the Graphviz diagram check.

For the threaded browser package build:

```bash
./scripts/build-wasm.sh
```

For the Cloudflare Pages bundle:

```bash
./scripts/build-pages.sh
```

The threaded WASM path requires `nightly-2024-08-02` with `rust-src` and `wasm32-unknown-unknown` installed:

```bash
rustup toolchain install nightly-2024-08-02 --component rust-src --target wasm32-unknown-unknown
cargo install wasm-pack --locked --version 0.13.1
```

For a quick native behavior smoke test:

```bash
./run_simulation.sh 0.01 1 10 20 100 100
```

For a reproducible ecology smoke test that should reach later generations:

```bash
./run_simulation.sh 0.01 20 50 100 150 150 7
```

For a reproducible battle smoke test that should show nonzero combat pressure:

```bash
./run_scenarios.sh quick_test
```

## Browser Smoke Test

After building `pkg/`, run:

```bash
python3 server.py
```

Then open `http://127.0.0.1:8000` and verify:

- The page starts the simulation automatically.
- The population, behavior-state, predator/prey, birth, and kill stats advance without pressing a start button.
- `Reset Simulation` returns the simulation to its configured initial population and keeps it running.
- The runtime panel reports renderer and CPU worker status.
- The browser console does not show initialization errors.

After the Cloudflare Pages deployment is active, smoke test `https://evo-1.tre.systems` after every pushed code change. If the production domain is not active yet, smoke test `http://127.0.0.1:8000` and the local `dist/` build.
