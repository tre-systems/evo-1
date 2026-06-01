use hecs::World;
use rand::{prelude::*, rngs::StdRng, SeedableRng};
use std::collections::HashSet;

mod components;
mod spatial;

pub use components::*;
pub use spatial::SpatialGrid;

// ============================================================================
// WORLD MANAGEMENT
// ============================================================================

pub struct EcsWorld {
    pub world: World,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    pub target_agents: usize,
    pub spatial_grid: SpatialGrid,
    pub consumption_events_this_frame: usize,
    pub total_consumption_events: usize,
    pub birth_events_this_frame: usize,
    pub death_events_this_frame: usize,
    pub kill_events_this_frame: usize,
    pub total_birth_events: usize,
    pub total_death_events: usize,
    pub total_kill_events: usize,
    pub frame_events: Vec<FrameEvent>,
    rng: StdRng,
    seed: Option<u64>,
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
        Self::new_with_population_and_seed(
            canvas_width,
            canvas_height,
            max_agents,
            max_resources,
            initial_agents,
            initial_resources,
            None,
        )
    }

    pub fn new_with_population_and_seed(
        canvas_width: f64,
        canvas_height: f64,
        max_agents: usize,
        max_resources: usize,
        initial_agents: usize,
        initial_resources: usize,
        seed: Option<u64>,
    ) -> Self {
        let world = World::new();
        let spatial_grid = SpatialGrid::new(canvas_width, canvas_height, 50.0);

        let mut ecs_world = Self {
            world,
            canvas_width,
            canvas_height,
            max_agents,
            max_resources,
            target_agents: initial_agents.min(max_agents),
            spatial_grid,
            consumption_events_this_frame: 0,
            total_consumption_events: 0,
            birth_events_this_frame: 0,
            death_events_this_frame: 0,
            kill_events_this_frame: 0,
            total_birth_events: 0,
            total_death_events: 0,
            total_kill_events: 0,
            frame_events: Vec::new(),
            rng: Self::rng_from_seed(seed),
            seed,
        };

        ecs_world.spawn_initial_population(
            initial_agents.min(max_agents),
            initial_resources.min(max_resources),
        );

        ecs_world
    }

    fn rng_from_seed(seed: Option<u64>) -> StdRng {
        match seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        }
    }

    pub fn update(&mut self) {
        let delta_time = 10.0 / 60.0;
        self.begin_frame();

        self.update_spatial_grid();
        self.update_resources(delta_time);
        self.handle_consumption();
        self.update_spatial_grid();
        self.update_agents(delta_time);
        self.handle_death();
        self.handle_reproduction();
        self.handle_resource_depletion();
    }

    pub fn begin_frame(&mut self) {
        self.consumption_events_this_frame = 0;
        self.birth_events_this_frame = 0;
        self.death_events_this_frame = 0;
        self.kill_events_this_frame = 0;
        self.frame_events.clear();
    }

    pub fn resources_consumed_this_frame(&self) -> usize {
        self.frame_events
            .iter()
            .filter_map(|event| match event {
                FrameEvent::ResourceConsumed { resource, .. } => Some(*resource),
                _ => None,
            })
            .collect::<HashSet<_>>()
            .len()
    }

    pub fn handle_resource_depletion(&mut self) {
        let mut to_deplete = Vec::new();

        for (entity, resource) in self.world.query::<&Resource>().iter() {
            if self.world.get::<&ResourceTag>(entity).is_ok() && resource.energy <= 0.0 {
                to_deplete.push(entity);
            }
        }

        for entity in to_deplete {
            self.world.despawn(entity).ok();
        }
    }

    pub fn handle_consumption(&mut self) {
        let mut consumed_resources = Vec::new();

        for (agent_entity, (agent_pos, agent_genes)) in
            self.world.query::<(&Position, &Genes)>().iter()
        {
            if self.world.get::<&AgentTag>(agent_entity).is_err() {
                continue;
            }

            let consumption_radius = agent_genes.size * 5.0 + 4.0;
            for resource_entity in
                self.spatial_grid
                    .get_nearby_entities(agent_pos.x, agent_pos.y, consumption_radius)
            {
                if self.world.get::<&ResourceTag>(resource_entity).is_err() {
                    continue;
                }

                let Ok(resource_pos) = self.world.get::<&Position>(resource_entity) else {
                    continue;
                };
                let Ok(resource) = self.world.get::<&Resource>(resource_entity) else {
                    continue;
                };
                if !resource.is_available() {
                    continue;
                }

                let distance = ((agent_pos.x - resource_pos.x).powi(2)
                    + (agent_pos.y - resource_pos.y).powi(2))
                .sqrt();

                if distance <= consumption_radius {
                    consumed_resources.push((
                        agent_entity,
                        resource_entity,
                        agent_genes.energy_efficiency,
                    ));
                }
            }
        }

        for (agent_entity, resource_entity, energy_efficiency) in consumed_resources {
            if let Ok(mut resource) = self.world.get::<&mut Resource>(resource_entity) {
                let consumption_amount = (resource.energy * 0.8).min(100.0); // Consume 80% of resource energy, max 100
                let actual_consumed = resource.consume(consumption_amount);

                if actual_consumed <= 0.0 {
                    continue;
                }

                self.consumption_events_this_frame += 1;
                self.total_consumption_events += 1;
                self.frame_events.push(FrameEvent::ResourceConsumed {
                    agent: agent_entity,
                    resource: resource_entity,
                    amount: actual_consumed,
                });

                if let Ok(mut agent_energy) = self.world.get::<&mut Energy>(agent_entity) {
                    let energy_gain = actual_consumed * energy_efficiency;
                    agent_energy.current =
                        (agent_energy.current + energy_gain).min(agent_energy.max);
                }

                // If resource is now depleted, mark it for removal
                if resource.energy <= 0.0 {
                    resource.is_depleting = true;
                }
            }
        }

        let mut to_kill = Vec::new();
        let mut claimed_prey = HashSet::new();

        for (predator_entity, (predator_pos, predator_genes, predator_energy)) in
            self.world.query::<(&Position, &Genes, &Energy)>().iter()
        {
            if self.world.get::<&AgentTag>(predator_entity).is_err()
                || predator_genes.is_predator <= 0.5
                || predator_energy.current <= 50.0
            {
                continue;
            }

            let hunting_radius = predator_genes.hunting_speed * 3.0;
            for prey_entity in self.spatial_grid.get_nearby_entities(
                predator_pos.x,
                predator_pos.y,
                hunting_radius,
            ) {
                if prey_entity == predator_entity || claimed_prey.contains(&prey_entity) {
                    continue;
                }
                if self.world.get::<&AgentTag>(prey_entity).is_err() {
                    continue;
                }

                let Ok(prey_pos) = self.world.get::<&Position>(prey_entity) else {
                    continue;
                };
                let Ok(prey_genes) = self.world.get::<&Genes>(prey_entity) else {
                    continue;
                };
                let Ok(prey_energy) = self.world.get::<&Energy>(prey_entity) else {
                    continue;
                };
                if prey_genes.is_predator >= 0.5 || prey_energy.current <= 0.0 {
                    continue;
                }

                let distance = ((predator_pos.x - prey_pos.x).powi(2)
                    + (predator_pos.y - prey_pos.y).powi(2))
                .sqrt();

                if distance <= hunting_radius {
                    to_kill.push((predator_entity, prey_entity));
                    claimed_prey.insert(prey_entity);
                    break;
                }
            }
        }

        for (predator_entity, prey_entity) in to_kill {
            if let Ok(mut death_anim) = self.world.get::<&mut DeathAnimation>(prey_entity) {
                death_anim.is_dying = true;
                death_anim.reason = DeathReason::KilledByPredator;
            }
            if let Ok(mut prey_energy) = self.world.get::<&mut Energy>(prey_entity) {
                prey_energy.current = 0.0;
            }
            if let Ok(mut predator_state) = self.world.get::<&mut AgentState>(predator_entity) {
                predator_state.kills += 1;
            }
            if let Ok(mut predator_energy) = self.world.get::<&mut Energy>(predator_entity) {
                predator_energy.current = (predator_energy.current + 25.0).min(predator_energy.max);
            }

            self.kill_events_this_frame += 1;
            self.total_kill_events += 1;
            self.frame_events.push(FrameEvent::AgentKilled {
                predator: predator_entity,
                prey: prey_entity,
            });
        }
    }

    pub fn update_spatial_grid(&mut self) {
        self.spatial_grid.clear();

        // Add agents to spatial grid
        for (entity, pos) in self.world.query::<&Position>().iter() {
            if self.world.get::<&AgentTag>(entity).is_ok() {
                self.spatial_grid.add_entity(entity, pos.x, pos.y);
            }
        }

        // Add resources to spatial grid
        for (entity, pos) in self.world.query::<&Position>().iter() {
            if self.world.get::<&ResourceTag>(entity).is_ok() {
                self.spatial_grid.add_entity(entity, pos.x, pos.y);
            }
        }
    }

    fn update_resources(&mut self, delta_time: f64) {
        // Use query_mut for resources that need updating
        for (_, resource) in self.world.query_mut::<&mut Resource>() {
            resource.update(delta_time);
        }
    }

    fn update_agents(&mut self, delta_time: f64) {
        let canvas_width = self.canvas_width;
        let canvas_height = self.canvas_height;

        // Collect agent entities that need updating
        let agent_entities: Vec<_> = self
            .world
            .query::<(&Position, &Velocity, &Energy, &Age, &AgentState, &Genes)>()
            .iter()
            .filter(|(entity, _)| self.world.get::<&AgentTag>(*entity).is_ok())
            .map(|(entity, _)| entity)
            .collect();

        // Update each agent individually to avoid borrowing conflicts
        for entity in agent_entities {
            self.update_single_agent_optimized(entity, delta_time, canvas_width, canvas_height);
        }
    }

    pub fn update_single_agent_optimized(
        &mut self,
        entity: hecs::Entity,
        delta_time: f64,
        canvas_width: f64,
        canvas_height: f64,
    ) {
        let (sense_range, personal_space, pos_x, pos_y) = {
            let Ok(genes) = self.world.get::<&Genes>(entity) else {
                return;
            };
            let Ok(pos) = self.world.get::<&Position>(entity) else {
                return;
            };
            (genes.sense_range, genes.personal_space, pos.x, pos.y)
        };

        let nearby_entities = self
            .spatial_grid
            .get_nearby_entities(pos_x, pos_y, sense_range);

        let mut nearby_resources = Vec::new();
        for entity in nearby_entities {
            if self.world.get::<&ResourceTag>(entity).is_ok() {
                if let Ok(resource_pos) = self.world.get::<&Position>(entity) {
                    if let Ok(resource) = self.world.get::<&Resource>(entity) {
                        if resource.is_available() {
                            nearby_resources.push((
                                resource_pos.x,
                                resource_pos.y,
                                resource.energy,
                            ));
                        }
                    }
                }
            }
        }

        let mut avoidance_dx = 0.0;
        let mut avoidance_dy = 0.0;
        for neighbor in self
            .spatial_grid
            .get_nearby_entities(pos_x, pos_y, personal_space)
        {
            if neighbor == entity || self.world.get::<&AgentTag>(neighbor).is_err() {
                continue;
            }

            let Ok(neighbor_pos) = self.world.get::<&Position>(neighbor) else {
                continue;
            };

            let dx = pos_x - neighbor_pos.x;
            let dy = pos_y - neighbor_pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 0.0 && distance < personal_space {
                let strength = (personal_space - distance) / personal_space;
                avoidance_dx += (dx / distance) * strength;
                avoidance_dy += (dy / distance) * strength;
            }
        }

        let random_turn = self.rng.gen::<f64>() < 0.08;
        let random_turn_angle = self.rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let fallback_angle = self.rng.gen_range(0.0..2.0 * std::f64::consts::PI);

        let Ok((pos, vel, energy, age, state, genes)) = self.world.query_one_mut::<(
            &mut Position,
            &mut Velocity,
            &mut Energy,
            &mut Age,
            &mut AgentState,
            &Genes,
        )>(entity) else {
            return;
        };

        age.value += delta_time;

        let base_energy_cost = (genes.size * 0.08 + genes.speed * 0.04) * delta_time;
        let metabolism_factor = genes.metabolism;
        let environmental_factor = 1.0 + (pos.x / canvas_width + pos.y / canvas_height) * 0.001;
        let total_energy_cost = base_energy_cost * metabolism_factor * environmental_factor;
        energy.current -= total_energy_cost / genes.energy_efficiency;

        let mut best_target = None;
        let mut best_score = f64::NEG_INFINITY;

        for (rx, ry, resource_energy) in &nearby_resources {
            let distance = ((pos.x - rx).powi(2) + (pos.y - ry).powi(2)).sqrt();
            if distance <= genes.sense_range {
                let score = resource_energy / (distance + 1.0);
                if score > best_score {
                    best_score = score;
                    best_target = Some((*rx, *ry));
                }
            }
        }

        if let Some((tx, ty)) = best_target {
            state.target_x = Some(tx);
            state.target_y = Some(ty);
            state.state = AgentStateEnum::Hunting;

            let dx = tx - pos.x;
            let dy = ty - pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 0.0 {
                vel.dx = (dx / distance) * genes.speed;
                vel.dy = (dy / distance) * genes.speed;
            }
        } else {
            state.target_x = None;
            state.target_y = None;
            state.state = AgentStateEnum::Seeking;

            if random_turn {
                vel.dx = random_turn_angle.cos() * genes.speed;
                vel.dy = random_turn_angle.sin() * genes.speed;
            }

            let current_speed = (vel.dx * vel.dx + vel.dy * vel.dy).sqrt();
            if current_speed < genes.speed * 0.5 {
                vel.dx = fallback_angle.cos() * genes.speed;
                vel.dy = fallback_angle.sin() * genes.speed;
            }
        }

        vel.dx += avoidance_dx * genes.speed * 0.4;
        vel.dy += avoidance_dy * genes.speed * 0.4;

        let current_speed = (vel.dx * vel.dx + vel.dy * vel.dy).sqrt();
        if current_speed > genes.speed * 1.5 {
            vel.dx = (vel.dx / current_speed) * genes.speed * 1.5;
            vel.dy = (vel.dy / current_speed) * genes.speed * 1.5;
        }

        pos.x += vel.dx * delta_time;
        pos.y += vel.dy * delta_time;

        if pos.x < 0.0 {
            pos.x = canvas_width;
        }
        if pos.x > canvas_width {
            pos.x = 0.0;
        }
        if pos.y < 0.0 {
            pos.y = canvas_height;
        }
        if pos.y > canvas_height {
            pos.y = 0.0;
        }
    }

    pub fn handle_death(&mut self) {
        let mut to_despawn = Vec::new();

        for (entity, (energy, age, death_animation)) in self
            .world
            .query::<(&Energy, &Age, &DeathAnimation)>()
            .iter()
        {
            if self.world.get::<&AgentTag>(entity).is_err() {
                continue;
            }

            let reason = if death_animation.is_dying {
                Some(death_animation.reason.clone())
            } else if energy.current <= 0.0 {
                Some(DeathReason::Starvation)
            } else if age.value > 1000.0 {
                Some(DeathReason::OldAge)
            } else {
                None
            };

            if let Some(reason) = reason {
                to_despawn.push((entity, reason));
            }
        }

        for (entity, reason) in to_despawn {
            self.world.despawn(entity).ok();
            self.death_events_this_frame += 1;
            self.total_death_events += 1;
            self.frame_events.push(FrameEvent::AgentDied {
                agent: entity,
                reason,
            });
        }
    }

    pub fn handle_reproduction(&mut self) {
        if self.get_agent_count() >= self.reproduction_soft_cap() {
            return;
        }

        // Find all agents that can reproduce
        let mut potential_parents: Vec<_> = self
            .world
            .query::<(
                &crate::ecs::Position,
                &crate::ecs::Energy,
                &crate::ecs::Age,
                &crate::ecs::AgentState,
                &crate::ecs::Genes,
            )>()
            .iter()
            .filter(|(entity, _)| self.world.get::<&crate::ecs::AgentTag>(*entity).is_ok())
            .filter(|(_, (_, energy, age, state, genes))| {
                self.can_reproduce(energy, age, state, genes)
            })
            .map(|(entity, (pos, energy, age, state, genes))| {
                (
                    entity,
                    pos.clone(),
                    energy.clone(),
                    age.clone(),
                    state.clone(),
                    genes.clone(),
                )
            })
            .collect();

        // Early return if not enough potential parents
        if potential_parents.len() < 2 {
            return;
        }

        // Shuffle to randomize reproduction order
        potential_parents.shuffle(&mut self.rng);

        // Try to pair agents for reproduction
        let mut i = 0;
        while i < potential_parents.len() - 1
            && self.get_agent_count() < self.reproduction_soft_cap()
        {
            let (entity1, pos1, energy1, age1, state1, genes1) = &potential_parents[i];
            let (entity2, pos2, energy2, age2, state2, genes2) = &potential_parents[i + 1];

            // Check if agents are close enough to reproduce
            let distance = self.distance(pos1, pos2);
            if distance < 100.0 {
                // Increased reproduction radius
                // Calculate reproduction probability based on energy and age
                let energy_factor =
                    (energy1.current / energy1.max).min(energy2.current / energy2.max);
                let age_factor = (age1.value / 5.0).min(age2.value / 5.0).min(1.0); // Reduced age requirement
                let reproduction_chance = energy_factor * age_factor * 0.25;

                if self.rng.gen::<f64>() < reproduction_chance {
                    let mutation_rate = (genes1.mutation_rate + genes2.mutation_rate) / 2.0;
                    let offspring_genes =
                        genes1.inherit_from_with_rng(genes2, mutation_rate, &mut self.rng);
                    let offspring_x = (pos1.x + pos2.x) / 2.0;
                    let offspring_y = (pos1.y + pos2.y) / 2.0;
                    let offspring_generation = state1.generation.max(state2.generation) + 1;

                    let offspring = self.spawn_agent(
                        offspring_x,
                        offspring_y,
                        offspring_genes,
                        offspring_generation,
                    );
                    self.birth_events_this_frame += 1;
                    self.total_birth_events += 1;
                    self.frame_events.push(FrameEvent::AgentBorn {
                        agent: offspring,
                        generation: offspring_generation,
                    });

                    // Update parent reproduction timers
                    if let Ok(mut state1_mut) =
                        self.world.get::<&mut crate::ecs::AgentState>(*entity1)
                    {
                        state1_mut.last_reproduction = age1.value;
                    }
                    if let Ok(mut state2_mut) =
                        self.world.get::<&mut crate::ecs::AgentState>(*entity2)
                    {
                        state2_mut.last_reproduction = age2.value;
                    }

                    // Consume some energy from parents
                    if let Ok(mut energy1_mut) = self.world.get::<&mut crate::ecs::Energy>(*entity1)
                    {
                        energy1_mut.current = (energy1_mut.current - 25.0).max(15.0);
                    }
                    if let Ok(mut energy2_mut) = self.world.get::<&mut crate::ecs::Energy>(*entity2)
                    {
                        energy2_mut.current = (energy2_mut.current - 25.0).max(15.0);
                    }
                }
            }
            i += 2; // Skip both parents
        }
    }

    fn can_reproduce(&self, energy: &Energy, age: &Age, state: &AgentState, genes: &Genes) -> bool {
        energy.current >= genes.reproduction_threshold
            && energy.current / energy.max > 0.6
            && age.value > 8.0
            && age.value - state.last_reproduction > 10.0
    }

    fn reproduction_soft_cap(&self) -> usize {
        if self.target_agents == 0 {
            return self.max_agents;
        }

        (self.target_agents + (self.target_agents * 3 / 4)).min(self.max_agents)
    }

    fn distance(&self, pos1: &Position, pos2: &Position) -> f64 {
        ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt()
    }

    fn spawn_agent(&mut self, x: f64, y: f64, genes: Genes, generation: u32) -> hecs::Entity {
        let angle = self.rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let size_value = genes.size * 3.0;

        self.world.spawn((
            Position { x, y },
            Velocity {
                dx: angle.cos() * genes.speed,
                dy: angle.sin() * genes.speed,
            },
            Energy {
                current: 70.0,
                max: 120.0,
            },
            Age { value: 0.0 },
            genes,
            AgentState {
                state: AgentStateEnum::Seeking,
                target_x: None,
                target_y: None,
                last_reproduction: 0.0,
                kills: 0,
                generation,
            },
            DeathAnimation {
                fade: 0.0,
                reason: DeathReason::NaturalCauses,
                is_dying: false,
            },
            SpawnAnimation {
                fade: 0.0,
                spawn_position: Some((x, y)),
            },
            Size { value: size_value },
            AgentTag,
        ))
    }

    pub fn spawn_resource(&mut self) {
        let x = self.rng.gen_range(0.0..self.canvas_width);
        let y = self.rng.gen_range(0.0..self.canvas_height);

        let initial_energy = self.rng.gen_range(40.0..90.0);
        let max_energy = self.rng.gen_range(70.0..120.0);

        self.world.spawn((
            Position { x, y },
            Resource {
                energy: initial_energy,
                max_energy,
                size: 3.0,
                growth_rate: self.rng.gen_range(0.1..0.5),
                regeneration_rate: self.rng.gen_range(0.08..0.18),
                age: 0.0,
                target_energy: max_energy,
                is_spawning: false,
                spawn_fade: 1.0,
                is_depleting: false,
                deplete_fade: 0.0,
            },
            Size { value: 3.0 },
            ResourceTag,
        ));
    }

    fn spawn_initial_population(&mut self, initial_agents: usize, initial_resources: usize) {
        for _ in 0..initial_agents {
            let x = self.rng.gen_range(0.0..self.canvas_width);
            let y = self.rng.gen_range(0.0..self.canvas_height);
            let genes = self.generate_random_genes();
            self.spawn_agent(x, y, genes, 0);
        }

        for _ in 0..initial_resources {
            self.spawn_resource();
        }
    }

    fn generate_random_genes(&mut self) -> Genes {
        Genes::random_with_rng(&mut self.rng)
    }

    pub fn maintain_resource_floor(&mut self, target_resources: usize) {
        let target_resources = target_resources.min(self.max_resources);
        let deficit = target_resources.saturating_sub(self.get_resource_count());
        for _ in 0..deficit.min(8) {
            self.spawn_resource();
        }
    }

    pub fn get_agent_count(&self) -> usize {
        self.world.query::<&AgentTag>().iter().count()
    }

    pub fn get_resource_count(&self) -> usize {
        self.world.query::<&ResourceTag>().iter().count()
    }

    pub fn add_agent(&mut self, x: f64, y: f64) {
        if self.get_agent_count() < self.max_agents {
            let genes = self.generate_random_genes();
            self.spawn_agent(x, y, genes, 0);
        }
    }

    pub fn add_resource(&mut self, x: f64, y: f64) {
        if self.get_resource_count() < self.max_resources {
            let _entity = self.world.spawn((
                Position { x, y },
                Resource {
                    energy: 0.0,
                    max_energy: 60.0,
                    size: 3.0,
                    growth_rate: 0.3,
                    regeneration_rate: 0.05,
                    age: 0.0,
                    target_energy: 30.0,
                    is_spawning: true,
                    spawn_fade: 0.0,
                    is_depleting: false,
                    deplete_fade: 0.0,
                },
                Size { value: 3.0 },
                ResourceTag,
            ));
        }
    }

    pub fn reset(&mut self) {
        self.reset_with_population(100, 500);
    }

    pub fn reset_with_population(&mut self, initial_agents: usize, initial_resources: usize) {
        self.reset_with_population_and_seed(initial_agents, initial_resources, self.seed);
    }

    pub fn reset_with_population_and_seed(
        &mut self,
        initial_agents: usize,
        initial_resources: usize,
        seed: Option<u64>,
    ) {
        self.world = World::new();
        self.spatial_grid.clear();
        self.rng = Self::rng_from_seed(seed);
        self.seed = seed;
        self.target_agents = initial_agents.min(self.max_agents);
        self.consumption_events_this_frame = 0;
        self.total_consumption_events = 0;
        self.birth_events_this_frame = 0;
        self.death_events_this_frame = 0;
        self.kill_events_this_frame = 0;
        self.total_birth_events = 0;
        self.total_death_events = 0;
        self.total_kill_events = 0;
        self.frame_events.clear();
        self.spawn_initial_population(
            initial_agents.min(self.max_agents),
            initial_resources.min(self.max_resources),
        );
    }

    pub fn get_agents(&self) -> Vec<(Position, Velocity, Energy, Age, AgentState, Genes, Size)> {
        self.world
            .query::<(
                &Position,
                &Velocity,
                &Energy,
                &Age,
                &AgentState,
                &Genes,
                &Size,
            )>()
            .iter()
            .filter(|(entity, _)| self.world.get::<&AgentTag>(*entity).is_ok())
            .map(|(_, (pos, vel, energy, age, state, genes, size))| {
                (
                    pos.clone(),
                    vel.clone(),
                    energy.clone(),
                    age.clone(),
                    state.clone(),
                    genes.clone(),
                    size.clone(),
                )
            })
            .collect()
    }

    pub fn get_resources(&self) -> Vec<(Position, Resource, Size)> {
        self.world
            .query::<(&Position, &Resource, &Size)>()
            .iter()
            .filter(|(entity, _)| self.world.get::<&ResourceTag>(*entity).is_ok())
            .map(|(_, (pos, resource, _))| {
                (
                    pos.clone(),
                    resource.clone(),
                    Size {
                        value: resource.size,
                    },
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predator_kill_records_events_and_removes_prey() {
        let mut world = EcsWorld::new_with_population(100.0, 100.0, 10, 10, 0, 0);

        let mut predator_genes = Genes::new();
        predator_genes.is_predator = 1.0;
        predator_genes.hunting_speed = 10.0;

        let mut prey_genes = Genes::new();
        prey_genes.is_predator = 0.0;

        let predator = world.spawn_agent(50.0, 50.0, predator_genes, 0);
        let prey = world.spawn_agent(52.0, 50.0, prey_genes, 0);

        world
            .world
            .get::<&mut Energy>(predator)
            .expect("predator energy")
            .current = 80.0;

        world.begin_frame();
        world.update_spatial_grid();
        world.handle_consumption();

        assert_eq!(world.kill_events_this_frame, 1);
        assert_eq!(world.total_kill_events, 1);

        world.handle_death();

        assert_eq!(world.get_agent_count(), 1);
        assert!(world.world.get::<&AgentTag>(predator).is_ok());
        assert!(world.world.get::<&AgentTag>(prey).is_err());
        assert_eq!(world.death_events_this_frame, 1);
        assert_eq!(world.total_death_events, 1);
    }
}
