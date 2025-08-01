# BattleO Implementation Guide

Complete technical implementation details for BattleO's core systems.

## Parallel Processing

BattleO uses true parallel processing in both native Rust and WebAssembly environments.

### Overview

- **Native**: Uses rayon thread pool for multi-core processing
- **WASM**: Uses wasm-bindgen-rayon with Web Workers for true threading
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
# Install nightly Rust (required for wasm-bindgen-rayon)
rustup toolchain install nightly
rustup default nightly

# Install wasm-pack
cargo install wasm-pack

# Add WASM target
rustup target add wasm32-unknown-unknown --toolchain nightly
```

#### Cargo.toml Configuration

```toml
[dependencies]
rayon = "1.10"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen-rayon = "1.3"
web-sys = { version = "0.3", features = ["Navigator"] }
```

### Usage

#### Initialization

```javascript
import init, { ParallelProcessor } from "./pkg/battleo.js";

async function main() {
  await init();

  // Create parallel processor
  const processor = new ParallelProcessor();

  // Initialize thread pool (returns a Promise)
  await processor.initialize();

  // Check if parallel processing is available
  if (processor.is_rayon_available()) {
    console.log("Parallel processing enabled!");
  } else {
    console.log("Using sequential fallback");
  }
}
```

#### Rust Implementation

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub struct ParallelProcessor {
    initialized: bool,
    worker_count: usize,
}

#[wasm_bindgen]
impl ParallelProcessor {
    pub fn initialize(&mut self) -> js_sys::Promise {
        #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
        {
            use wasm_bindgen_rayon::init_thread_pool;

            // Initialize Web Workers
            init_thread_pool(self.worker_count).then(|result| {
                match result {
                    Ok(_) => {
                        simulation::set_rayon_available(true);
                        web_sys::console::log_1(&"Thread pool initialized".into());
                    }
                    Err(_) => {
                        simulation::set_rayon_available(false);
                        web_sys::console::warn_1(&"Failed to initialize thread pool".into());
                    }
                }
                wasm_bindgen::JsValue::NULL
            })
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
#[derive(Component)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Component)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Component)]
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
    pub spatial_grid: SpatialGrid,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

impl EcsWorld {
    pub fn new(canvas_width: f64, canvas_height: f64) -> Self {
        Self {
            world: World::new(),
            spatial_grid: SpatialGrid::new(canvas_width, canvas_height, 50.0),
            canvas_width,
            canvas_height,
        }
    }
}
```

#### Component Definitions

```rust
// Core components
#[derive(Component, Clone, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Component, Clone, Debug)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Component, Clone, Debug)]
pub struct Energy {
    pub current: f64,
    pub max: f64,
}

#[derive(Component, Clone, Debug)]
pub struct Age {
    pub value: f64,
}

// Agent-specific components
#[derive(Component, Clone, Debug)]
pub struct AgentState {
    pub state: AgentStateEnum,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub last_reproduction: f64,
    pub kills: u32,
    pub generation: u32,
}

#[derive(Component, Clone, Debug)]
pub struct Genes {
    pub speed: f64,
    pub sense_range: f64,
    pub size: f64,
    pub energy_efficiency: f64,
    pub reproduction_threshold: f64,
    pub mutation_rate: f64,
    pub aggression: f64,
    pub color_hue: f64,
    pub is_predator: bool,
    pub hunting_speed: f64,
    pub attack_power: f64,
    pub defense: f64,
    pub stealth: f64,
    pub pack_mentality: f64,
    pub territory_size: f64,
    pub metabolism: f64,
    pub intelligence: f64,
    pub stamina: f64,
}

// Resource-specific components
#[derive(Component, Clone, Debug)]
pub struct Resource {
    pub energy: f64,
    pub max_energy: f64,
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
#[derive(Component)]
pub struct AgentTag;

#[derive(Component)]
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

#### Safe Parallel Queries

```rust
use rayon::prelude::*;

fn update_agents_parallel(world: &mut World) {
    // Collect entities first to avoid borrowing conflicts
    let agent_entities: Vec<_> = world
        .query::<(&Position, &Velocity, &Energy, &Age, &AgentState, &Genes)>()
        .iter()
        .filter(|(entity, _)| world.get::<&AgentTag>(*entity).is_ok())
        .map(|(entity, _)| entity)
        .collect();

    // Update agents in parallel
    agent_entities.par_iter().for_each(|&entity| {
        // Use query_one_mut for safe single entity access
        if let Ok((pos, vel, energy, age, state, genes)) = world.query_one_mut::<(
            &mut Position, &mut Velocity, &mut Energy, &mut Age, &mut AgentState, &Genes,
        )>(entity) {
            // Update agent
        }
    });
}
```

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
}
```

#### Initialization

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

        Ok(WebRenderer {
            canvas,
            ctx_2d,
            gl,
            use_webgl,
        })
    }
}
```

### WebGL Rendering

#### Rendering Pipeline

```rust
fn render_webgl(&mut self, simulation: &Simulation) {
    if let Some(gl) = &self.gl {
        // Clear the canvas
        gl.clear_color(0.1, 0.1, 0.18, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        // Get simulation data
        let agents = simulation.get_agents();
        let resources = simulation.get_resources();

        // Render agents
        for agent in &agents {
            self.render_agent_webgl(gl, agent);
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
fn render_agent_webgl(&self, gl: &WebGlRenderingContext, agent: &Agent) {
    let x = agent.x as f32;
    let y = agent.y as f32;
    let size = agent.genes.size as f32 * 3.0;

    // Convert agent color to RGB
    let hue = agent.genes.color_hue as f32;
    let (r, g, b) = hsl_to_rgb(hue, 70.0, 60.0);

    // Draw agent as a colored rectangle using scissor test
    gl.enable(WebGlRenderingContext::SCISSOR_TEST);
    gl.scissor(
        (x - size/2.0) as i32,
        (y - size/2.0) as i32,
        size as i32,
        size as i32,
    );
    gl.clear_color(r, g, b, 1.0);
    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    gl.disable(WebGlRenderingContext::SCISSOR_TEST);
}
```

### Canvas2D Rendering

#### Rendering Pipeline

```rust
fn render_canvas2d(&mut self, simulation: &Simulation) {
    if let Some(ctx) = &self.ctx_2d {
        // Clear the canvas
        ctx.set_fill_style(&"#1a1a2e".into());
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
    ctx.set_fill_style(&format!("hsl({}, {}%, {}%)", hue, saturation, lightness).into());
    ctx.begin_path();
    ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI).unwrap();
    ctx.fill();

    // Draw border
    ctx.set_stroke_style(&"#ffffff".into());
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

1. **Scissor Test**: Efficient rectangle rendering
2. **Batch Rendering**: Group similar operations
3. **Minimal State Changes**: Reduce WebGL state switches
4. **Efficient Clearing**: Use clear operations instead of draw calls

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

- **Web**: 1,000-10,000 agents for 60 FPS
- **Headless**: 100,000+ agents with speed multipliers
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
