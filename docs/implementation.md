# BattleO Implementation Guide

Complete technical implementation details for BattleO's core systems.

## Architecture Diagrams

BattleO follows the same diagram convention as the Antenna and swade-toolbox projects: complex architecture diagrams use Graphviz/DOT sources in `docs/diagrams`, rendered PNGs are committed for GitHub/browser viewing, and Mermaid is reserved for small inline Markdown sketches.

Start with these diagrams before changing the core architecture:

![System overview](diagrams/system-overview.png)

![Simulation frame pipeline](diagrams/frame-pipeline.png)

![Runtime model](diagrams/runtime-model.png)

## Architecture Pattern Catalog

BattleO is easiest to understand as a small set of recurring architectural patterns. These patterns are the intended shape of the codebase and should be used when adding or changing behavior.

### Boundary Adapter Pattern

External entry points are thin adapters around the simulation core:

- Browser/WASM entry point: `BattleSimulation` in `src/lib.rs`
- Native experiment entry point: `headless::HeadlessSimulation` in `src/headless.rs`
- Command-line runner: `src/bin/headless.rs`
- Browser UI shell: `index.html`

Adapters should translate platform-specific concerns into calls on `simulation::Simulation`. They should not contain simulation rules. If a behavior needs to affect agents, resources, reproduction, death, or statistics, it belongs in the simulation/ECS layer rather than in the adapter.

### Simulation Facade Pattern

`simulation::Simulation` is the facade for core behavior. It owns:

- `SimulationConfig`
- `RuntimeCapabilities`
- simulation time
- resource spawn timing
- the ECS world
- public commands such as `update`, `reset`, `add_agent`, and `add_resource`
- public query DTOs such as `SimulationStats`, legacy `Agent`, and legacy `Resource`

Callers should use `Simulation` rather than reaching into `EcsWorld` directly. `EcsWorld` remains public because rendering and compatibility code still need conversion access, but new features should prefer the facade boundary.

### Config-Driven Construction Pattern

Initial population and entity limits come from configuration:

- `Simulation::new()` applies `SimulationConfig::default()`.
- `Simulation::new_with_config(config)` applies explicit configuration.
- `EcsWorld::new_with_population(...)` clamps initial counts to max counts.
- `Simulation::reset()` restores the configured initial population.

Do not create a default simulation and then add configured entities on top. That was an older pattern and caused headless runs to start with hidden extra agents and resources.

### ECS Component Pattern

ECS entities are composed from data-only components in `src/ecs.rs`:

- common components: `Position`, `Velocity`, `Energy`, `Age`, `Size`
- agent components: `Genes`, `AgentState`, `DeathAnimation`, `SpawnAnimation`
- resource components: `Resource`
- marker components: `AgentTag`, `ResourceTag`

Systems identify agents and resources by marker components. New entity categories should use the same marker-component pattern instead of relying on partial component tuples alone.

### Ordered System Pipeline Pattern

Each frame follows the same high-level pipeline in `Simulation::update()`:

1. Advance simulation time.
2. Begin a new frame event ledger.
3. Refresh the spatial grid.
4. Update resources.
5. Handle consumption and predator/prey interactions.
6. Update agents.
7. Handle death.
8. Handle reproduction.
9. Remove depleted resources.

New per-frame systems should be placed deliberately in this order. Avoid adding hidden updates inside rendering, stats collection, or platform adapters.

### Collect-Then-Mutate Pattern

When an operation needs to inspect many entities and then mutate the world, first collect entity IDs or value snapshots, then apply mutations in a second pass. The code uses this pattern for:

- resource updates in `Simulation::update_resources_parallel`
- agent entity selection in `Simulation::update_agents_spatial`
- consumption targets in `EcsWorld::handle_consumption`
- death lists in `EcsWorld::handle_death`
- reproduction parent candidates in `EcsWorld::handle_reproduction`

This is the main rule that keeps HECS borrowing straightforward. Avoid mutating the world while iterating over broad queries unless the query is already a narrow `query_mut` pass.

### Event Ledger Pattern

Per-frame lifecycle facts are recorded as `FrameEvent` values in `EcsWorld`:

- `ResourceConsumed`
- `AgentKilled`
- `AgentDied`
- `AgentBorn`

The ledger is reset at frame start and counters are derived from events instead of inferred later from final population counts. New systems that create lifecycle facts should record events at the same point they mutate ECS state.

### Spatial Query Pattern

