use crate::ecs::EcsWorld;
use crate::agent::Agent;
use crate::resource::Resource;
use crate::genes::Genes;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

// ============================================================================
// STATIC STATE FOR RAYON AVAILABILITY
// ============================================================================

static RAYON_AVAILABLE: AtomicBool = AtomicBool::new(false);

pub fn set_rayon_available(available: bool) {
    RAYON_AVAILABLE.store(available, Ordering::SeqCst);
}

pub fn is_rayon_available() -> bool {
    RAYON_AVAILABLE.load(Ordering::SeqCst)
}

// ============================================================================
// SIMULATION STATS
// ============================================================================

#[derive(Clone, Serialize, Deserialize)]
pub struct SimulationStats {
    pub agent_count: usize,
    pub resource_count: usize,
    pub total_energy: f64,
    pub average_age: f64,
    pub average_speed: f64,
    pub average_size: f64,
    pub average_aggression: f64,
    pub average_sense_range: f64,
    pub average_energy_efficiency: f64,
    pub max_generation: u32,
    pub total_kills: u32,
    pub average_fitness: f64,
}

impl Default for SimulationStats {
    fn default() -> Self {
        Self {
            agent_count: 0,
            resource_count: 0,
            total_energy: 0.0,
            average_age: 0.0,
            average_speed: 0.0,
            average_size: 0.0,
            average_aggression: 0.0,
            average_sense_range: 0.0,
            average_energy_efficiency: 0.0,
            max_generation: 0,
            total_kills: 0,
            average_fitness: 0.0,
        }
    }
}

// ============================================================================
// SIMULATION CONFIG
// ============================================================================

#[derive(Clone, Serialize)]
pub struct SimulationConfig {
    pub width: f64,
    pub height: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    pub initial_agents: usize,
    pub initial_resources: usize,
    pub resource_spawn_rate: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            width: 1000.0,
            height: 800.0,
            max_agents: 5000,
            max_resources: 2000,
            initial_agents: 500,
            initial_resources: 500,
            resource_spawn_rate: 0.2,
        }
    }
}

// ============================================================================
// CORE SIMULATION
// ============================================================================

pub struct Simulation {
    ecs_world: EcsWorld,
    config: SimulationConfig,
    time: f64,
    resource_spawn_timer: f64,
}

impl Simulation {
    pub fn new() -> Self {
        let config = SimulationConfig::default();
        let ecs_world = EcsWorld::new(config.width, config.height);

        Self {
            ecs_world,
            config,
            time: 0.0,
            resource_spawn_timer: 0.0,
        }
    }

    pub fn new_with_config(config: SimulationConfig) -> Self {
        let ecs_world = EcsWorld::new(config.width, config.height);

        Self {
            ecs_world,
            config,
            time: 0.0,
            resource_spawn_timer: 0.0,
        }
    }

    pub fn update(&mut self) {
        let delta_time = 1.0 / 60.0;
        self.time += delta_time;
        self.resource_spawn_timer += delta_time;

        // Update spatial grid for efficient neighbor lookups
        self.ecs_world.update_spatial_grid();

        // Update resources (can be parallelized)
        if is_rayon_available() {
            self.update_resources_parallel(delta_time);
        } else {
            self.update_resources_sequential(delta_time);
        }

        // Update agents (can be parallelized)
        if is_rayon_available() {
            self.update_agents_parallel(delta_time);
        } else {
            self.update_agents_sequential(delta_time);
        }

        // Handle death and reproduction
        self.ecs_world.handle_death();
        self.ecs_world.handle_reproduction();

        // Spawn new resources
        if self.resource_spawn_timer >= 1.0 / self.config.resource_spawn_rate {
            if self.ecs_world.get_resource_count() < self.config.max_resources {
                self.ecs_world.spawn_resource();
            }
            self.resource_spawn_timer = 0.0;
        }
    }

