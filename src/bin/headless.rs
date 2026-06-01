#[cfg(not(target_arch = "wasm32"))]
use std::env;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use battleo::headless::{HeadlessConfig, HeadlessSimulation};
    use std::time::Instant;

    println!("=== BattleO Headless Simulation ===");
    println!("Running high-performance evolutionary simulation...\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let config = if args.len() >= 7 {
        // Custom configuration from command line
        HeadlessConfig {
            duration_minutes: args[1].parse().unwrap_or(2.0),
            speed_multiplier: args[2].parse().unwrap_or(20.0),
            initial_agents: args[3].parse().unwrap_or(500),
            initial_resources: args[4].parse().unwrap_or(500),
            max_agents: args[5].parse().unwrap_or(3000),
            max_resources: args[6].parse().unwrap_or(2000),
        }
    } else {
        // Default configuration
        HeadlessConfig {
            duration_minutes: 2.0,  // Run for 2 minutes
            speed_multiplier: 20.0, // 20x faster than real-time
            initial_agents: 500,    // Start with 500 agents
            initial_resources: 500, // Start with 500 resources
            max_agents: 3000,       // Maximum 3000 agents
            max_resources: 2000,    // Maximum 2000 resources
        }
    };

    println!("Configuration:");
    println!("  Duration: {:.2} minutes", config.duration_minutes);
    println!("  Speed multiplier: {}x", config.speed_multiplier);
    println!("  Initial agents: {}", config.initial_agents);
    println!("  Initial resources: {}", config.initial_resources);
    println!("  Max agents: {}", config.max_agents);
    println!("  Max resources: {}", config.max_resources);
    println!();

    // Create and run simulation
    let start_time = Instant::now();
    let mut simulation = HeadlessSimulation::new(config.clone());

    println!("Starting simulation...");
    let diagnostics = simulation.run();
    let total_time = start_time.elapsed();

    // Print results
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

    println!("\n=== Simulation Quality ===");
    println!("Quality score: {:.3}", diagnostics.simulation_quality_score);
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

    // Quality assessment
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
        "The simulation ran {}x faster than real-time",
        config.speed_multiplier
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
