use crate::agent::Agent;
use crate::ecs::EcsWorld;
use crate::resource::Resource;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// RUNTIME CAPABILITIES
// ============================================================================

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeCapabilities {
    pub parallel_resources: bool,
}

impl RuntimeCapabilities {
    pub fn native_parallel() -> Self {
        Self {
            parallel_resources: true,
        }
    }
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
    // Event and resource diagnostics
    pub total_resource_energy: f64,
    pub average_resource_energy: f64,
    pub resources_being_consumed: usize,
    pub consumption_events_this_frame: usize,
    pub total_consumption_events: usize,
    pub birth_events_this_frame: usize,
    pub death_events_this_frame: usize,
    pub kill_events_this_frame: usize,
    pub total_birth_events: usize,
    pub total_death_events: usize,
    pub average_agent_energy: f64,
    pub agents_with_targets: usize,
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
            // Event and resource diagnostics
            total_resource_energy: 0.0,
            average_resource_energy: 0.0,
            resources_being_consumed: 0,
            consumption_events_this_frame: 0,
            total_consumption_events: 0,
            birth_events_this_frame: 0,
            death_events_this_frame: 0,
            kill_events_this_frame: 0,
            total_birth_events: 0,
            total_death_events: 0,
            average_agent_energy: 0.0,
            agents_with_targets: 0,
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
        }
    }
}

// ============================================================================
// CORE SIMULATION
// ============================================================================

pub struct Simulation {
    ecs_world: EcsWorld,
    config: SimulationConfig,
    runtime_capabilities: RuntimeCapabilities,
    time: f64,
}

impl Default for Simulation {
    fn default() -> Self {
        Self::new()
    }
}

impl Simulation {
    pub fn new() -> Self {
        let config = SimulationConfig::default();
        let ecs_world = EcsWorld::new_with_population(
            config.width,
            config.height,
            config.max_agents,
            config.max_resources,
            config.initial_agents,
            config.initial_resources,
        );

        Self {
            ecs_world,
            config,
            runtime_capabilities: RuntimeCapabilities::default(),
            time: 0.0,
        }
    }

    pub fn new_with_config(config: SimulationConfig) -> Self {
        Self::new_with_config_and_capabilities(config, RuntimeCapabilities::default())
    }

    pub fn new_with_config_and_capabilities(
        config: SimulationConfig,
        runtime_capabilities: RuntimeCapabilities,
    ) -> Self {
        let ecs_world = EcsWorld::new_with_population(
            config.width,
            config.height,
            config.max_agents,
            config.max_resources,
            config.initial_agents,
            config.initial_resources,
        );

        Self {
            ecs_world,
            config,
            runtime_capabilities,
            time: 0.0,
        }
    }

    pub fn update(&mut self) {
        let delta_time = 10.0 / 60.0;
        self.time += delta_time;
        self.ecs_world.begin_frame();

        // Update spatial grid for efficient neighbor lookups
        self.ecs_world.update_spatial_grid();

        // Update resources (can be parallelized)
        if self.runtime_capabilities.parallel_resources {
            self.update_resources_parallel(delta_time);
        } else {
            self.update_resources_sequential(delta_time);
        }

        // Handle resource consumption (agents eating resources)
        self.ecs_world.handle_consumption();

        // Update agents through the spatial-grid optimized ECS pass.
        self.update_agents_spatial(delta_time);

        // Handle death and reproduction
        self.ecs_world.handle_death();
        self.ecs_world.handle_reproduction();

        self.ecs_world.handle_resource_depletion();
    }

    fn update_resources_parallel(&mut self, delta_time: f64) {
        // Collect all resource data for parallel processing
        let resource_data: Vec<_> = self
            .ecs_world
            .world
            .query::<&crate::ecs::Resource>()
            .iter()
            .filter(|(entity, _)| {
                self.ecs_world
                    .world
                    .get::<&crate::ecs::ResourceTag>(*entity)
                    .is_ok()
            })
            .map(|(entity, resource)| (entity, resource.clone()))
            .collect();

        // Calculate updates in parallel
        let updates: Vec<_> = resource_data
            .par_iter()
            .map(|(entity, resource)| {
                let mut updated_resource = resource.clone();
                updated_resource.update(delta_time);
                (*entity, updated_resource)
            })
            .collect();

        // Apply updates sequentially (to avoid borrowing issues)
        for (entity, updated_resource) in updates {
            if let Ok(mut resource) = self
                .ecs_world
                .world
                .get::<&mut crate::ecs::Resource>(entity)
            {
                *resource = updated_resource;
            }
        }
    }

    fn update_resources_sequential(&mut self, delta_time: f64) {
        for (_, resource) in self
            .ecs_world
            .world
            .query_mut::<&mut crate::ecs::Resource>()
        {
            resource.update(delta_time);
        }
    }