    fn update_resources_parallel(&mut self, delta_time: f64) {
        // Collect all resource data for parallel processing
        let resource_data: Vec<_> = self.ecs_world.world
            .query::<&crate::ecs::Resource>()
            .iter()
            .filter(|(entity, _)| self.ecs_world.world.get::<&crate::ecs::ResourceTag>(*entity).is_ok())
            .map(|(entity, resource)| (entity, resource.clone()))
            .collect();

        // Calculate updates in parallel
        let updates: Vec<_> = resource_data.par_iter().map(|(entity, resource)| {
            let mut updated_resource = resource.clone();
            updated_resource.update(delta_time);
            (*entity, updated_resource)
        }).collect();

        // Apply updates sequentially (to avoid borrowing issues)
        for (entity, updated_resource) in updates {
            if let Ok(mut resource) = self.ecs_world.world.get::<&mut crate::ecs::Resource>(entity) {
                *resource = updated_resource;
            }
        }
    }

    fn update_resources_sequential(&mut self, delta_time: f64) {
        for (_, resource) in self.ecs_world.world.query_mut::<&mut crate::ecs::Resource>() {
            resource.update(delta_time);
        }
    }

    fn update_agents_parallel(&mut self, delta_time: f64) {
        // Collect all resources for efficient lookup (shared read-only data)
        let resources: Vec<_> = self.ecs_world.world
            .query::<(&crate::ecs::Position, &crate::ecs::Resource)>()
            .iter()
            .filter(|(entity, _)| self.ecs_world.world.get::<&crate::ecs::ResourceTag>(*entity).is_ok())
            .map(|(_, (pos, res))| (pos.x, pos.y, res.clone()))
            .collect();

        // Collect all agent data for parallel processing
        let agent_data: Vec<_> = self.ecs_world.world
            .query::<(&crate::ecs::Position, &crate::ecs::Velocity, &crate::ecs::Energy, &crate::ecs::Age, &crate::ecs::AgentState, &crate::ecs::Genes)>()
            .iter()
            .filter(|(entity, _)| self.ecs_world.world.get::<&crate::ecs::AgentTag>(*entity).is_ok())
            .map(|(entity, (pos, vel, energy, age, state, genes))| {
                (entity, pos.clone(), vel.clone(), energy.clone(), age.clone(), state.clone(), genes.clone())
            })
            .collect();

        // Calculate updates in parallel using rayon
        let updates: Vec<_> = agent_data.par_iter().map(|(entity, pos, vel, energy, age, state, genes)| {
            // Calculate new position
            let new_x = (pos.x + vel.dx * delta_time).max(0.0).min(self.config.width);
            let new_y = (pos.y + vel.dy * delta_time).max(0.0).min(self.config.height);
            
            // Calculate energy consumption
            let energy_consumption = genes.metabolism * delta_time;
            let new_energy = (energy.current - energy_consumption).max(0.0);
            
            // Calculate age
            let new_age = age.value + delta_time;
            
            // Find nearest resource for AI behavior
            let mut nearest_distance = f64::INFINITY;
            let mut nearest_resource = None;
            
            for (res_x, res_y, res) in &resources {
                let distance = ((new_x - res_x).powi(2) + (new_y - res_y).powi(2)).sqrt();
                if distance < nearest_distance && distance <= genes.sense_range {
                    nearest_distance = distance;
                    nearest_resource = Some((*res_x, *res_y, res.clone()));
                }
            }
            
            // Calculate new velocity based on AI behavior
            let mut new_vel_x = vel.dx;
            let mut new_vel_y = vel.dy;
            
            if let Some((res_x, res_y, _)) = nearest_resource {
                // Move towards resource
                let dx = res_x - new_x;
                let dy = res_y - new_y;
                let distance = (dx.powi(2) + dy.powi(2)).sqrt();
                
                if distance > 0.0 {
                    new_vel_x = (dx / distance) * genes.speed;
                    new_vel_y = (dy / distance) * genes.speed;
                }
            } else {
                // Random movement if no resource nearby
                let angle = (age.value * 0.1) % (2.0 * std::f64::consts::PI);
                new_vel_x = angle.cos() * genes.speed * 0.5;
                new_vel_y = angle.sin() * genes.speed * 0.5;
            }
            
            (*entity, new_x, new_y, new_vel_x, new_vel_y, new_energy, new_age)
        }).collect();

        // Apply updates sequentially (to avoid borrowing issues)
        for (entity, new_x, new_y, new_vel_x, new_vel_y, new_energy, new_age) in updates {
            if let Ok(mut pos) = self.ecs_world.world.get::<&mut crate::ecs::Position>(entity) {
                pos.x = new_x;
                pos.y = new_y;
            }
            if let Ok(mut vel) = self.ecs_world.world.get::<&mut crate::ecs::Velocity>(entity) {
                vel.dx = new_vel_x;
                vel.dy = new_vel_y;
            }
            if let Ok(mut energy) = self.ecs_world.world.get::<&mut crate::ecs::Energy>(entity) {
                energy.current = new_energy;
            }
            if let Ok(mut age) = self.ecs_world.world.get::<&mut crate::ecs::Age>(entity) {
                age.value = new_age;
            }
        }
    }

