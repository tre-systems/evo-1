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
pub struct BattleSimulation {
    simulation: simulation::Simulation,
    #[wasm_bindgen(skip)]
    renderer: Option<renderer::WebRenderer>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl BattleSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: Option<String>) -> Result<BattleSimulation, JsValue> {
        console_error_panic_hook::set_once();

        let simulation = simulation::Simulation::new();

        let renderer = if let Some(canvas_id) = canvas_id {
            Some(renderer::WebRenderer::new(&canvas_id)?)
        } else {
            None
        };

        Ok(BattleSimulation {
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
        serde_wasm_bindgen::to_value(&self.simulation.get_stats()).unwrap()
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
}

// ============================================================================
// PARALLEL PROCESSING (WASM with wasm-bindgen-rayon, native with rayon)
// ============================================================================

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct ParallelProcessor {
    initialized: bool,
    worker_count: usize,
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
            closure: None,
        }
    }

    pub fn initialize(&mut self) -> js_sys::Promise {
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
            let closure = Closure::wrap(Box::new(move |result: JsValue| {
                web_sys::console::log_1(&format!("Thread pool init result: {:?}", result).into());
                match result.as_f64() {
                    Some(_) => {
                        web_sys::console::log_1(
                            &format!("WASM thread pool initialized with {} workers", worker_count)
                                .into(),
                        );
                        simulation::set_rayon_available(true);
                    }
                    None => {
                        web_sys::console::warn_1(
                            &format!(
                                "Failed to initialize WASM thread pool. Result: {:?}",
                                result
                            )
                            .into(),
                        );
                        simulation::set_rayon_available(false);
                    }
                }
            }) as Box<dyn FnMut(JsValue)>);

            // Store the closure so it doesn't get dropped
            self.closure = Some(closure);
            self.initialized = true;
            init_thread_pool(self.worker_count).then(&self.closure.as_ref().unwrap())
        }

        #[cfg(not(all(target_arch = "wasm32", feature = "wasm-bindgen-rayon")))]
        {
            // Native target - initialize rayon thread pool
            use rayon::ThreadPoolBuilder;

            match ThreadPoolBuilder::new()
                .num_threads(self.worker_count)
                .build_global()
            {
                Ok(_) => {
                    simulation::set_rayon_available(true);
                    web_sys::console::log_1(
                        &format!(
                            "Native thread pool initialized with {} workers",
                            self.worker_count
                        )
                        .into(),
                    );
                }
                Err(_) => {
                    simulation::set_rayon_available(false);
                    web_sys::console::warn_1(&"Failed to initialize native thread pool".into());
                }
            }

            self.initialized = true;
            js_sys::Promise::resolve(&JsValue::NULL)
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn get_worker_count(&self) -> usize {
        self.worker_count
    }

    pub fn is_rayon_available(&self) -> bool {
        simulation::is_rayon_available()
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

#[cfg(target_arch = "wasm32")]
fn get_optimal_worker_count() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        // Use fewer workers for testing - start with 2
        2
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }
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
    simulation: simulation::Simulation,
    config: headless::HeadlessConfig,
}

#[cfg(not(target_arch = "wasm32"))]
impl HeadlessSimulation {
    pub fn new(config: headless::HeadlessConfig) -> Self {
        let simulation = simulation::Simulation::new_with_config(simulation::SimulationConfig {
            initial_agents: config.initial_agents,
            initial_resources: config.initial_resources,
            max_agents: config.max_agents,
            max_resources: config.max_resources,
            ..simulation::SimulationConfig::default()
        });

        Self { simulation, config }
    }

    pub fn run(&mut self) -> headless::SimulationDiagnostics {
        let mut headless_sim = headless::HeadlessSimulation::new(self.config.clone());
        headless_sim.run()
    }

    pub fn get_current_stats(&self) -> simulation::SimulationStats {
        self.simulation.get_stats()
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

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parallel_processing() {
        let mut processor = ParallelProcessor::new();
        let _ = processor.initialize();

        // Test that rayon is available after initialization
        assert!(processor.is_rayon_available() || !processor.is_initialized());
    }
}