    fn update_agents_spatial(&mut self, delta_time: f64) {
        // Use the existing ECS world's optimized update method
        // This uses the spatial grid for efficient neighbor lookups
        let agent_entities: Vec<_> = self
            .ecs_world
            .world
            .query::<(
                &crate::ecs::Position,
                &crate::ecs::Velocity,
                &crate::ecs::Energy,
                &crate::ecs::Age,
                &crate::ecs::AgentState,
                &crate::ecs::Genes,
            )>()
            .iter()
            .filter(|(entity, _)| {
                self.ecs_world
                    .world
                    .get::<&crate::ecs::AgentTag>(*entity)
                    .is_ok()
            })
            .map(|(entity, _)| entity)
            .collect();

        // Update agents sequentially using the optimized ECS method
        for entity in agent_entities {
            self.ecs_world.update_single_agent_optimized(
                entity,
                delta_time,
                self.config.width,
                self.config.height,
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
        self.ecs_world
            .reset_with_population(self.config.initial_agents, self.config.initial_resources);
        self.time = 0.0;
    }

    pub fn set_runtime_capabilities(&mut self, runtime_capabilities: RuntimeCapabilities) {
        self.runtime_capabilities = runtime_capabilities;
    }

    pub fn runtime_capabilities(&self) -> RuntimeCapabilities {
        self.runtime_capabilities
    }

    pub fn get_stats(&self) -> SimulationStats {
        let agent_count = self.ecs_world.get_agent_count();
        let resource_count = self.ecs_world.get_resource_count();

        if agent_count == 0 {
            let resources_being_consumed = self.ecs_world.resources_consumed_this_frame();
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
                total_kills: self.ecs_world.total_kill_events as u32,
                average_fitness: 0.0,
                // Event and resource diagnostics
                total_resource_energy: 0.0,
                average_resource_energy: 0.0,
                resources_being_consumed,
                consumption_events_this_frame: self.ecs_world.consumption_events_this_frame,
                total_consumption_events: self.ecs_world.total_consumption_events,
                birth_events_this_frame: self.ecs_world.birth_events_this_frame,
                death_events_this_frame: self.ecs_world.death_events_this_frame,
                kill_events_this_frame: self.ecs_world.kill_events_this_frame,
                total_birth_events: self.ecs_world.total_birth_events,
                total_death_events: self.ecs_world.total_death_events,
                average_agent_energy: 0.0,
                agents_with_targets: 0,
            };
        }

        // Get all agents for statistics
        let agents = self.ecs_world.get_agents();

        let total_energy: f64 = agents
            .iter()
            .map(|(_, _, energy, _, _, _, _)| energy.current)
            .sum();
        let average_age: f64 = agents
            .iter()
            .map(|(_, _, _, age, _, _, _)| age.value)
            .sum::<f64>()
            / agent_count as f64;
        let average_speed: f64 = agents
            .iter()
            .map(|(_, _, _, _, _, genes, _)| genes.speed)
            .sum::<f64>()
            / agent_count as f64;
        let average_size: f64 = agents
            .iter()
            .map(|(_, _, _, _, _, genes, _)| genes.size)
            .sum::<f64>()
            / agent_count as f64;
        let average_aggression: f64 = agents
            .iter()
            .map(|(_, _, _, _, _, genes, _)| genes.aggression)
            .sum::<f64>()
            / agent_count as f64;
        let average_sense_range: f64 = agents
            .iter()
            .map(|(_, _, _, _, _, genes, _)| genes.sense_range)
            .sum::<f64>()
            / agent_count as f64;
        let average_energy_efficiency: f64 = agents
            .iter()
            .map(|(_, _, _, _, _, genes, _)| genes.energy_efficiency)
            .sum::<f64>()
            / agent_count as f64;
        let max_generation = agents
            .iter()
            .map(|(_, _, _, _, state, _, _)| state.generation)
            .max()
            .unwrap_or(0);
        let total_kills = self.ecs_world.total_kill_events as u32;
        let average_fitness: f64 = agents
            .iter()
            .map(|(_, _, energy, _, _, _, _)| energy.current / energy.max)
            .sum::<f64>()
            / agent_count as f64;

        // Get resource statistics
        let resources = self.ecs_world.get_resources();
        let total_resource_energy: f64 = resources
            .iter()
            .map(|(_, resource, _)| resource.energy)
            .sum();
        let average_resource_energy = if resource_count > 0 {
            total_resource_energy / resource_count as f64
        } else {
            0.0
        };

        // Count agents with targets
        let agents_with_targets = agents
            .iter()
            .filter(|(_, _, _, _, state, _, _)| {
                state.target_x.is_some() || state.target_y.is_some()
            })
            .count();
        let resources_being_consumed = self.ecs_world.resources_consumed_this_frame();

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
            // Event and resource diagnostics
            total_resource_energy,
            average_resource_energy,
            resources_being_consumed,
            consumption_events_this_frame: self.ecs_world.consumption_events_this_frame,
            total_consumption_events: self.ecs_world.total_consumption_events,
            birth_events_this_frame: self.ecs_world.birth_events_this_frame,
            death_events_this_frame: self.ecs_world.death_events_this_frame,
            kill_events_this_frame: self.ecs_world.kill_events_this_frame,
            total_birth_events: self.ecs_world.total_birth_events,
            total_death_events: self.ecs_world.total_death_events,
            average_agent_energy: total_energy / agent_count as f64,
            agents_with_targets,
        }
    }

    pub fn agent_count(&self) -> usize {
        self.ecs_world.get_agent_count()
    }

    pub fn resource_count(&self) -> usize {
        self.ecs_world.get_resource_count()
    }

    pub fn get_agents(&self) -> Vec<Agent> {
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
                genes: genes.clone(),
                target_x: state.target_x,
                target_y: state.target_y,
                state: match state.state {
                    crate::ecs::AgentStateEnum::Seeking => crate::agent::AgentState::Seeking,
                    crate::ecs::AgentStateEnum::Hunting => crate::agent::AgentState::Hunting,
                    crate::ecs::AgentStateEnum::Feeding => crate::agent::AgentState::Feeding,
                    crate::ecs::AgentStateEnum::Reproducing => {
                        crate::agent::AgentState::Reproducing
                    }
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