    fn update_agents_sequential(&mut self, delta_time: f64) {
        // Collect all resources for efficient lookup
        let resources: Vec<_> = self.ecs_world.world
            .query::<(&crate::ecs::Position, &crate::ecs::Resource)>()
            .iter()
            .filter(|(entity, _)| self.ecs_world.world.get::<&crate::ecs::ResourceTag>(*entity).is_ok())
            .map(|(_, (pos, res))| (pos.x, pos.y, res.clone()))
            .collect();

        // Get all agent entities that need updating
        let agent_entities: Vec<_> = self.ecs_world.world
            .query::<(&crate::ecs::Position, &crate::ecs::Velocity, &crate::ecs::Energy, &crate::ecs::Age, &crate::ecs::AgentState, &crate::ecs::Genes)>()
            .iter()
            .filter(|(entity, _)| self.ecs_world.world.get::<&crate::ecs::AgentTag>(*entity).is_ok())
            .map(|(entity, _)| entity)
            .collect();

        // Update agents sequentially
        for entity in agent_entities {
            self.ecs_world.update_single_agent(
                entity, 
                delta_time, 
                &resources, 
                self.config.width, 
                self.config.height
            );
        }
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        if self.ecs_world.get_agent_count() < self.config.max_agents {
            self.ecs_world.add_agent(x, y);
        }
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        if self.ecs_world.get_resource_count() < self.config.max_resources {
            self.ecs_world.add_resource(x, y);
        }
    }

    pub fn reset(&mut self) {
        self.ecs_world.reset();
        self.time = 0.0;
        self.resource_spawn_timer = 0.0;
    }

    pub fn get_stats(&self) -> SimulationStats {
        let agent_count = self.ecs_world.get_agent_count();
        let resource_count = self.ecs_world.get_resource_count();

        if agent_count == 0 {
            return SimulationStats {
                agent_count: 0,
                resource_count,
                total_energy: 0.0,
                average_age: 0.0,
                average_speed: 0.0,
                average_size: 0.0,
                average_aggression: 0.0,
                average_sense_range: 0.0,
                average_energy_efficiency: 0.0,
                max_generation: 0,
                total_kills: 0,
                average_fitness: 0.0,
            };
        }

        // Get all agents for statistics
        let agents = self.ecs_world.get_agents();

        let total_energy: f64 = agents.iter().map(|(_, _, energy, _, _, _, _)| energy.current).sum();
        let average_age: f64 = agents.iter().map(|(_, _, _, age, _, _, _)| age.value).sum::<f64>() / agent_count as f64;
        let average_speed: f64 = agents.iter().map(|(_, _, _, _, _, genes, _)| genes.speed).sum::<f64>() / agent_count as f64;
        let average_size: f64 = agents.iter().map(|(_, _, _, _, _, genes, _)| genes.size).sum::<f64>() / agent_count as f64;
        let average_aggression: f64 = agents.iter().map(|(_, _, _, _, _, genes, _)| genes.aggression).sum::<f64>() / agent_count as f64;
        let average_sense_range: f64 = agents.iter().map(|(_, _, _, _, _, genes, _)| genes.sense_range).sum::<f64>() / agent_count as f64;
        let average_energy_efficiency: f64 = agents.iter().map(|(_, _, _, _, _, genes, _)| genes.energy_efficiency).sum::<f64>() / agent_count as f64;
        let max_generation = agents.iter().map(|(_, _, _, _, state, _, _)| state.generation).max().unwrap_or(0);
        let total_kills: u32 = agents.iter().map(|(_, _, _, _, state, _, _)| state.kills).sum();
        let average_fitness: f64 = agents.iter().map(|(_, _, energy, _, _, _, _)| energy.current / energy.max).sum::<f64>() / agent_count as f64;

        SimulationStats {
            agent_count,
            resource_count,
            total_energy,
            average_age,
            average_speed,
            average_size,
            average_aggression,
            average_sense_range,
            average_energy_efficiency,
            max_generation,
            total_kills,
            average_fitness,
        }
    }

