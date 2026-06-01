#[cfg(target_arch = "wasm32")]
use std::{cell::Cell, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Core simulation modules (shared between headless and web)
pub mod agent;
pub mod ecs;
pub mod genes;
pub mod resource;
pub mod simulation;

// Rendering modules (web only)
#[cfg(target_arch = "wasm32")]
pub mod renderer;

// Headless simulation (native only)
#[cfg(not(target_arch = "wasm32"))]
pub mod headless;

// ============================================================================
// CORE SIMULATION (Shared between headless and web)
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct EvoOneSimulation {
    simulation: simulation::Simulation,
    #[wasm_bindgen(skip)]
    renderer: Option<renderer::WebRenderer>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl EvoOneSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: Option<String>) -> Result<EvoOneSimulation, JsValue> {
        console_error_panic_hook::set_once();

        let simulation = simulation::Simulation::new();

        let renderer = if let Some(canvas_id) = canvas_id {
            Some(renderer::WebRenderer::new_fallback(&canvas_id)?)
        } else {
            None
        };

        Ok(EvoOneSimulation {
            simulation,
            renderer,
        })
    }

    pub fn update(&mut self) {
        self.simulation.update();

        if let Some(renderer) = &mut self.renderer {
            renderer.render(&self.simulation);
        }
    }

    pub fn get_stats(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.simulation.get_stats()).unwrap_or_else(|error| {
            web_sys::console::error_1(&format!("Failed to serialize stats: {error}").into());
            JsValue::NULL
        })
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        self.simulation.add_agent(x, y);
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        self.simulation.add_resource(x, y);
    }

    pub fn reset(&mut self) {
        self.simulation.reset();
    }

    pub fn get_rendering_mode(&self) -> String {
        if let Some(renderer) = &self.renderer {
            renderer.get_rendering_mode()
        } else {
            "No rendering (headless)".to_string()
        }
    }

    pub fn set_parallel_resources_enabled(&mut self, enabled: bool) {
        self.simulation
            .set_runtime_capabilities(simulation::RuntimeCapabilities {
                parallel_resources: enabled,
            });
    }

    pub fn is_parallel_resources_enabled(&self) -> bool {
        self.simulation.runtime_capabilities().parallel_resources
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = createEvoOneSimulation)]
pub async fn create_evo_one_simulation(
    canvas_id: Option<String>,
) -> Result<EvoOneSimulation, JsValue> {
    console_error_panic_hook::set_once();

    let simulation = simulation::Simulation::new();
    let renderer = if let Some(canvas_id) = canvas_id {
        Some(renderer::WebRenderer::new(&canvas_id).await?)
    } else {
        None
    };

    Ok(EvoOneSimulation {
        simulation,
        renderer,
    })
}

