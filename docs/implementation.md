# Architecture and Patterns

This document is the source of truth for BattleO's architectural shape and recurring implementation patterns. Public callable APIs are documented in [api-reference.md](api-reference.md); generated diagrams are documented in [diagrams/README.md](diagrams/README.md).

## System Shape

BattleO has one simulation core with two runtime adapters:

- Browser runtime: `index.html` loads the generated WASM package and calls `BattleSimulation` and `ParallelProcessor` from [src/lib.rs](../src/lib.rs).
- Native runtime: [src/bin/headless.rs](../src/bin/headless.rs) parses CLI arguments and runs `HeadlessSimulation` from [src/headless.rs](../src/headless.rs).
- Shared core: [src/simulation.rs](../src/simulation.rs) owns the public simulation facade and delegates world mutation to [src/ecs.rs](../src/ecs.rs).

The diagrams show the same model visually:

- [System overview](diagrams/system-overview.png)
- [Frame pipeline](diagrams/frame-pipeline.png)
- [Runtime model](diagrams/runtime-model.png)

## Module Map

| Module | Responsibility |
| --- | --- |
| [src/lib.rs](../src/lib.rs) | Crate exports plus WASM-only `BattleSimulation`, `ParallelProcessor`, and panic hook bindings. |
| [src/simulation.rs](../src/simulation.rs) | Stable facade for construction, updates, commands, stats, runtime capabilities, and snapshot DTO conversion. |
| [src/ecs.rs](../src/ecs.rs) | ECS world ownership, entity spawning, ordered systems, frame-event ledger, lifecycle rules, and query helpers. |
| [src/ecs/components.rs](../src/ecs/components.rs) | Data-only components and marker tags such as `AgentTag` and `ResourceTag`. |
| [src/ecs/spatial.rs](../src/ecs/spatial.rs) | Spatial grid used for nearby-entity lookups. |
| [src/genes.rs](../src/genes.rs) | Compatibility import path for the canonical ECS `Genes` component plus gene construction and inheritance helpers. |
| [src/agent.rs](../src/agent.rs) | Read-only agent snapshot DTO exposed to renderers and external callers. |
| [src/resource.rs](../src/resource.rs) | Read-only resource snapshot DTO exposed to renderers and external callers. |
| [src/renderer.rs](../src/renderer.rs) | WASM-only renderer with WebGL first and Canvas2D fallback. |
| [src/headless.rs](../src/headless.rs) | Native experiment runner and diagnostics aggregation. |

## Pattern Catalog

### Boundary Adapter

Platform-specific code stays at the edge:

- `BattleSimulation` converts browser/WASM calls into `Simulation` calls.
- `ParallelProcessor` owns WASM thread-pool initialization and exposes readiness as a capability.
- `HeadlessSimulation` owns native run configuration, Rayon setup, progress output, and diagnostics.

Core simulation rules should not depend on browser APIs, DOM objects, CLI parsing, or stdout formatting.

### Simulation Facade

`Simulation` is the public core API. It owns `SimulationConfig`, `RuntimeCapabilities`, the frame clock, and an `EcsWorld`.

Callers should use `Simulation` for construction, reset, updates, counts, stats, and snapshot reads. New behavior that mutates entities should live in the ECS world or a focused ECS system helper, then be called by the `Simulation::update` pipeline.

### Config-Driven Construction

Population limits and initial counts flow through `SimulationConfig`. `Simulation::new_with_config_and_capabilities` is the most explicit constructor because it records both simulation size and runtime feature support.

Initial counts are clamped to maximum counts when the ECS world is created or reset. Avoid adding constructors that bypass this path.

### Explicit Runtime Capability

Parallel resource updates are controlled by:

```rust
RuntimeCapabilities {
    parallel_resources: bool,
}
```

Native headless runs enable this around a global Rayon pool that the runner initializes or reuses. Browser runs enable it only after `ParallelProcessor.initialize()` confirms the WASM worker pool is available. The core update path reads the capability; it does not infer platform support itself.

### ECS Entity Shape

Entity roles are expressed with marker tags:

- Agent entities have `AgentTag`, position, velocity, energy, age, genes, state, size, and animation components.
- Resource entities have `ResourceTag`, position, resource data, and size.

Components should stay data-oriented. Behavior belongs in ECS system functions, not component methods, except for small local invariants such as resource availability or gene inheritance.

### Ordered Frame Pipeline

`Simulation::update` is the primary frame pipeline:

