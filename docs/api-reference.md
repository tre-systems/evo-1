# API Reference

Complete API documentation for the BattleO framework.

## Core Types

### Simulation

```rust
pub struct Simulation {
    ecs_world: EcsWorld,
    config: SimulationConfig,
    time: f64,
    resource_spawn_timer: f64,
}
```

**Methods:**
- `new() -> Self` - Create new simulation with default config
- `new_with_config(config: SimulationConfig) -> Self` - Create simulation with custom population and limit config
- `update()` - Update simulation for one frame
- `add_agent(x: f64, y: f64)` - Add agent at position
- `add_resource(x: f64, y: f64)` - Add resource at position
- `reset()` - Reset simulation to initial state
- `get_stats() -> SimulationStats` - Get current simulation statistics
- `get_agents() -> Vec<Agent>` - Get all agents (legacy format)
- `get_resources() -> Vec<Resource>` - Get all resources (legacy format)

### SimulationConfig

```rust
pub struct SimulationConfig {
    pub width: f64,                    // Simulation world width
    pub height: f64,                   // Simulation world height
    pub max_agents: usize,             // Maximum agents allowed
    pub max_resources: usize,          // Maximum resources allowed
    pub initial_agents: usize,         // Initial agent count
    pub initial_resources: usize,      // Initial resource count
    pub resource_spawn_rate: f64,      // Reserved for automatic spawning
}
```

Initial counts are applied at construction and reset time, clamped to the configured maximums.

### SimulationStats

```rust
pub struct SimulationStats {
    pub agent_count: usize,            // Current agent count
    pub resource_count: usize,         // Current resource count
    pub total_energy: f64,             // Total energy across all agents
    pub average_age: f64,              // Average agent age
    pub average_speed: f64,            // Average agent speed
    pub average_size: f64,             // Average agent size
    pub average_aggression: f64,       // Average agent aggression
    pub average_sense_range: f64,      // Average agent sense range
    pub average_energy_efficiency: f64, // Average energy efficiency
    pub max_generation: u32,           // Highest generation reached
    pub total_kills: u32,              // Total kills across all agents
    pub average_fitness: f64,          // Average fitness (energy/max_energy)
}
```

## WASM API

### BattleSimulation

```javascript
class BattleSimulation {
    constructor(canvas_id?: string)
    
    // Core methods
    update() -> void
    get_stats() -> SimulationStats
    add_agent(x: number, y: number) -> void
    add_resource(x: number, y: number) -> void
    reset() -> void
    get_rendering_mode() -> string
}
```

**Usage:**
```javascript
import init, { BattleSimulation } from "./pkg/battleo.js";

async function main() {
    await init();
    
    // Create simulation with rendering
    const simulation = new BattleSimulation("canvas");
    
    // Run simulation loop
    function animate() {
        simulation.update();
        requestAnimationFrame(animate);
    }
    animate();
}
```

### ParallelProcessor

```javascript
class ParallelProcessor {
    constructor() -> ParallelProcessor
    
    // Parallel processing
    initialize() -> Promise<void>
    is_initialized() -> boolean
    get_worker_count() -> number
    is_rayon_available() -> boolean
}
```

**Usage:**
```javascript
import init, { ParallelProcessor } from "./pkg/battleo.js";

async function main() {
    await init();
    
    const processor = new ParallelProcessor();
    await processor.initialize();
    
    if (processor.is_rayon_available()) {
        console.log("Parallel processing enabled!");
    }
}
```

### HeadlessSimulation (Native Only)

```rust
#[cfg(not(target_arch = "wasm32"))]
pub struct HeadlessSimulation {
    simulation: Simulation,
    config: HeadlessConfig,
    diagnostics: SimulationDiagnostics,
}
```

**Methods:**
- `new(config: HeadlessConfig) -> Self` - Create headless simulation
- `run() -> SimulationDiagnostics` - Run simulation and return diagnostics
- `get_current_stats() -> SimulationStats` - Get current statistics

## ECS Components

### Core Components

```rust
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

#[derive(Component, Clone, Debug)]
pub struct Size {
    pub value: f64,
}
```

### Agent Components