Neighborhood-dependent behavior should go through `SpatialGrid` instead of scanning all entities. The grid is rebuilt once per frame and used by resource consumption, predator/prey interactions, and optimized agent targeting. New proximity features, such as flocking or territorial behavior, should extend this pattern rather than reintroducing O(n^2) loops.

### Two-Phase Parallel Pattern

Parallel work must be split into:

1. collect cloneable input data from ECS,
2. compute updates in parallel over plain values,
3. apply results back to ECS sequentially.

This is currently used for resource updates. Agent updates still use a sequential optimized pass because they depend on mutable ECS state and spatial queries. Do not call parallel iterators over live mutable HECS borrows.

### Runtime Capability Pattern

Platform capability is explicit input to the simulation, not hidden global state. `RuntimeCapabilities` controls whether resource updates use the Rayon path. Native headless runs enable this capability after initializing the global Rayon pool. Browser runs enable it only after `ParallelProcessor.initialize()` succeeds.

### Compatibility DTO Pattern

The active domain model is ECS. Legacy `agent::Agent` and `resource::Resource` are compatibility DTOs used by rendering and external API surfaces. `genes::Genes` re-exports the ECS gene component so there is only one writable gene model. Conversion methods live on `Simulation`:

- `Simulation::get_agents()`
- `Simulation::get_resources()`
- `Simulation::get_stats()`

New simulation rules should use ECS components directly. Legacy DTOs should remain read-oriented until the compatibility layer can be removed.

### Rendering Strategy Pattern

`renderer::WebRenderer` chooses a rendering backend at construction:

- WebGL first
- Canvas2D fallback

Rendering reads snapshots from `Simulation`; it does not mutate simulation state. New visual effects should preserve that read-only boundary.

### Diagnostics Pattern

Headless runs produce `SimulationDiagnostics`, while browser runs expose `SimulationStats`. Long-running experiment metrics belong in diagnostics, not in the browser UI, unless they are also useful for live visualization.

## Pattern Compliance Review

Current consistency is good in the core frame loop, configuration, collect-then-mutate workflow, spatial interactions, runtime capabilities, and rendering boundary. The main inconsistencies are transitional:

- There are still legacy DTO structs for agents and resources.
- `EcsWorld` is still a large module containing components, spatial indexing, world construction, and systems.

These are not immediate correctness problems, but they are the next places to tighten if the project becomes active again.

## Patterns To Adopt Next

The code would benefit from these additional patterns:

- **Canonical Domain Model**: make ECS components the only writable model and gradually remove duplicated legacy behavior from `agent.rs`, `genes.rs`, and `resource.rs`.
- **System Module Pattern**: split `EcsWorld` systems into small modules such as `consumption`, `movement`, `reproduction`, and `lifecycle`.
- **Deterministic Run Pattern**: add optional seeded RNG to make headless scenarios reproducible.
- **Render Snapshot Pattern**: generate compact render snapshots instead of cloning full legacy DTOs every frame.

## Parallel Processing

BattleO uses true parallel processing in both native Rust and WebAssembly environments.

### Overview

- **Native**: Uses rayon thread pool for multi-core processing
- **WASM**: Uses wasm-bindgen-rayon with Web Workers for true threading when `RuntimeCapabilities.parallel_resources` is enabled by the browser adapter
- **Fallback**: Graceful degradation to sequential processing when needed

### How wasm-bindgen-rayon Works

#### Web Workers for Threading

WebAssembly doesn't have native threading, but wasm-bindgen-rayon creates JavaScript Web Workers that simulate threads:

1. **Worker Creation**: Spawns Web Workers equal to available CPU cores
2. **Message Passing**: Uses postMessage API for communication
3. **Task Distribution**: Distributes parallel tasks across workers
4. **Shared Memory**: Uses SharedArrayBuffer for efficient data sharing

#### Performance Benefits

- **2-3x speedup** for CPU-intensive operations
- **True parallelism** using all available cores
- **Automatic load balancing** across workers
- **Minimal overhead** compared to sequential processing

### Setup and Configuration

#### Prerequisites

```bash
# Install pinned nightly Rust for threaded WASM
rustup toolchain install nightly-2024-08-02 --component rust-src --target wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack
```

#### Cargo.toml Configuration

```toml
[dependencies]
rayon = "1.10"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-rayon = { version = "1.3", optional = true, features = ["no-bundler"] }
web-sys = { version = "0.3", features = ["Navigator"] }
```