1. Advance fixed delta time.
2. Clear the per-frame event ledger.
3. Rebuild the spatial grid.
4. Update resources through the sequential or parallel resource path.
5. Resolve resource consumption and predator/prey interactions.
6. Update agents with spatial targeting and personal-space avoidance.
7. Despawn dead agents and record deaths.
8. Reproduce eligible agents and record births.
9. Remove depleted resources.

Preserve this ordering unless the behavior change explicitly requires a different causality model. In particular, stats and rendering assume a committed frame after lifecycle cleanup.

### Collect-Then-Mutate

Many ECS operations first collect entity IDs or cloneable component snapshots, then mutate the world in a second pass. This avoids borrow conflicts and keeps parallel work away from direct `hecs::World` mutation.

The parallel resource path follows this pattern:

1. Collect resource entity IDs and cloned resource components.
2. Compute updated resources in parallel.
3. Apply updates sequentially back into the world.

Use the same pattern for any new system that needs multiple reads before mutation, or any system that may become parallel later.

### Spatial Query

The spatial grid is rebuilt once at the start of the public frame pipeline and is used for:

- nearby resources for agent targeting,
- nearby agents for personal-space avoidance,
- resource consumption checks,
- predator/prey checks.

New proximity behavior should use `SpatialGrid` instead of broad all-pairs scans. If a new behavior needs positions that change during the same frame, document where the grid is refreshed or why stale-within-frame positions are acceptable.

### Frame Event Ledger

`EcsWorld::frame_events` records per-frame facts such as consumed resources, agent kills, deaths, and births. Aggregated counters feed `SimulationStats` and `SimulationDiagnostics`.

When adding user-visible lifecycle behavior, add the event at the mutation site rather than recomputing it later from final state.

### Snapshot DTO

The active writable model is ECS. `agent::Agent` and `resource::Resource` are read-only snapshots produced for rendering and API callers. They should not become a second domain model.

`genes::Genes` is a compatibility re-export of the ECS `Genes` component, not a separate snapshot type.

### Rendering Strategy

Rendering is browser-only and reads snapshots from `Simulation`. `WebRenderer` attempts WebGL first and falls back to Canvas2D when WebGL setup is unavailable.

Rendering code should stay read-only with respect to simulation rules. Add render-specific fields to snapshot DTOs only when they are stable API data; otherwise prefer a dedicated render snapshot type.

### Diagnostics Summary

The native runner converts final stats into `SimulationDiagnostics`: duration, steps, stability flags, generation progress, reproduction/death totals, and a quality score.

Diagnostics are run summaries, not simulation rules. Keep scoring changes inside [src/headless.rs](../src/headless.rs).

### Generated Diagram Documentation

Complex diagrams use Graphviz `.dot` sources under [docs/diagrams](diagrams). Rendered PNGs are committed for easy browser and GitHub reading.

Use Mermaid only for small inline sketches. For architecture diagrams, update the `.dot` file and regenerate PNGs with:

```bash
node scripts/render-diagrams.mjs
node scripts/check-diagrams.mjs
```

## Build and Runtime Notes

The browser package is built by [scripts/build-wasm.sh](../scripts/build-wasm.sh). It uses:

- `nightly-2024-08-02`,
- `rust-src`,
- `wasm32-unknown-unknown`,
- `wasm-pack --target web`,
- atomics and bulk-memory target features,
- the `wasm-bindgen-rayon` feature with `no-bundler`.

The local development server is [server.py](../server.py). It binds to `127.0.0.1`, serves static files, and sends COOP/COEP headers required for browser thread support.

The full local check suite is [scripts/check.sh](../scripts/check.sh). It covers Rust formatting, clippy, tests, stable WASM checking, threaded-WASM checking, and diagram validation.

## Patterns to Tighten Next

These are the current architecture pressure points to address before adding larger features:

- Split the largest ECS systems into focused system modules once behavior changes require it. Today they are still understandable in one file, but `EcsWorld` is the main growth risk.
- Make deterministic seeded runs a first-class option for reproducible experiments. Current runs use thread-local randomness directly.
- Replace full agent/resource DTO cloning with compact render-specific snapshots if browser rendering or API serialization becomes a bottleneck.
- Collapse the resource `Size` duplication. Resource entities currently carry both `Resource::size` and a `Size` component, so future resource-rendering changes should pick one source of truth.

## Documentation Boundary

Keep this file about architecture and patterns. Do not duplicate full public APIs here; update [api-reference.md](api-reference.md) instead. Do not record benchmark-style claims here unless they are produced by a committed benchmark and include the command used to reproduce them.
