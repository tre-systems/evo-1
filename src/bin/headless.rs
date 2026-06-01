#[cfg(not(target_arch = "wasm32"))]
use clap::Parser;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Parser, Debug)]
#[command(author, version, about = "Run BattleO as a native headless simulation")]
struct Args {
    #[arg(default_value_t = 2.0)]
    duration_minutes: f64,
    #[arg(default_value_t = 20.0)]
    speed_multiplier: f64,
    #[arg(default_value_t = 500)]
    initial_agents: usize,
    #[arg(default_value_t = 500)]
    initial_resources: usize,
    #[arg(default_value_t = 3000)]
    max_agents: usize,
    #[arg(default_value_t = 2000)]
    max_resources: usize,
    #[arg(long)]
    seed: Option<u64>,
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use battleo::headless::{HeadlessConfig, HeadlessSimulation};
    use std::time::Instant;

    println!("=== BattleO Headless Simulation ===");
    println!("Running high-performance evolutionary simulation...\n");

    let args = Args::parse();
    let config = HeadlessConfig {
        duration_minutes: args.duration_minutes,
        speed_multiplier: args.speed_multiplier,
        initial_agents: args.initial_agents,
        initial_resources: args.initial_resources,
        max_agents: args.max_agents,
        max_resources: args.max_resources,
        seed: args.seed,
    };

    println!("Configuration:");
    println!("  Duration: {:.2} minutes", config.duration_minutes);
    println!("  Speed multiplier: {}x", config.speed_multiplier);
    println!("  Initial agents: {}", config.initial_agents);
    println!("  Initial resources: {}", config.initial_resources);
    println!("  Max agents: {}", config.max_agents);
    println!("  Max resources: {}", config.max_resources);
    if let Some(seed) = config.seed {
        println!("  Seed: {seed}");
    }
    println!();

    let start_time = Instant::now();
    let mut simulation = HeadlessSimulation::new(config.clone());

    println!("Starting simulation...");
    let diagnostics = simulation.run();
    let total_time = start_time.elapsed();

    println!("\n=== Simulation Results ===");
    println!("Total runtime: {:.2}s", total_time.as_secs_f64());
    println!("Simulation time: {:.2}s", diagnostics.duration_seconds);
    println!("Total steps: {}", diagnostics.total_steps);
    println!("Steps per second: {:.1}", diagnostics.steps_per_second);
    println!(
        "Speed achieved: {:.1}x real-time",
        diagnostics.steps_per_second / 60.0
    );

    println!("\n=== Final Population ===");
    println!("Final agents: {}", diagnostics.final_stats.agent_count);
    println!(
        "Final resources: {}",
        diagnostics.final_stats.resource_count
    );
    println!("Total energy: {:.1}", diagnostics.final_stats.total_energy);
    println!("Average age: {:.1}", diagnostics.final_stats.average_age);
    println!("Max generation: {}", diagnostics.final_stats.max_generation);
    println!("Total kills: {}", diagnostics.final_stats.total_kills);
    println!(
        "Predators / prey: {} / {}",
        diagnostics.final_stats.predator_agents, diagnostics.final_stats.prey_agents
    );

    println!("\n=== Agent Behavior ===");
    println!("Seeking: {}", diagnostics.final_stats.seeking_agents);
    println!("Hunting: {}", diagnostics.final_stats.hunting_agents);
    println!("Feeding: {}", diagnostics.final_stats.feeding_agents);
    println!("Fleeing: {}", diagnostics.final_stats.fleeing_agents);
    println!("Fighting: {}", diagnostics.final_stats.fighting_agents);
    println!(
        "Reproducing: {}",
        diagnostics.final_stats.reproducing_agents
    );
    println!(
        "Ready to mate: {}",
        diagnostics.final_stats.reproduction_candidates
    );

    println!("\n=== Simulation Quality ===");
    println!("Quality score: {:.3}", diagnostics.simulation_quality_score);
    println!("Stable run: {}", diagnostics.is_stable);
    println!("Stability score: {:.3}", diagnostics.stability_score);
    println!("Extinction occurred: {}", diagnostics.extinction_occurred);
    println!("Population explosion: {}", diagnostics.population_explosion);
    println!(
        "Average generations: {:.1}",
        diagnostics.average_generations
    );
    println!("Total reproductions: {}", diagnostics.total_reproductions);
    println!("Total deaths: {}", diagnostics.total_deaths);

    println!("\n=== Performance Metrics ===");
    println!(
        "Average agent speed: {:.2}",
        diagnostics.final_stats.average_speed
    );
    println!(
        "Average sense range: {:.2}",
        diagnostics.final_stats.average_sense_range
    );
    println!(
        "Average aggression: {:.2}",
        diagnostics.final_stats.average_aggression
    );
    println!(
        "Average energy efficiency: {:.2}",
        diagnostics.final_stats.average_energy_efficiency
    );
    println!(
        "Average fitness: {:.3}",
        diagnostics.final_stats.average_fitness
    );

    println!("\n=== Quality Assessment ===");
    if diagnostics.simulation_quality_score > 0.8 {
        println!("✅ Excellent simulation quality!");
    } else if diagnostics.simulation_quality_score > 0.6 {
        println!("✅ Good simulation quality");
    } else if diagnostics.simulation_quality_score > 0.4 {
        println!("⚠️  Moderate simulation quality");
    } else {
        println!("❌ Poor simulation quality");
    }

    if diagnostics.extinction_occurred {
        println!("⚠️  Population went extinct");
    } else if diagnostics.population_explosion {
        println!("⚠️  Population exploded");
    } else {
        println!("✅ Stable population maintained");
    }

    if diagnostics.average_generations > 2.0 {
        println!("✅ Good evolutionary progress");
    } else if diagnostics.average_generations > 1.0 {
        println!("⚠️  Limited evolutionary progress");
    } else {
        println!("❌ No significant evolution");
    }

    println!("\n=== Summary ===");
    println!("BattleO headless simulation completed successfully!");
    println!(
        "The simulation ran {:.1}x faster than real-time",
        diagnostics.steps_per_second / 60.0
    );
    println!(
        "Processed {} simulation steps in {:.2} seconds",
        diagnostics.total_steps,
        total_time.as_secs_f64()
    );
    println!(
        "Final population: {} agents, {} resources",
        diagnostics.final_stats.agent_count, diagnostics.final_stats.resource_count
    );
}

#[cfg(target_arch = "wasm32")]
fn main() {
    println!("Headless simulation is not available in WASM mode.");
    println!("Use the web interface instead.");
}
