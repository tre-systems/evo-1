use crate::simulation::{RuntimeCapabilities, Simulation, SimulationConfig, SimulationStats};
use serde::{Deserialize, Serialize};

// ============================================================================
// HEADLESS CONFIG
// ============================================================================

#[derive(Clone, Serialize, Deserialize)]
pub struct HeadlessConfig {
    pub duration_minutes: f64,
    pub speed_multiplier: f64,
    pub initial_agents: usize,
    pub initial_resources: usize,
    pub max_agents: usize,
    pub max_resources: usize,
    pub seed: Option<u64>,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            duration_minutes: 5.0,
            speed_multiplier: 10.0,
            initial_agents: 500,
            initial_resources: 500,
            max_agents: 3000,
            max_resources: 2000,
            seed: None,
        }
    }
}

// ============================================================================
// SIMULATION DIAGNOSTICS
// ============================================================================

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationDiagnostics {
    pub duration_seconds: f64,
    pub total_steps: usize,
    pub steps_per_second: f64,
    pub final_stats: SimulationStats,
    pub stability_score: f64,
    pub is_stable: bool,
    pub extinction_occurred: bool,
    pub population_explosion: bool,
    pub average_generations: f64,
    pub total_reproductions: usize,
    pub total_deaths: usize,
    pub simulation_quality_score: f64,
}

impl Default for SimulationDiagnostics {
    fn default() -> Self {
        Self {
            duration_seconds: 0.0,
            total_steps: 0,
            steps_per_second: 0.0,
            final_stats: SimulationStats::default(),
            stability_score: 0.0,
            is_stable: false,
            extinction_occurred: false,
            population_explosion: false,
            average_generations: 0.0,
            total_reproductions: 0,
            total_deaths: 0,
            simulation_quality_score: 0.0,
        }
    }
}

// ============================================================================
// HEADLESS SIMULATION RUNNER
// ============================================================================

pub struct HeadlessSimulation {
    simulation: Simulation,
    config: HeadlessConfig,
    diagnostics: SimulationDiagnostics,
}

impl HeadlessSimulation {
    pub fn new(config: HeadlessConfig) -> Self {
        // Initialize rayon for parallel processing
        let thread_count = std::thread::available_parallelism()
            .map(|threads| threads.get())
            .unwrap_or(1);
        if rayon::ThreadPoolBuilder::new()
            .num_threads(thread_count)
            .build_global()
            .is_ok()
        {
            println!("Initialized rayon with {thread_count} threads");
        } else {
            println!("Rayon thread pool already initialized; reusing global pool");
        }

        let simulation = Simulation::new_with_config_and_capabilities(
            SimulationConfig {
                initial_agents: config.initial_agents,
                initial_resources: config.initial_resources,
                max_agents: config.max_agents,
                max_resources: config.max_resources,
                seed: config.seed,
                ..SimulationConfig::default()
            },
            RuntimeCapabilities::native_parallel(),
        );

        Self {
            simulation,
            config,
            diagnostics: SimulationDiagnostics::default(),
        }
    }

    pub fn run(&mut self) -> SimulationDiagnostics {
        let start_time = std::time::Instant::now();
        let target_steps = self.target_steps();

        println!(
            "Starting headless simulation with {}x speed multiplier",
            self.config.speed_multiplier
        );
        println!(
            "Target duration: {:.2} minutes",
            self.config.duration_minutes
        );
        println!("Target steps: {target_steps}");

        let mut step_count = 0;
        while step_count < target_steps {
            self.simulation.update();
            step_count += 1;

            // Progress reporting
            if step_count % 10000 == 0 {
                let progress = (step_count as f64 / target_steps as f64) * 100.0;
                let elapsed = start_time.elapsed().as_secs_f64();
                let steps_per_sec = step_count as f64 / elapsed;
                println!(
                    "Progress: {progress:.1}% ({step_count}/{target_steps} steps, {steps_per_sec:.0} steps/sec)"
                );
            }

            let agent_count = self.simulation.agent_count();
            if agent_count == 0 || agent_count > self.config.max_agents {
                println!("Early termination at step {step_count}");
                break;
            }
        }

        // Finalize diagnostics
        let duration = start_time.elapsed();
        self.diagnostics.duration_seconds = duration.as_secs_f64();
        self.diagnostics.total_steps = step_count;
        self.diagnostics.steps_per_second = step_count as f64 / duration.as_secs_f64();
        self.diagnostics.final_stats = self.simulation.get_stats();

        // Calculate additional metrics
        self.calculate_diagnostics();

        self.diagnostics.clone()
    }