```rust
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
    pub speed: f64,                    // Movement speed
    pub sense_range: f64,              // Detection range
    pub size: f64,                     // Physical size
    pub energy_efficiency: f64,        // Energy consumption rate
    pub reproduction_threshold: f64,   // Energy needed to reproduce
    pub mutation_rate: f64,            // Gene mutation probability
    pub aggression: f64,               // Aggressive behavior
    pub color_hue: f64,                // Visual color (0-360)
    pub is_predator: bool,             // Predator flag
    pub hunting_speed: f64,            // Speed when hunting
    pub attack_power: f64,             // Attack strength
    pub defense: f64,                  // Defense capability
    pub stealth: f64,                  // Stealth ability
    pub pack_mentality: f64,           // Group behavior
    pub territory_size: f64,           // Territory range
    pub metabolism: f64,               // Energy consumption
    pub intelligence: f64,             // Decision making
    pub stamina: f64,                  // Endurance
}

#[derive(Component)]
pub struct AgentTag;  // Marker component
```

### Resource Components

```rust
#[derive(Component, Clone, Debug)]
pub struct Resource {
    pub energy: f64,                   // Current energy
    pub max_energy: f64,               // Maximum energy
    pub growth_rate: f64,              // Energy regeneration rate
    pub regeneration_rate: f64,        // Spawn regeneration rate
    pub age: f64,                      // Resource age
    pub target_energy: f64,            // Target energy level
    pub is_spawning: bool,             // Currently spawning
    pub spawn_fade: f64,               // Spawn animation
    pub is_depleting: bool,            // Currently being consumed
    pub deplete_fade: f64,             // Depletion animation
}

#[derive(Component)]
pub struct ResourceTag;  // Marker component
```

### Agent States

```rust
#[derive(Clone, Debug)]
pub enum AgentStateEnum {
    Seeking,    // Looking for resources
    Hunting,    // Moving toward target
    Feeding,    // Consuming resource
    Reproducing, // Creating offspring
    Fighting,   // Combat with other agents
    Fleeing,    // Running from danger
}
```

## ECS World

### EcsWorld

```rust
pub struct EcsWorld {
    pub world: World,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    pub spatial_grid: SpatialGrid,
}
```

**Methods:**
- `new(canvas_width: f64, canvas_height: f64) -> Self` - Create new ECS world
- `new_with_population(canvas_width, canvas_height, max_agents, max_resources, initial_agents, initial_resources) -> Self` - Create an ECS world with explicit limits and initial population
- `add_agent(x: f64, y: f64)` - Spawn agent at position
- `add_resource(x: f64, y: f64)` - Spawn resource at position
- `update_spatial_grid()` - Update spatial partitioning
- `handle_death()` - Remove dead entities
- `handle_reproduction()` - Process reproduction
- `spawn_resource()` - Create new resource
- `update_single_agent(entity, delta_time, resources, width, height)` - Update specific agent
- `get_agent_count() -> usize` - Count agents
- `get_resource_count() -> usize` - Count resources
- `get_agents() -> Vec<(Position, Velocity, Energy, Age, AgentState, Genes, Size)>` - Get all agents
- `get_resources() -> Vec<(Position, Resource, Size)>` - Get all resources
- `reset()` - Clear all entities
- `reset_with_population(initial_agents, initial_resources)` - Reset with explicit initial population, clamped to max limits

### SpatialGrid

```rust
pub struct SpatialGrid {
    pub cell_size: f64,
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<Vec<hecs::Entity>>>,
}
```

**Methods:**
- `new(canvas_width: f64, canvas_height: f64, cell_size: f64) -> Self` - Create spatial grid
- `clear()` - Clear all cells
- `get_cell(x: f64, y: f64) -> (usize, usize)` - Get grid cell for position
- `add_entity(entity: hecs::Entity, x: f64, y: f64)` - Add entity to grid
- `get_nearby_entities(x: f64, y: f64, radius: f64) -> Vec<hecs::Entity>` - Get entities in range

## Headless Configuration

### HeadlessConfig

```rust
pub struct HeadlessConfig {
    pub duration_minutes: f64,         // Simulation duration
    pub speed_multiplier: f64,         // Speed multiplier (1.0 = real-time)
    pub initial_agents: usize,         // Starting agent count
    pub initial_resources: usize,      // Starting resource count
    pub max_agents: usize,             // Maximum agents
    pub max_resources: usize,          // Maximum resources
}
```

### SimulationDiagnostics

```rust
pub struct SimulationDiagnostics {
    pub duration_seconds: f64,         // Actual runtime
    pub total_steps: usize,            // Total simulation steps
    pub steps_per_second: f64,         // Performance metric
    pub final_stats: SimulationStats,  // Final population stats
    pub stability_score: f64,          // Population stability
    pub is_stable: bool,               // Stable population flag
    pub extinction_occurred: bool,     // Extinction flag
    pub population_explosion: bool,    // Population explosion flag
    pub average_generations: f64,      // Average generations
    pub total_reproductions: usize,    // Total reproductions
    pub total_deaths: usize,           // Total deaths
    pub simulation_quality_score: f64, // Overall quality score
}
```

