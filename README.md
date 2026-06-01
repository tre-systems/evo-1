# BattleO

BattleO is a Rust evolutionary simulation with two supported runtimes: a native headless runner for experiments and a WebAssembly browser app for interactive visualization.

## What It Contains

- ECS-based simulation state using `hecs`.
- Spatial-grid lookups for nearby agents and resources.
- Genetic traits, reproduction, resource consumption, death, and predator/prey interactions.
- Native headless execution with Rayon-enabled resource updates.
- Browser execution with Canvas2D/WebGL rendering and optional `wasm-bindgen-rayon` worker-pool support.
- Graphviz architecture diagrams committed as source `.dot` files and rendered PNGs.

## Quick Start

### Prerequisites

- Rust stable for native builds and standard checks.
- `wasm32-unknown-unknown` for the stable WASM check.
- `nightly-2024-08-02` with `rust-src` for the threaded WASM build.
- `wasm-pack` `0.13.1` for browser packages.
- Graphviz for diagram rendering.

```bash
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly-2024-08-02 --component rust-src --target wasm32-unknown-unknown
cargo install wasm-pack --locked --version 0.13.1
# Install Graphviz with your package manager; on macOS:
brew install graphviz
```

### Browser App

```bash
./scripts/build-wasm.sh
python3 server.py
```

Open `http://127.0.0.1:8000`.

`server.py` binds to `127.0.0.1` and sends the COOP/COEP headers required for `SharedArrayBuffer` and `wasm-bindgen-rayon`.

### Headless Runner

```bash
cargo run --release --locked --bin headless -- 2.0 20 500 500 3000 2000
```

The positional arguments are:

```text
duration_minutes speed_multiplier initial_agents initial_resources max_agents max_resources [--seed seed]
```

Convenience scripts are available:

```bash
./run_simulation.sh
./run_simulation.sh 1.0 10 200 200 1000 800 202
./run_scenarios.sh
./run_scenarios.sh quick_test
```

`run_scenarios.sh` is the source of truth for the named scenario catalog. Named scenarios carry fixed seeds so ecology changes can be compared across runs.

## Local Checks

Run the full local check suite before pushing code changes:

```bash
./scripts/check.sh
```

It runs formatting, clippy, tests, stable WASM checking, the pinned threaded-WASM check, and Graphviz diagram validation.

Refresh rendered diagrams after editing `.dot` files:

```bash
node scripts/render-diagrams.mjs
node scripts/check-diagrams.mjs
```

## Documentation

- [Architecture and Patterns](docs/implementation.md) - runtime boundaries, module map, frame pipeline, and recurring implementation patterns.
- [Product Vision](docs/vision.md) - original ambition, research notes, and iteration roadmap.
- [API Reference](docs/api-reference.md) - public Rust, WASM, and headless interfaces.
- [Architecture Diagrams](docs/diagrams/README.md) - Graphviz sources, rendered PNGs, conventions, and render commands.
- [Maintenance Notes](docs/maintenance.md) - required checks, browser smoke test, and deployment status.
- [Documentation Index](docs/README.md) - compact docs map.

## Browser Requirements

The browser app needs a modern browser with WebAssembly, WebGL or Canvas2D, Web Workers, and `SharedArrayBuffer` under cross-origin isolation. Use `python3 server.py` locally so those headers are present.

## License

MIT License. See [LICENSE](LICENSE).