// ============================================================================
// PARALLEL PROCESSING (WASM with wasm-bindgen-rayon, native with rayon)
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct ParallelProcessor {
    initialized: bool,
    worker_count: usize,
    available: Rc<Cell<bool>>,
    #[wasm_bindgen(skip)]
    #[allow(dead_code)]
    closure: Option<wasm_bindgen::closure::Closure<dyn FnMut(JsValue)>>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl ParallelProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let worker_count = get_optimal_worker_count();

        Self {
            initialized: false,
            worker_count,
            available: Rc::new(Cell::new(false)),
            closure: None,
        }
    }

    pub fn initialize(&mut self) -> js_sys::Promise {
        if self.initialized {
            return js_sys::Promise::resolve(&JsValue::from_bool(self.available.get()));
        }

        #[cfg(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon"))]
        {
            use wasm_bindgen::closure::Closure;
            use wasm_bindgen_rayon::init_thread_pool;

            web_sys::console::log_1(
                &format!(
                    "Initializing WASM thread pool with {} workers",
                    self.worker_count
                )
                .into(),
            );

            let worker_count = self.worker_count;
            let available = self.available.clone();
            let closure = Closure::wrap(Box::new(move |result: JsValue| {
                web_sys::console::log_1(&format!("Thread pool init result: {:?}", result).into());
                available.set(true);
                web_sys::console::log_1(
                    &format!("WASM thread pool initialized with {worker_count} workers").into(),
                );
            }) as Box<dyn FnMut(JsValue)>);

            self.closure = Some(closure);
            self.initialized = true;
            if let Some(callback) = self.closure.as_ref() {
                init_thread_pool(self.worker_count).then(callback)
            } else {
                js_sys::Promise::reject(&"Thread pool callback was not retained".into())
            }
        }

        #[cfg(not(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon")))]
        {
            self.initialized = false;
            self.available.set(false);
            web_sys::console::warn_1(
                &"Threaded WASM support was not compiled into this package".into(),
            );
            js_sys::Promise::resolve(&JsValue::FALSE)
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    pub fn is_rayon_available(&self) -> bool {
        self.available.get()
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

#[cfg(target_arch = "wasm32")]
fn get_optimal_worker_count() -> usize {
    let cores = web_sys::window()
        .map(|window| window.navigator().hardware_concurrency() as usize)
        .unwrap_or(2);

    cores.clamp(1, 8)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

// ============================================================================
// HEADLESS SIMULATION (Native only)
// ============================================================================

#[cfg(not(target_arch = "wasm32"))]
pub struct HeadlessSimulation {
    inner: headless::HeadlessSimulation,
}

#[cfg(not(target_arch = "wasm32"))]
impl HeadlessSimulation {
    pub fn new(config: headless::HeadlessConfig) -> Self {
        Self {
            inner: headless::HeadlessSimulation::new(config),
        }
    }

    pub fn run(&mut self) -> headless::SimulationDiagnostics {
        self.inner.run()
    }

    pub fn get_current_stats(&self) -> simulation::SimulationStats {
        self.inner.get_current_stats()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulation_basic() {
        let mut sim = simulation::Simulation::new();

        // Add some agents and resources
        sim.add_agent(100.0, 100.0);
        sim.add_agent(200.0, 200.0);
        sim.add_resource(150.0, 150.0);

        // Run a few steps
        for _ in 0..100 {
            sim.update();
        }

        let stats = sim.get_stats();
        assert!(stats.agent_count > 0);
        assert!(stats.resource_count > 0);
    }

    #[test]
    fn test_simulation_config_controls_initial_population() {
        let config = simulation::SimulationConfig {
            initial_agents: 12,
            initial_resources: 34,
            max_agents: 20,
            max_resources: 40,
            ..simulation::SimulationConfig::default()
        };

        let sim = simulation::Simulation::new_with_config(config);
        let stats = sim.get_stats();

        assert_eq!(stats.agent_count, 12);
        assert_eq!(stats.resource_count, 34);
    }

    #[test]
    fn test_seeded_runs_are_reproducible() {
        let first = run_seeded_scenario(42, 240);
        let second = run_seeded_scenario(42, 240);

        assert_eq!(first.agent_count, second.agent_count);
        assert_eq!(first.resource_count, second.resource_count);
        assert_eq!(first.max_generation, second.max_generation);
        assert_eq!(first.total_kills, second.total_kills);
        assert_eq!(first.total_birth_events, second.total_birth_events);
        assert_eq!(first.total_death_events, second.total_death_events);
        assert_eq!(first.hunting_agents, second.hunting_agents);
        assert_eq!(first.fleeing_agents, second.fleeing_agents);
        assert_eq!(first.reproducing_agents, second.reproducing_agents);
        assert_eq!(first.predator_agents, second.predator_agents);
        assert_eq!(
            first.reproduction_candidates,
            second.reproduction_candidates
        );
        assert_eq!(
            first.average_speed.to_bits(),
            second.average_speed.to_bits()
        );
        assert_eq!(
            first.average_energy_efficiency.to_bits(),
            second.average_energy_efficiency.to_bits()
        );
    }

    #[test]
    fn test_seeded_ecology_reaches_later_generations() {
        let stats = run_seeded_scenario(7, 720);

        assert!(stats.agent_count > 0);
        assert!(stats.agent_count <= 90);
        assert!(stats.max_generation >= 3);
        assert!(stats.total_birth_events > 0);
        assert_eq!(stats.resource_count, 100);
    }

    #[test]
    fn test_seeded_battle_scenario_records_combat_pressure() {
        let stats = run_seeded_battle_scenario(101, 720);

        let state_count = stats.seeking_agents
            + stats.hunting_agents
            + stats.feeding_agents
            + stats.fleeing_agents
            + stats.fighting_agents
            + stats.reproducing_agents;

        assert!(stats.agent_count > 0);
        assert_eq!(stats.resource_count, 100);
        assert_eq!(state_count, stats.agent_count);
        assert!(stats.total_birth_events > 0);
        assert!(stats.total_kills > 0);
        assert!(stats.max_generation >= 2);
        assert!(stats.predator_agents > 0);
        assert!(stats.prey_agents > 0);
        assert!(stats.hunting_agents + stats.fighting_agents > 0);
        assert!(stats.fleeing_agents > 0);
    }

    fn run_seeded_scenario(seed: u64, steps: usize) -> simulation::SimulationStats {
        let config = simulation::SimulationConfig {
            initial_agents: 50,
            initial_resources: 100,
            max_agents: 150,
            max_resources: 150,
            seed: Some(seed),
            ..simulation::SimulationConfig::default()
        };
        let mut sim = simulation::Simulation::new_with_config(config);

        for _ in 0..steps {
            sim.update();
        }

        sim.get_stats()
    }

    fn run_seeded_battle_scenario(seed: u64, steps: usize) -> simulation::SimulationStats {
        let config = simulation::SimulationConfig {
            initial_agents: 100,
            initial_resources: 100,
            max_agents: 500,
            max_resources: 500,
            seed: Some(seed),
            ..simulation::SimulationConfig::default()
        };
        let mut sim = simulation::Simulation::new_with_config(config);

        for _ in 0..steps {
            sim.update();
        }

        sim.get_stats()
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parallel_processing() {
        let mut processor = ParallelProcessor::new();
        let _ = processor.initialize();

        // Test that rayon is available after initialization
        assert!(processor.is_rayon_available() || !processor.is_initialized());
    }
}