## Rendering

### WebRenderer

```rust
pub struct WebRenderer {
    canvas: HtmlCanvasElement,
    ctx_2d: Option<CanvasRenderingContext2d>,
    gl: Option<WebGlRenderingContext>,
    use_webgl: bool,
}
```

**Methods:**
- `new(canvas_id: &str) -> Result<Self, JsValue>` - Create renderer
- `render(simulation: &Simulation)` - Render simulation
- `get_rendering_mode() -> String` - Get current rendering mode

## Utility Functions

### Parallel Processing

```rust
// Check if parallel processing is available
pub fn is_rayon_available() -> bool

// Set parallel processing availability
pub fn set_rayon_available(available: bool)

// Get optimal worker count
fn get_optimal_worker_count() -> usize
```

### Color Conversion

```rust
// Convert HSL to RGB
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32)
```

## Error Handling

### Common Errors

- **Canvas not found**: Ensure canvas element exists with correct ID
- **WebGL not supported**: Falls back to Canvas2D automatically
- **SharedArrayBuffer not available**: Parallel processing disabled
- **Thread pool initialization failed**: Falls back to sequential processing

### Error Recovery

```javascript
// Check for errors and provide fallbacks
try {
    const simulation = new BattleSimulation("canvas");
} catch (error) {
    console.warn("Failed to create simulation:", error);
    // Provide fallback or error message
}

// Check parallel processing availability
if (!processor.is_rayon_available()) {
    console.warn("Using sequential processing");
}
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

### Simulation Setup

1. **Start small**: Begin with 100-500 agents
2. **Monitor performance**: Watch frame rates and memory usage
3. **Tune parameters**: Adjust reproduction and death rates
4. **Use headless mode**: For parameter tuning and experiments

### Web Integration

1. **Initialize parallel processing**: Always call `processor.initialize()`
2. **Handle errors gracefully**: Provide fallbacks for unsupported features
3. **Monitor performance**: Track frame rates in production
4. **Optimize rendering**: Use WebGL when available

### ECS Usage

1. **Use marker components**: For efficient filtering
2. **Batch operations**: Group related entity operations
3. **Optimize queries**: Only query needed components
4. **Use spatial partitioning**: For proximity queries

## Examples

### Basic Web Setup

```javascript
import init, { BattleSimulation, ParallelProcessor } from "./pkg/battleo.js";

async function main() {
    await init();
    
    // Initialize parallel processing
    const processor = new ParallelProcessor();
    await processor.initialize();
    
    // Create simulation
    const simulation = new BattleSimulation("canvas");
    
    // Add initial population
    for (let i = 0; i < 100; i++) {
        simulation.add_agent(Math.random() * 800, Math.random() * 600);
        simulation.add_resource(Math.random() * 800, Math.random() * 600);
    }
    
    // Run simulation
    function animate() {
        simulation.update();
        
        // Get statistics
        const stats = simulation.get_stats();
        console.log(`Agents: ${stats.agent_count}, Resources: ${stats.resource_count}`);
        
        requestAnimationFrame(animate);
    }
    animate();
}

main();
```

### Headless Experiment

```rust
use battleo::headless::{HeadlessSimulation, HeadlessConfig};

fn main() {
    let config = HeadlessConfig {
        duration_minutes: 5.0,
        speed_multiplier: 10.0,
        initial_agents: 500,
        initial_resources: 500,
        max_agents: 3000,
        max_resources: 2000,
    };
    
    let mut simulation = HeadlessSimulation::new(config);
    let diagnostics = simulation.run();
    
    println!("Quality Score: {:.3}", diagnostics.simulation_quality_score);
    println!("Steps per second: {:.1}", diagnostics.steps_per_second);
    println!("Final agents: {}", diagnostics.final_stats.agent_count);
}
```

### Custom ECS System

```rust
use battleo::ecs::EcsWorld;

fn custom_movement_system(world: &mut EcsWorld, delta_time: f64) {
    for (entity, (pos, vel)) in world.world.query_mut::<(&mut Position, &Velocity)>().iter() {
        // Custom movement logic
        pos.x += vel.dx * delta_time * 1.5; // 1.5x speed multiplier
        pos.y += vel.dy * delta_time * 1.5;
        
        // Wrap around screen
        if pos.x < 0.0 { pos.x = 800.0; }
        if pos.x > 800.0 { pos.x = 0.0; }
        if pos.y < 0.0 { pos.y = 600.0; }
        if pos.y > 600.0 { pos.y = 0.0; }
    }
}
```