    fn calculate_diagnostics(&mut self) {
        let stats = &self.diagnostics.final_stats;

        // Check for extinction/explosion
        self.diagnostics.extinction_occurred = stats.agent_count == 0;
        self.diagnostics.population_explosion = stats.agent_count > self.config.max_agents;

        // Calculate average generations
        let agents = self.simulation.get_agents();
        if !agents.is_empty() {
            let total_generations: u32 = agents.iter().map(|a| a.generation).sum();
            self.diagnostics.average_generations = total_generations as f64 / agents.len() as f64;
        }

        // Calculate reproduction/death stats
        self.diagnostics.total_reproductions = stats.total_birth_events;
        self.diagnostics.total_deaths = stats.total_death_events;
        self.diagnostics.stability_score = self.calculate_stability_score(stats.agent_count);
        self.diagnostics.is_stable = self.diagnostics.stability_score >= 0.6;

        self.diagnostics.simulation_quality_score = self.calculate_quality_score();
    }

    fn calculate_quality_score(&self) -> f64 {
        let mut score = 0.0;
        let stats = &self.diagnostics.final_stats;

        let completion_ratio = self.diagnostics.total_steps as f64 / self.target_steps() as f64;
        score += completion_ratio.min(1.0) * 0.2;

        // Population health (25%)
        let final_agents = stats.agent_count;
        let target_agents = self.config.initial_agents;
        let agent_ratio = final_agents as f64 / target_agents as f64;
        if (0.5..=2.0).contains(&agent_ratio) {
            score += 0.25;
        } else if (0.3..=3.0).contains(&agent_ratio) {
            score += 0.15;
        }

        // Evolution progress (15%)
        if self.diagnostics.average_generations > 1.0 {
            score += 0.15;
        } else if self.diagnostics.average_generations > 0.5 {
            score += 0.10;
        }

        // Performance (10%)
        let target_steps_per_sec = 60.0 * self.config.speed_multiplier;
        let actual_steps_per_sec = self.diagnostics.steps_per_second;
        let performance_ratio = actual_steps_per_sec / target_steps_per_sec;
        score += performance_ratio.min(1.0) * 0.10;

        // Penalties
        if self.diagnostics.extinction_occurred {
            score -= 0.5;
        }
        if self.diagnostics.population_explosion {
            score -= 0.3;
        }

        score.clamp(0.0, 1.0)
    }

    fn calculate_stability_score(&self, final_agents: usize) -> f64 {
        if final_agents == 0 || self.config.initial_agents == 0 {
            return 0.0;
        }

        let ratio = final_agents as f64 / self.config.initial_agents as f64;
        (1.0 - (ratio - 1.0).abs() / 2.0).clamp(0.0, 1.0)
    }

    fn target_steps(&self) -> usize {
        ((self.config.duration_minutes * 60.0 * 60.0 * self.config.speed_multiplier) as usize)
            .max(1)
    }

    pub fn get_current_stats(&self) -> SimulationStats {
        self.simulation.get_stats()
    }

    pub fn print_summary(&self) {
        println!("\n=== Headless Simulation Summary ===");
        println!("Duration: {:.2}s", self.diagnostics.duration_seconds);
        println!("Total steps: {}", self.diagnostics.total_steps);
        println!("Steps per second: {:.1}", self.diagnostics.steps_per_second);
        println!("Speed multiplier: {}x", self.config.speed_multiplier);

        println!("\n=== Final Population ===");
        println!("Final agents: {}", self.diagnostics.final_stats.agent_count);
        println!(
            "Final resources: {}",
            self.diagnostics.final_stats.resource_count
        );
        println!(
            "Total energy: {:.1}",
            self.diagnostics.final_stats.total_energy
        );
        println!("Total kills: {}", self.diagnostics.final_stats.total_kills);
        println!(
            "Predators / prey: {} / {}",
            self.diagnostics.final_stats.predator_agents, self.diagnostics.final_stats.prey_agents
        );

        println!("\n=== Agent Behavior ===");
        println!("Seeking: {}", self.diagnostics.final_stats.seeking_agents);
        println!("Hunting: {}", self.diagnostics.final_stats.hunting_agents);
        println!("Feeding: {}", self.diagnostics.final_stats.feeding_agents);
        println!("Fleeing: {}", self.diagnostics.final_stats.fleeing_agents);
        println!("Fighting: {}", self.diagnostics.final_stats.fighting_agents);
        println!(
            "Reproducing: {}",
            self.diagnostics.final_stats.reproducing_agents
        );
        println!(
            "Ready to mate: {}",
            self.diagnostics.final_stats.reproduction_candidates
        );

        println!("\n=== Simulation Quality ===");
        println!(
            "Quality score: {:.3}",
            self.diagnostics.simulation_quality_score
        );
        println!("Stable run: {}", self.diagnostics.is_stable);
        println!("Stability score: {:.3}", self.diagnostics.stability_score);
        println!(
            "Extinction occurred: {}",
            self.diagnostics.extinction_occurred
        );
        println!(
            "Population explosion: {}",
            self.diagnostics.population_explosion
        );
        println!(
            "Average generations: {:.1}",
            self.diagnostics.average_generations
        );
        println!(
            "Total reproductions: {}",
            self.diagnostics.total_reproductions
        );
        println!("Total deaths: {}", self.diagnostics.total_deaths);
    }
}