### Usage

#### Initialization

```javascript
import init, { BattleSimulation, ParallelProcessor } from "./pkg/battleo.js";

async function main() {
  await init();

  const simulation = new BattleSimulation("canvas");
  const processor = new ParallelProcessor();

  // Initialize thread pool (returns a Promise)
  await processor.initialize();

  if (processor.is_rayon_available()) {
    simulation.set_parallel_resources_enabled(true);
    console.log("Parallel processing enabled!");
  } else {
    simulation.set_parallel_resources_enabled(false);
    console.log("Using sequential fallback");
  }
}
```

#### Rust Implementation

```rust
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub struct ParallelProcessor {
    initialized: bool,
    worker_count: usize,
    available: Rc<Cell<bool>>,
    closure: Option<Closure<dyn FnMut(JsValue)>>,
}

#[wasm_bindgen]
impl ParallelProcessor {
    pub fn initialize(&mut self) -> js_sys::Promise {
        #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
        {
            use wasm_bindgen_rayon::init_thread_pool;

            let available = self.available.clone();
            let on_ready = Closure::wrap(Box::new(move |_| {
                available.set(true);
            }) as Box<dyn FnMut(JsValue)>);
            self.closure = Some(on_ready);
            init_thread_pool(self.worker_count).then(self.closure.as_ref().unwrap())
        }
    }
}
```

### Browser Requirements

#### SharedArrayBuffer Support

wasm-bindgen-rayon requires SharedArrayBuffer, which needs specific security headers:

```javascript
// Server headers required
Cross-Origin-Opener-Policy: same-origin
Cross-Origin-Embedder-Policy: require-corp
```

#### Server Configuration

**Node.js/Express:**

```javascript
app.use((req, res, next) => {
  res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
  res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
  next();
});
```

## ECS Architecture

BattleO uses the HECS (Heterogeneous Component System) library to implement a high-performance Entity Component System architecture.

### Core Concepts

#### Entities

Unique identifiers that group components together. In HECS, entities are just IDs.

#### Components

Data-only structs that represent attributes of entities:

```rust
pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

pub struct Energy {
    pub current: f64,
    pub max: f64,
}
```

#### Systems

Logic that operates on components across entities:

```rust
fn update_movement_system(world: &mut World) {
    for (entity, (pos, vel)) in world.query_mut::<(&mut Position, &Velocity)>() {
        pos.x += vel.dx * delta_time;
        pos.y += vel.dy * delta_time;
    }
}
```

### HECS Implementation

#### World Setup

```rust
use hecs::World;

pub struct EcsWorld {
    pub world: World,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    pub spatial_grid: SpatialGrid,
}

impl EcsWorld {
    pub fn new(canvas_width: f64, canvas_height: f64) -> Self {
        Self::new_with_population(canvas_width, canvas_height, 10000, 1500, 100, 500)
    }

    pub fn new_with_population(
        canvas_width: f64,
        canvas_height: f64,
        max_agents: usize,
        max_resources: usize,
        initial_agents: usize,
        initial_resources: usize,
    ) -> Self {
        let mut world = Self {
            world: World::new(),
            canvas_width,
            canvas_height,
            max_agents,
            max_resources,
            resource_spawn_timer: 0.0,
            spatial_grid: SpatialGrid::new(canvas_width, canvas_height, 50.0),
            consumption_events_this_frame: 0,
            total_consumption_events: 0,
        };

        world.spawn_initial_population(
            initial_agents.min(max_agents),
            initial_resources.min(max_resources),
        );

        world
    }
}
```

`EcsWorld::new` exists for legacy callers. Core construction should usually go through `Simulation::new` or `Simulation::new_with_config`, which preserve the config-driven construction pattern.

#### Component Definitions

HECS treats any Rust type as a component, so BattleO components are ordinary cloneable data structs rather than types deriving a separate component macro:

```rust
// Core components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub current: f64,
    pub max: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Age {
    pub value: f64,
}

// Agent-specific components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentState {
    pub state: AgentStateEnum,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub last_reproduction: f64,
    pub kills: u32,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genes {
    pub speed: f64,
    pub sense_range: f64,
    pub size: f64,
    pub energy_efficiency: f64,
    pub reproduction_threshold: f64,
    pub mutation_rate: f64,
    pub aggression: f64,
    pub color_hue: f64,
    pub is_predator: f64,
    pub hunting_speed: f64,
    pub attack_power: f64,
    pub defense: f64,
    pub stealth: f64,
    pub pack_mentality: f64,
    pub territory_size: f64,
    pub metabolism: f64,
    pub intelligence: f64,
    pub stamina: f64,
    pub personal_space: f64,
}

// Resource-specific components
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
    pub energy: f64,
    pub max_energy: f64,
    pub size: f64,
    pub growth_rate: f64,
    pub regeneration_rate: f64,
    pub age: f64,
    pub target_energy: f64,
    pub is_spawning: bool,
    pub spawn_fade: f64,
    pub is_depleting: bool,
    pub deplete_fade: f64,
}

// Marker components for efficient filtering
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTag;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceTag;
```

### Spatial Partitioning

#### Spatial Grid

For efficient proximity queries, we use a spatial grid:

```rust
#[derive(Clone, Debug)]
pub struct SpatialGrid {
    pub cell_size: f64,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Vec<hecs::Entity>>>,
}

impl SpatialGrid {
    pub fn new(canvas_width: f64, canvas_height: f64, cell_size: f64) -> Self {
        let width = (canvas_width / cell_size).ceil() as usize;
        let height = (canvas_height / cell_size).ceil() as usize;

        Self {
            cell_size,
            width,
            height,
            cells: vec![vec![Vec::new(); height]; width],
        }
    }

    pub fn get_cell(&self, x: f64, y: f64) -> (usize, usize) {
        let grid_x = (x / self.cell_size).floor() as usize;
        let grid_y = (y / self.cell_size).floor() as usize;
        (grid_x.min(self.width - 1), grid_y.min(self.height - 1))
    }

    pub fn get_nearby_entities(&self, x: f64, y: f64, radius: f64) -> Vec<hecs::Entity> {
        let mut entities = Vec::new();
        let (center_x, center_y) = self.get_cell(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as usize;

        for dx in -cell_radius as i32..=cell_radius as i32 {
            for dy in -cell_radius as i32..=cell_radius as i32 {
                let grid_x = (center_x as i32 + dx) as usize;
                let grid_y = (center_y as i32 + dy) as usize;

                if grid_x < self.width && grid_y < self.height {
                    entities.extend(self.cells[grid_x][grid_y].iter().cloned());
                }
            }
        }

        entities
    }
}
```

### Query Patterns

#### Efficient Queries

```rust
// Read-only queries
for (entity, (pos, energy)) in world.query::<(&Position, &Energy)>().iter() {
    // Process entity data
}

// Mutable queries
for (entity, (pos, vel)) in world.query_mut::<(&mut Position, &Velocity)>().iter() {
    // Update entity data
}

// Filtered queries
for (entity, (pos, vel, energy, age, state, genes)) in world.query::<(&Position, &Velocity, &Energy, &Age, &AgentState, &Genes)>().iter() {
    if world.get::<&AgentTag>(entity).is_ok() {
        // Process only agents
    }
}
```

### Parallel Processing with ECS

#### Two-Phase Resource Updates

The current parallel ECS pattern is collect, compute, apply. Resource updates use rayon over cloned component data, then write results back sequentially:

```rust
use rayon::prelude::*;

fn update_resources_parallel(world: &mut World, delta_time: f64) {
    let resource_data: Vec<_> = world
        .query::<&Resource>()
        .iter()
        .filter(|(entity, _)| world.get::<&ResourceTag>(*entity).is_ok())
        .map(|(entity, resource)| (entity, resource.clone()))
        .collect();

    let updates: Vec<_> = resource_data
        .par_iter()
        .map(|(entity, resource)| {
            let mut updated = resource.clone();
            updated.update(delta_time);
            (*entity, updated)
        })
        .collect();

    for (entity, updated) in updates {
        if let Ok(mut resource) = world.get::<&mut Resource>(entity) {
            *resource = updated;
        }
    }
}
```

Agent updates should not use parallel iterators over live mutable HECS borrows. They currently run through an optimized sequential pass after spatial-grid lookup. If agent updates become parallel later, preserve the same collect, compute, apply split.

## WebGL Rendering

BattleO uses WebGL for hardware-accelerated rendering with Canvas2D fallback for compatibility.

### Architecture

#### Renderer Structure

```rust
pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    ctx_2d: Option<CanvasRenderingContext2d>,
    gl: Option<WebGlRenderingContext>,
    use_webgl: bool,
    program: Option<WebGlProgram>,
    vertex_buffer: Option<WebGlBuffer>,
    canvas_width: f32,
    canvas_height: f32,
    start_time: f64,
}
```

