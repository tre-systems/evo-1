# BattleO - Evolutionary Agent Simulation

A high-performance evolutionary simulation framework built in Rust with WebAssembly support for real-time browser visualization.

## Features

- 🚀 **High Performance**: Parallel processing with rayon (native) and wasm-bindgen-rayon (WASM)
- 🧬 **Evolutionary**: Genetic algorithms with mutation, reproduction, and natural selection
- 🎮 **Real-time**: 60 FPS simulation with WebGL/Canvas2D rendering
- 🔬 **Headless Mode**: Fast batch processing for experiments and research
- 🏗️ **ECS Architecture**: Entity Component System using HECS for scalability
- 🌐 **Cross-platform**: Native Rust and WebAssembly targets

## Quick Start

### Prerequisites

- Rust stable for native builds
- Nightly Rust for threaded WASM builds
- wasm-pack for browser packages: `cargo install wasm-pack`

### Build and Run

#### Web Mode (Browser)

```bash
# Build with parallel processing support
RUSTUP_TOOLCHAIN=nightly wasm-pack build --target web --out-dir pkg . --features wasm-bindgen-rayon -Z unstable-options

# Serve with SharedArrayBuffer headers and open in browser
python3 server.py
# Open http://localhost:8000
```

The `-Z unstable-options` passthrough is required by current Cargo/wasm-pack combinations when wasm-pack forwards its output directory to `cargo build`.

#### Headless Mode (Native)

```bash
# Build and run headless simulation
cargo build --release
cargo run --bin headless

# Or use the convenient scripts:
./run_simulation.sh                    # Default configuration
./run_scenarios.sh                     # List available scenarios
./run_scenarios.sh evolution_test      # Run specific scenario
```

### Simulation Scripts

#### `run_simulation.sh` - Custom Parameterized Runner

Flexible script for custom experiments and parameter testing:

```bash
# Usage: ./run_simulation.sh [duration] [speed] [agents] [resources] [max_agents] [max_resources]

# Examples:
./run_simulation.sh                    # Defaults: 2min, 20x, 500/500, 3000/2000
./run_simulation.sh 1.0 10 200 200     # 1min, 10x, 200 agents/resources
./run_simulation.sh 0.5 15 100 100 500 500  # Full custom configuration
```

**Perfect for:**

- Quick experiments
- Parameter testing
- Custom configurations
- Iterative development

#### `run_scenarios.sh` - Predefined Evolution Scenarios

Curated collection of interesting evolution scenarios:

```bash
# List all available scenarios:
./run_scenarios.sh

# Run specific scenarios:
./run_scenarios.sh quick_test          # 30 seconds, fast evolution test
./run_scenarios.sh stress_test         # 2 minutes, max CPU utilization
./run_scenarios.sh evolution_test      # 15 minutes, focused evolution
./run_scenarios.sh sustained_evolution # 30 minutes, long-term evolution
```

**Available Scenarios:**

- **`quick_test`**: 0.5min, 10x, 100/100 → 500/500 (fast test)
- **`short_run`**: 1.0min, 20x, 200/200 → 1000/800 (quick evolution)
- **`medium_run`**: 5.0min, 15x, 500/500 → 2000/1500 (balanced)
- **`long_run`**: 10.0min, 10x, 1000/1000 → 5000/3000 (extended)
- **`stress_test`**: 2.0min, 50x, 2000/2000 → 10000/5000 (CPU stress)
- **`evolution_test`**: 15.0min, 5x, 300/300 → 1500/1000 (evolution focused)
- **`evolution_focused`**: 10.0min, 8x, 400/600 → 1200/800 (balanced evolution)
- **`balanced_evolution`**: 20.0min, 3x, 200/400 → 800/600 (sustained)
- **`sustained_evolution`**: 30.0min, 2x, 150/300 → 600/500 (long-term)

**Perfect for:**

- Reproducible experiments
- Different evolution scenarios
- Performance benchmarking
- Long-term studies

Both scripts automatically:

- ✅ **Build** the optimized binary
- ✅ **Initialize** rayon with the available CPU threads
- ✅ **Run** the simulation
- ✅ **Display** detailed results
- ✅ **Show** evolution metrics

## Architecture

### Core Components

- **Simulation Engine**: ECS-based with spatial partitioning
- **Parallel Processing**: Rayon (native) / wasm-bindgen-rayon (WASM)
- **Rendering**: WebGL with Canvas2D fallback
- **Agents**: Evolvable creatures with genes and behaviors
- **Resources**: Consumable energy sources

### Key Technologies

- **HECS**: Entity Component System for scalable simulation
- **wasm-bindgen-rayon**: True parallel processing in WebAssembly
- **WebGL**: Hardware-accelerated rendering
- **Spatial Grid**: O(1) proximity queries for performance

## Documentation

See [docs/README.md](docs/README.md) for complete documentation covering:

- **Implementation Guide** - Parallel processing, ECS architecture, and WebGL rendering
- **API Reference** - Complete API documentation
- **Maintenance Notes** - Checks, browser smoke tests, live-site status, and branch cleanup guidance

## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# WASM build with parallel processing
RUSTUP_TOOLCHAIN=nightly wasm-pack build --target web --out-dir pkg . --features wasm-bindgen-rayon -Z unstable-options

# Run tests
cargo test
```

### Configuration

The simulation can be configured for different use cases:

```rust
use battleo::simulation::SimulationConfig;

let config = SimulationConfig {
    width: 1000.0,
    height: 800.0,
    max_agents: 5000,
    max_resources: 2000,
    initial_agents: 500,
    initial_resources: 500,
    resource_spawn_rate: 1.0, // Reserved for automatic resource spawning
};
```

`initial_agents` and `initial_resources` are applied when the simulation is created or reset. If they exceed the configured maximums, they are clamped to `max_agents` and `max_resources`.

## Performance

BattleO has native and WASM execution paths, with rayon available for native headless runs and wasm-bindgen-rayon available for browser builds. Actual throughput depends heavily on population size, browser support for SharedArrayBuffer, and whether the threaded WASM build is available.

## Browser Support

- Chrome 92+ (with SharedArrayBuffer)
- Firefox 79+ (with SharedArrayBuffer)
- Safari 15.2+ (with SharedArrayBuffer)

**Note**: SharedArrayBuffer requires proper CORS headers for security.

## License

MIT License - see LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Roadmap

- [ ] GPU acceleration with WebGPU
- [ ] Advanced genetic algorithms
- [ ] Multi-species evolution
- [ ] Network simulation
- [ ] Machine learning integration