    pub fn get_agents(&self) -> Vec<Agent> {
        // Convert ECS agents to legacy Agent format for compatibility
        self.ecs_world
            .get_agents()
            .into_iter()
            .map(|(pos, vel, energy, age, state, genes, _size)| Agent {
                x: pos.x,
                y: pos.y,
                dx: vel.dx,
                dy: vel.dy,
                energy: energy.current,
                max_energy: energy.max,
                age: age.value,
                genes: Genes {
                    speed: genes.speed,
                    sense_range: genes.sense_range,
                    size: genes.size,
                    energy_efficiency: genes.energy_efficiency,
                    reproduction_threshold: genes.reproduction_threshold,
                    mutation_rate: genes.mutation_rate,
                    aggression: genes.aggression,
                    color_hue: genes.color_hue,
                    is_predator: genes.is_predator,
                    hunting_speed: genes.hunting_speed,
                    attack_power: genes.attack_power,
                    defense: genes.defense,
                    stealth: genes.stealth,
                    pack_mentality: genes.pack_mentality,
                    territory_size: genes.territory_size,
                    metabolism: genes.metabolism,
                    intelligence: genes.intelligence,
                    stamina: genes.stamina,
                },
                target_x: state.target_x,
                target_y: state.target_y,
                state: match state.state {
                    crate::ecs::AgentStateEnum::Seeking => crate::agent::AgentState::Seeking,
                    crate::ecs::AgentStateEnum::Hunting => crate::agent::AgentState::Hunting,
                    crate::ecs::AgentStateEnum::Feeding => crate::agent::AgentState::Feeding,
                    crate::ecs::AgentStateEnum::Reproducing => crate::agent::AgentState::Reproducing,
                    crate::ecs::AgentStateEnum::Fighting => crate::agent::AgentState::Fighting,
                    crate::ecs::AgentStateEnum::Fleeing => crate::agent::AgentState::Fleeing,
                },
                last_reproduction: state.last_reproduction,
                kills: state.kills,
                generation: state.generation,
                death_fade: 0.0,
                death_reason: None,
                is_dying: false,
                spawn_fade: 0.0,
                spawn_position: None,
            })
            .collect()
    }

    pub fn get_resources(&self) -> Vec<Resource> {
        // Convert ECS resources to legacy Resource format for compatibility
        self.ecs_world
            .get_resources()
            .into_iter()
            .map(|(pos, ecs_resource, size)| Resource {
                x: pos.x,
                y: pos.y,
                energy: ecs_resource.energy,
                max_energy: ecs_resource.max_energy,
                size: size.value,
                growth_rate: ecs_resource.growth_rate,
                regeneration_rate: ecs_resource.regeneration_rate,
                age: ecs_resource.age,
                target_energy: ecs_resource.target_energy,
                is_spawning: ecs_resource.is_spawning,
                spawn_fade: ecs_resource.spawn_fade,
                is_depleting: ecs_resource.is_depleting,
                deplete_fade: ecs_resource.deplete_fade,
            })
            .collect()
    }

    pub fn get_config(&self) -> &SimulationConfig {
        &self.config
    }
} 