#### Initialization

This is a structural sketch of the current initialization pattern. The implementation also initializes shader programs, buffers, blend state, and canvas dimensions when WebGL is available.

```rust
impl WebRenderer {
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let window = web_sys::window().ok_or("No window")?;
        let document = window.document().ok_or("No document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
            .ok_or("Canvas not found")?;

        // Try WebGL first, fallback to Canvas2D
        let gl = canvas
            .get_context("webgl")
            .map_err(|_| "Failed to get WebGL context")?
            .and_then(|context| context.dyn_into::<WebGlRenderingContext>().ok());

        let ctx_2d = if gl.is_none() {
            canvas
                .get_context("2d")
                .map_err(|_| "Failed to get 2D context")?
                .and_then(|context| context.dyn_into::<CanvasRenderingContext2d>().ok())
        } else {
            None
        };

        let use_webgl = gl.is_some();
        let canvas_width = canvas.width() as f32;
        let canvas_height = canvas.height() as f32;

        Ok(WebRenderer {
            canvas,
            ctx_2d,
            gl: None,
            use_webgl,
            program: None,
            vertex_buffer: None,
            canvas_width,
            canvas_height,
            start_time: 0.0,
        })
    }
}
```

### WebGL Rendering

#### Rendering Pipeline

The WebGL pipeline reads simulation snapshots and draws them with a reusable shader program and vertex buffer:

```rust
fn render_webgl(&mut self, simulation: &Simulation) {
    if let Some(gl) = &self.gl {
        let current_time = js_sys::Date::now() / 1000.0;
        let time = current_time - self.start_time;

        // Clear the canvas
        gl.clear_color(0.02, 0.03, 0.08, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        // Get simulation data
        let agents = simulation.get_agents();
        let resources = simulation.get_resources();

        // Render agents
        for agent in &agents {
            self.render_agent_webgl(gl, agent, time);
        }

        // Render resources
        for resource in &resources {
            self.render_resource_webgl(gl, resource);
        }
    }
}
```

#### Agent Rendering

```rust
fn render_agent_webgl(&self, gl: &WebGlRenderingContext, agent: &Agent, time: f64) {
    if let (Some(program), Some(vertex_buffer)) = (&self.program, &self.vertex_buffer) {
        let energy_ratio = (agent.energy / agent.max_energy) as f32;
        let size = agent.genes.size as f32 * 8.0 * energy_ratio.max(0.1);
        let (r, g, b) = hsl_to_rgb(agent.genes.color_hue as f32, 70.0, 60.0 * energy_ratio.max(0.3));

        gl.use_program(Some(program));
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(vertex_buffer));

        // Uniforms carry per-agent position, size, color, resolution, and animation time.
        // A single circle mesh is reused for all agents.
        // See src/renderer.rs for the explicit uniform and attribute setup.
        gl.draw_arrays(WebGlRenderingContext::TRIANGLES, 0, 48);
    }
}
```

### Canvas2D Rendering

#### Rendering Pipeline

```rust
fn render_canvas2d(&mut self, simulation: &Simulation) {
    if let Some(ctx) = &self.ctx_2d {
        // Clear the canvas
        ctx.set_fill_style_str("#1a1a2e");
        ctx.fill_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64);

        // Get simulation data
        let agents = simulation.get_agents();
        let resources = simulation.get_resources();

        // Render agents
        for agent in &agents {
            self.render_agent_canvas2d(ctx, agent);
        }

        // Render resources
        for resource in &resources {
            self.render_resource_canvas2d(ctx, resource);
        }
    }
}
```

#### Agent Rendering

```rust
fn render_agent_canvas2d(&self, ctx: &CanvasRenderingContext2d, agent: &Agent) {
    let x = agent.x;
    let y = agent.y;
    let size = agent.genes.size * 3.0;

    // Convert agent color to HSL
    let hue = agent.genes.color_hue;
    let saturation = 70.0;
    let lightness = 60.0;

    // Draw agent
    ctx.set_fill_style_str(&format!("hsl({}, {}%, {}%)", hue, saturation, lightness));
    ctx.begin_path();
    ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI).unwrap();
    ctx.fill();

    // Draw border
    ctx.set_stroke_style_str("#ffffff");
    ctx.set_line_width(1.0);
    ctx.stroke();
}
```

### Color Management

#### HSL to RGB Conversion

```rust
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let h = h / 360.0;
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 1.0/6.0 {
        (c, x, 0.0)
    } else if h < 2.0/6.0 {
        (x, c, 0.0)
    } else if h < 3.0/6.0 {
        (0.0, c, x)
    } else if h < 4.0/6.0 {
        (0.0, x, c)
    } else if h < 5.0/6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (r + m, g + m, b + m)
}
```

#### Agent Color Encoding

Agent colors encode genetic information:

```rust
// Color hue represents genetic traits
let hue = agent.genes.color_hue; // 0-360 degrees

// Different hue ranges represent different traits:
// 0-60:   Red to Yellow (aggression)
// 60-120: Yellow to Green (speed)
// 120-180: Green to Cyan (energy efficiency)
// 180-240: Cyan to Blue (intelligence)
// 240-300: Blue to Magenta (defense)
// 300-360: Magenta to Red (attack power)
```

#### Resource Color Encoding

Resource colors encode energy levels:

```rust
// Color represents energy level
let energy_ratio = resource.energy / resource.max_energy;
let hue = energy_ratio * 120.0; // 0-120 degrees (green to red)

// Green: High energy
// Yellow: Medium energy
// Red: Low energy
```

### Performance Optimization

#### WebGL Optimizations

1. **Reusable Mesh**: Use one circle vertex buffer for many agents/resources
2. **Shader Uniforms**: Pass position, size, color, resolution, and time as uniforms
3. **Minimal State Changes**: Reuse shader program and vertex buffer where possible
4. **Efficient Clearing**: Clear the frame once before drawing entities

#### Canvas2D Optimizations

1. **Path Reuse**: Reuse path objects when possible
2. **Style Caching**: Cache frequently used styles
3. **Batch Operations**: Group similar drawing operations
4. **Efficient Clearing**: Use fillRect for clearing

### Browser Compatibility

#### WebGL Support

```javascript
// Check WebGL support
function checkWebGLSupport() {
  const canvas = document.createElement("canvas");
  const gl =
    canvas.getContext("webgl") || canvas.getContext("experimental-webgl");

  if (!gl) {
    console.warn("WebGL not supported, falling back to Canvas2D");
    return false;
  }

  return true;
}
```

#### Feature Detection

```javascript
const features = {
  webAssembly: typeof WebAssembly !== "undefined",
  sharedArrayBuffer: typeof SharedArrayBuffer !== "undefined",
  webWorkers: typeof Worker !== "undefined",
  crossOriginIsolated: crossOriginIsolated,
};

console.log("Features:", features);
```

## Performance Guidelines

### Optimal Settings

- **Web**: Tune population size to the browser and rendering mode
- **Headless**: Use release builds and benchmark with the target scenario
- **Spatial Grid**: 50px cell size for most simulations
- **Worker Count**: 4-8 workers for WASM, all cores for native

### Memory Usage

- **Agent**: ~200 bytes per agent
- **Resource**: ~100 bytes per resource
- **Spatial Grid**: ~1MB for 1000x800 world
- **Total**: ~1KB per entity for typical simulations

### Performance Monitoring

```rust
// Monitor frame rate
let start_time = std::time::Instant::now();
simulation.update();
let frame_time = start_time.elapsed().as_millis();

// Monitor memory usage
let agent_count = simulation.get_stats().agent_count;
let memory_usage = agent_count * 200; // Approximate bytes
```

## Best Practices

### Parallel Processing

1. **Always initialize** the thread pool before use
2. **Check availability** before using parallel operations
3. **Use appropriate chunk sizes** for your data
4. **Handle errors gracefully** with fallbacks
5. **Profile performance** to optimize chunk sizes
6. **Test on different browsers** for compatibility

### ECS Usage

1. **Use marker components**: For efficient filtering
2. **Batch operations**: Group related entity operations
3. **Optimize queries**: Only query needed components
4. **Use spatial partitioning**: For proximity queries

### Rendering

1. **Use WebGL when available**: Better performance for large numbers of entities
2. **Batch operations**: Group similar rendering operations
3. **Minimize state changes**: Reduce WebGL context switches
4. **Efficient clearing**: Use appropriate clear methods
5. **Monitor performance**: Track frame rates and rendering times
6. **Graceful fallbacks**: Always provide Canvas2D fallback
7. **Optimize data access**: Cache simulation data when possible
