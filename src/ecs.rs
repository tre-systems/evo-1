use hecs::World;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// COMPONENTS
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Energy {
    pub current: f64,
    pub max: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Age {
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genes {
    pub speed: f64,
    pub sense_range: f64,
    pub size: f64,
    pub energy_efficiency: f64,
    pub reproduction_threshold: f64,
    pub mutation_rate: f64,
    pub aggression: f64,
    pub color_hue: f64,
    pub is_predator: f64,
    pub hunting_speed: f64,
    pub attack_power: f64,
    pub defense: f64,
    pub stealth: f64,
    pub pack_mentality: f64,
    pub territory_size: f64,
    pub metabolism: f64,
    pub intelligence: f64,
    pub stamina: f64,
    pub personal_space: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentState {
    pub state: AgentStateEnum,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub last_reproduction: f64,
    pub kills: u32,
    pub generation: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AgentStateEnum {
    Seeking,
    Hunting,
    Feeding,
    Reproducing,
    Fighting,
    Fleeing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeathAnimation {
    pub fade: f64,
    pub reason: DeathReason,
    pub is_dying: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeathReason {
    Starvation,
    OldAge,
    KilledByPredator,
    Combat,
    NaturalCauses,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpawnAnimation {
    pub fade: f64,
    pub spawn_position: Option<(f64, f64)>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
    pub energy: f64,
    pub max_energy: f64,
    pub size: f64,
    pub growth_rate: f64,
    pub regeneration_rate: f64,
    pub age: f64,
    pub target_energy: f64,
    pub is_spawning: bool,
    pub spawn_fade: f64,
    pub is_depleting: bool,
    pub deplete_fade: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Size {
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTag;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceTag;

// ============================================================================
// SPATIAL PARTITIONING
// ============================================================================

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
        let cells = vec![vec![Vec::new(); height]; width];

        Self {
            cell_size,
            width,
            height,
            cells,
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            for cell in row {
                cell.clear();
            }
        }
    }

    pub fn get_cell(&self, x: f64, y: f64) -> (usize, usize) {
        let grid_x = (x / self.cell_size).floor() as usize;
        let grid_y = (y / self.cell_size).floor() as usize;
        (grid_x.min(self.width - 1), grid_y.min(self.height - 1))
    }

    pub fn add_entity(&mut self, entity: hecs::Entity, x: f64, y: f64) {
        let (grid_x, grid_y) = self.get_cell(x, y);
        if grid_x < self.width && grid_y < self.height {
            self.cells[grid_x][grid_y].push(entity);
        }
    }

    pub fn get_nearby_entities(&self, x: f64, y: f64, radius: f64) -> Vec<hecs::Entity> {
        let mut entities = Vec::new();
        let (center_x, center_y) = self.get_cell(x, y);
        let cell_radius = (radius / self.cell_size).ceil() as i32;

        for dx in -cell_radius..=cell_radius {
            for dy in -cell_radius..=cell_radius {
                let check_x = center_x as i32 + dx;
                let check_y = center_y as i32 + dy;

                if check_x >= 0
                    && check_x < self.width as i32
                    && check_y >= 0
                    && check_y < self.height as i32
                {
                    let cell_entities = &self.cells[check_x as usize][check_y as usize];
                    entities.extend(cell_entities.iter().cloned());
                }
            }
        }

        entities
    }
}

// ============================================================================
// WORLD MANAGEMENT
// ============================================================================

pub struct EcsWorld {
    pub world: World,
    pub canvas_width: f64,
    pub canvas_height: f64,
    pub max_agents: usize,
    pub max_resources: usize,
    pub resource_spawn_timer: f64,
    pub spatial_grid: SpatialGrid,
    // Debug tracking
    pub consumption_events_this_frame: usize,
    pub total_consumption_events: usize,
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
        let world = World::new();
        let spatial_grid = SpatialGrid::new(canvas_width, canvas_height, 50.0);

        let mut ecs_world = Self {
            world,
            canvas_width,
            canvas_height,
            max_agents,
            max_resources,
            resource_spawn_timer: 0.0,
            spatial_grid,
            // Debug tracking
            consumption_events_this_frame: 0,
            total_consumption_events: 0,
        };

        // Spawn initial population
        ecs_world.spawn_initial_population(
            initial_agents.min(max_agents),
            initial_resources.min(max_resources),
        );

        ecs_world
    }

    pub fn update(&mut self) {
        let delta_time = 10.0 / 60.0; // 10x faster simulation

        // Update resources
        self.update_resources(delta_time);

        // Handle consumption (agents eating resources and predators eating prey)
        self.handle_consumption();

        // Update spatial grid
        self.update_spatial_grid();

        // Update agents
        self.update_agents(delta_time);

        // Handle death
        self.handle_death();

        // Handle reproduction
        self.handle_reproduction();

        // Disable automatic resource spawning - let resources be finite
        // self.resource_spawn_timer += delta_time;
        // if self.resource_spawn_timer > 0.05 && self.get_resource_count() < self.max_resources {
        //     self.spawn_resource();
        //     self.resource_spawn_timer = 0.0;
        // }

        // Handle resource depletion
        self.handle_resource_depletion();
    }

    pub fn handle_resource_depletion(&mut self) {
        // Find resources that are being consumed and deplete them
        let mut to_deplete = Vec::new();

        for (entity, resource) in self.world.query::<&Resource>().iter() {
            if self.world.get::<&ResourceTag>(entity).is_ok() && resource.energy <= 0.0 {
                to_deplete.push(entity);
            }
        }

        // Remove depleted resources
        for entity in to_deplete {
            self.world.despawn(entity).ok();
        }
    }

    pub fn handle_consumption(&mut self) {
        // Reset frame counter
        self.consumption_events_this_frame = 0;

        // Track which resources are being consumed by agents
        let mut consumed_resources = Vec::new();

        // Check for agents consuming resources
        for (agent_entity, (agent_pos, agent_genes)) in
            self.world.query::<(&Position, &Genes)>().iter()
        {
            if self.world.get::<&AgentTag>(agent_entity).is_ok() {
                // Look for nearby resources this agent might consume
                for (resource_entity, (resource_pos, resource)) in
                    self.world.query::<(&Position, &Resource)>().iter()
                {
                    if self.world.get::<&ResourceTag>(resource_entity).is_ok()
                        && resource.is_available()
                    {
                        let distance = ((agent_pos.x - resource_pos.x).powi(2)
                            + (agent_pos.y - resource_pos.y).powi(2))
                        .sqrt();

                        // Agent can consume resource if close enough
                        if distance <= agent_genes.size * 2.0 {
                            consumed_resources.push((
                                agent_entity,
                                resource_entity,
                                agent_genes.energy_efficiency,
                            ));
                        }
                    }
                }
            }
        }

        // Actually consume the resources and give energy to agents
        for (agent_entity, resource_entity, energy_efficiency) in consumed_resources {
            if let Ok(mut resource) = self.world.get::<&mut Resource>(resource_entity) {
                let consumption_amount = (resource.energy * 0.8).min(100.0); // Consume 80% of resource energy, max 100
                let actual_consumed = resource.consume(consumption_amount);

                // Track consumption events
                self.consumption_events_this_frame += 1;
                self.total_consumption_events += 1;

                // Give energy to the agent that consumed the resource
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

        // Simple predator-prey consumption - predators consume nearby prey
        let mut to_kill = Vec::new();

        for (predator_entity, (predator_pos, predator_genes, predator_energy)) in
            self.world.query::<(&Position, &Genes, &Energy)>().iter()
        {
            if self.world.get::<&AgentTag>(predator_entity).is_ok()
                && predator_genes.is_predator > 0.5
                && predator_energy.current > 50.0
            {
                for (prey_entity, (prey_pos, prey_genes, prey_energy)) in
                    self.world.query::<(&Position, &Genes, &Energy)>().iter()
                {
                    if prey_entity != predator_entity
                        && self.world.get::<&AgentTag>(prey_entity).is_ok()
                        && prey_genes.is_predator < 0.5
                        && prey_energy.current > 0.0
                    {
                        let distance = ((predator_pos.x - prey_pos.x).powi(2)
                            + (predator_pos.y - prey_pos.y).powi(2))
                        .sqrt();
                        let hunting_radius = predator_genes.hunting_speed * 3.0;

                        if distance <= hunting_radius {
                            to_kill.push(prey_entity);
                            break;
                        }
                    }
                }
            }
        }

        // Mark prey for death
        for prey_entity in to_kill {
            if let Ok(mut death_anim) = self.world.get::<&mut DeathAnimation>(prey_entity) {
                death_anim.is_dying = true;
                death_anim.reason = DeathReason::KilledByPredator;
            }
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

        // First, collect all resources for efficient lookup
        let resources: Vec<_> = self
            .world
            .query::<(&Position, &Resource)>()
            .iter()
            .filter(|(entity, _)| self.world.get::<&ResourceTag>(*entity).is_ok())
            .map(|(_, (pos, res))| (pos.x, pos.y, res.clone()))
            .collect();

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
            self.update_single_agent(entity, delta_time, &resources, canvas_width, canvas_height);
        }
    }

    pub fn update_single_agent(
        &mut self,
        entity: hecs::Entity,
        delta_time: f64,
        _resources: &[(f64, f64, Resource)],
        canvas_width: f64,
        canvas_height: f64,
    ) {
        // Legacy method - kept for compatibility
        self.update_single_agent_optimized(entity, delta_time, canvas_width, canvas_height);
    }

    pub fn update_single_agent_optimized(
        &mut self,
        entity: hecs::Entity,
        delta_time: f64,
        canvas_width: f64,
        canvas_height: f64,
    ) {
        // Get the agent's genes and position first to know the sense range
        let (sense_range, pos_x, pos_y) = {
            let genes = self.world.get::<&Genes>(entity).unwrap();
            let pos = self.world.get::<&Position>(entity).unwrap();
            (genes.sense_range, pos.x, pos.y)
        };

        // Collect nearby resource data first to avoid borrowing conflicts
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

        // Now update the agent with the collected data
        let (pos, vel, energy, age, state, genes) = self
            .world
            .query_one_mut::<(
                &mut Position,
                &mut Velocity,
                &mut Energy,
                &mut Age,
                &mut AgentState,
                &Genes,
            )>(entity)
            .unwrap();

        // Update age
        age.value += delta_time;

        // Energy consumption - much higher to make starvation a real threat
        let base_energy_cost = (genes.size * 0.2 + genes.speed * 0.1) * delta_time; // Increased from 0.05/0.02
        let metabolism_factor = genes.metabolism;
        let environmental_factor = 1.0 + (pos.x / canvas_width + pos.y / canvas_height) * 0.001;
        let total_energy_cost = base_energy_cost * metabolism_factor * environmental_factor;
        energy.current -= total_energy_cost / genes.energy_efficiency;

        // Find best resource and handle consumption
        let mut best_target = None;
        let mut best_score = f64::NEG_INFINITY;
        let consumed_resource = false;

        for (rx, ry, resource_energy) in &nearby_resources {
            let distance = ((pos.x - rx).powi(2) + (pos.y - ry).powi(2)).sqrt();
            if distance <= genes.sense_range {
                let score = resource_energy / (distance + 1.0);
                if score > best_score {
                    best_score = score;
                    best_target = Some((*rx, *ry));
                }

                // Note: Resource consumption is handled in the main consumption loop
                // This is just for targeting and movement
            }
        }

        // Update behavior
        if let Some((tx, ty)) = best_target {
            state.target_x = Some(tx);
            state.target_y = Some(ty);

            if !consumed_resource {
                state.state = AgentStateEnum::Hunting;

                // Move towards target
                let dx = tx - pos.x;
                let dy = ty - pos.y;
                let distance = (dx * dx + dy * dy).sqrt();
                if distance > 0.0 {
                    vel.dx = (dx / distance) * genes.speed;
                    vel.dy = (dy / distance) * genes.speed;
                }
            }
        } else {
            // Random movement - make it more active
            let mut rng = thread_rng();

            // Change direction more frequently for more movement
            if rng.gen::<f64>() < 0.1 {
                // 10% chance to change direction each frame
                let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
                vel.dx = angle.cos() * genes.speed;
                vel.dy = angle.sin() * genes.speed;
            }

            // Ensure minimum movement speed
            let current_speed = (vel.dx * vel.dx + vel.dy * vel.dy).sqrt();
            if current_speed < genes.speed * 0.5 {
                let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
                vel.dx = angle.cos() * genes.speed;
                vel.dy = angle.sin() * genes.speed;
            }
        }

        // Calculate agent avoidance forces (simplified to avoid borrowing issues)
        let mut avoidance_dx = 0.0;
        let mut avoidance_dy = 0.0;

        // Simple avoidance based on current position and genes
        // We'll use a simplified approach that doesn't require additional world queries
        let _avoidance_radius = genes.personal_space * 0.5; // Smaller radius for performance

        // Apply a small random avoidance force to simulate personal space
        let mut rng = thread_rng();
        if rng.gen::<f64>() < 0.3 {
            // 30% chance to apply avoidance
            let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            avoidance_dx = angle.cos() * genes.speed * 0.3;
            avoidance_dy = angle.sin() * genes.speed * 0.3;
        }

        // Apply avoidance forces to velocity
        vel.dx += avoidance_dx;
        vel.dy += avoidance_dy;

        // Limit maximum speed
        let current_speed = (vel.dx * vel.dx + vel.dy * vel.dy).sqrt();
        if current_speed > genes.speed * 1.5 {
            vel.dx = (vel.dx / current_speed) * genes.speed * 1.5;
            vel.dy = (vel.dy / current_speed) * genes.speed * 1.5;
        }

        // Move agent
        pos.x += vel.dx * delta_time;
        pos.y += vel.dy * delta_time;

        // Boundary wrapping
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
        // Use a more efficient approach - collect entities to despawn
        let mut to_despawn = Vec::new();

        for (entity, (energy, age)) in self.world.query::<(&Energy, &Age)>().iter() {
            if energy.current <= 0.0 || age.value > 200.0 {
                to_despawn.push(entity);
            }
        }

        // Despawn dead entities
        for entity in to_despawn {
            self.world.despawn(entity).ok();
        }
    }

    pub fn handle_reproduction(&mut self) {
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
            .filter(|(_, (_, energy, age, state, _))| self.can_reproduce(energy, age, state))
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
        let mut rng = thread_rng();
        potential_parents.shuffle(&mut rng);

        // Try to pair agents for reproduction
        let mut i = 0;
        while i < potential_parents.len() - 1 && self.get_agent_count() < self.max_agents {
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
                let reproduction_chance = energy_factor * age_factor * 0.3; // Increased to 30% base chance

                if rng.gen::<f64>() < reproduction_chance {
                    // Create offspring with inherited genes
                    let offspring_genes = self.inherit_genes(genes1, genes2);

                    // Add some mutation
                    let mutated_genes = self.mutate_genes(&offspring_genes);

                    // Spawn offspring between parents
                    let offspring_x = (pos1.x + pos2.x) / 2.0;
                    let offspring_y = (pos1.y + pos2.y) / 2.0;
                    let offspring_generation = state1.generation.max(state2.generation) + 1;

                    self.spawn_agent(
                        offspring_x,
                        offspring_y,
                        mutated_genes,
                        offspring_generation,
                    );

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
                        energy1_mut.current = (energy1_mut.current - 20.0).max(10.0);
                    }
                    if let Ok(mut energy2_mut) = self.world.get::<&mut crate::ecs::Energy>(*entity2)
                    {
                        energy2_mut.current = (energy2_mut.current - 20.0).max(10.0);
                    }
                }
            }
            i += 2; // Skip both parents
        }
    }

    fn can_reproduce(&self, energy: &Energy, age: &Age, state: &AgentState) -> bool {
        energy.current > 20.0 && age.value > 1.0 && age.value - state.last_reproduction > 0.5
    }

    fn distance(&self, pos1: &Position, pos2: &Position) -> f64 {
        ((pos1.x - pos2.x).powi(2) + (pos1.y - pos2.y).powi(2)).sqrt()
    }

    fn inherit_genes(&self, genes1: &Genes, genes2: &Genes) -> Genes {
        let mut rng = thread_rng();
        let blend_factor = rng.gen_range(0.3..0.7);

        Genes {
            speed: genes1.speed * blend_factor + genes2.speed * (1.0 - blend_factor),
            sense_range: genes1.sense_range * blend_factor
                + genes2.sense_range * (1.0 - blend_factor),
            size: genes1.size * blend_factor + genes2.size * (1.0 - blend_factor),
            energy_efficiency: genes1.energy_efficiency * blend_factor
                + genes2.energy_efficiency * (1.0 - blend_factor),
            reproduction_threshold: genes1.reproduction_threshold * blend_factor
                + genes2.reproduction_threshold * (1.0 - blend_factor),
            mutation_rate: genes1.mutation_rate * blend_factor
                + genes2.mutation_rate * (1.0 - blend_factor),
            aggression: genes1.aggression * blend_factor + genes2.aggression * (1.0 - blend_factor),
            color_hue: genes1.color_hue * blend_factor + genes2.color_hue * (1.0 - blend_factor),
            is_predator: genes1.is_predator * blend_factor
                + genes2.is_predator * (1.0 - blend_factor),
            hunting_speed: genes1.hunting_speed * blend_factor
                + genes2.hunting_speed * (1.0 - blend_factor),
            attack_power: genes1.attack_power * blend_factor
                + genes2.attack_power * (1.0 - blend_factor),
            defense: genes1.defense * blend_factor + genes2.defense * (1.0 - blend_factor),
            stealth: genes1.stealth * blend_factor + genes2.stealth * (1.0 - blend_factor),
            pack_mentality: genes1.pack_mentality * blend_factor
                + genes2.pack_mentality * (1.0 - blend_factor),
            territory_size: genes1.territory_size * blend_factor
                + genes2.territory_size * (1.0 - blend_factor),
            metabolism: genes1.metabolism * blend_factor + genes2.metabolism * (1.0 - blend_factor),
            intelligence: genes1.intelligence * blend_factor
                + genes2.intelligence * (1.0 - blend_factor),
            stamina: genes1.stamina * blend_factor + genes2.stamina * (1.0 - blend_factor),
            personal_space: genes1.personal_space * blend_factor
                + genes2.personal_space * (1.0 - blend_factor),
        }
    }

    fn spawn_agent(&mut self, x: f64, y: f64, genes: Genes, generation: u32) {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        let size_value = genes.size * 3.0;

        self.world.spawn((
            Position { x, y },
            Velocity {
                dx: angle.cos() * genes.speed,
                dy: angle.sin() * genes.speed,
            },
            Energy {
                current: 30.0, // Reduced from 80.0 to make starvation more likely
                max: 100.0,
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
        ));
    }

    pub fn spawn_resource(&mut self) {
        let mut rng = thread_rng();
        let x = rng.gen_range(0.0..self.canvas_width);
        let y = rng.gen_range(0.0..self.canvas_height);

        let initial_energy = rng.gen_range(15.0..40.0);
        let max_energy = rng.gen_range(30.0..60.0);

        self.world.spawn((
            Position { x, y },
            Resource {
                energy: initial_energy, // Start with initial energy so resources are immediately available
                max_energy,
                size: 3.0,
                growth_rate: rng.gen_range(0.1..0.5),
                regeneration_rate: rng.gen_range(0.02..0.1),
                age: 0.0,
                target_energy: initial_energy,
                is_spawning: false, // Start fully spawned
                spawn_fade: 1.0,    // Start fully spawned and immediately available
                is_depleting: false,
                deplete_fade: 0.0,
            },
            Size { value: 3.0 },
            ResourceTag,
        ));
    }

    fn spawn_initial_population(&mut self, initial_agents: usize, initial_resources: usize) {
        let mut rng = thread_rng();

        // Spawn initial agents
        for _ in 0..initial_agents {
            let x = rng.gen_range(0.0..self.canvas_width);
            let y = rng.gen_range(0.0..self.canvas_height);
            let genes = self.generate_random_genes();
            self.spawn_agent(x, y, genes, 0);
        }

        // Spawn initial resources
        for _ in 0..initial_resources {
            self.spawn_resource();
        }
    }

    fn generate_random_genes(&self) -> Genes {
        let mut rng = thread_rng();

        Genes {
            speed: rng.gen_range(0.8..1.5),
            sense_range: rng.gen_range(30.0..80.0),
            size: rng.gen_range(0.9..1.3),
            energy_efficiency: rng.gen_range(0.8..1.2),
            reproduction_threshold: rng.gen_range(60.0..120.0),
            mutation_rate: rng.gen_range(0.02..0.08),
            aggression: rng.gen_range(0.2..0.8),
            color_hue: rng.gen_range(0.0..360.0),
            is_predator: rng.gen_range(0.0..0.3),
            hunting_speed: rng.gen_range(1.0..2.0),
            attack_power: rng.gen_range(0.5..1.5),
            defense: rng.gen_range(0.5..1.5),
            stealth: rng.gen_range(0.0..1.0),
            pack_mentality: rng.gen_range(0.0..1.0),
            territory_size: rng.gen_range(50.0..150.0),
            metabolism: rng.gen_range(0.8..1.4),
            intelligence: rng.gen_range(0.5..1.5),
            stamina: rng.gen_range(0.5..1.5),
            personal_space: rng.gen_range(20.0..60.0),
        }
    }

    fn mutate_genes(&self, genes: &Genes) -> Genes {
        let mut rng = thread_rng();
        let mutation_strength = 0.1; // 10% mutation strength

        Genes {
            speed: genes.speed * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            sense_range: genes.sense_range * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            size: genes.size * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            energy_efficiency: genes.energy_efficiency
                * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            reproduction_threshold: genes.reproduction_threshold
                * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            mutation_rate: genes.mutation_rate
                * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            aggression: genes.aggression * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            color_hue: genes.color_hue * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            is_predator: genes.is_predator * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            hunting_speed: genes.hunting_speed
                * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            attack_power: genes.attack_power * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            defense: genes.defense * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            stealth: genes.stealth * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            pack_mentality: genes.pack_mentality
                * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            territory_size: genes.territory_size
                * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            metabolism: genes.metabolism * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            intelligence: genes.intelligence * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            stamina: genes.stamina * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
            personal_space: genes.personal_space
                * (1.0 + (rng.gen::<f64>() - 0.5) * mutation_strength),
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
        self.world = World::new();
        self.resource_spawn_timer = 0.0;
        self.spatial_grid.clear();
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
            .map(|(_, (pos, resource, size))| (pos.clone(), resource.clone(), size.clone()))
            .collect()
    }
}

// Extension trait for Resource to add the update method
impl Resource {
    pub fn update(&mut self, delta_time: f64) {
        self.age += delta_time;

        // Handle spawning fade-in
        if self.is_spawning {
            self.spawn_fade += delta_time * 2.0;
            if self.spawn_fade >= 1.0 {
                self.spawn_fade = 1.0;
                self.is_spawning = false;
            }
        }

        // Handle depletion fade-out
        if self.is_depleting {
            self.deplete_fade += delta_time * 3.0;
            if self.deplete_fade >= 1.0 {
                self.deplete_fade = 1.0;
            }
        }

        // Disable resource growth and regeneration - resources should only deplete
        // if !self.is_spawning && !self.is_depleting {
        //     let energy_diff = self.target_energy - self.energy;
        //     if energy_diff.abs() > 0.1 {
        //         let growth_direction = if energy_diff > 0.0 { 1.0 } else { -1.0 };
        //         let growth_amount = self.growth_rate * delta_time * growth_direction;

        //         if energy_diff.abs() < growth_amount.abs() {
        //             self.energy = self.target_energy;
        //         } else {
        //             self.energy += growth_amount;
        //         }
        //     }

        //     // Natural growth towards max energy
        //     if self.energy < self.max_energy {
        //         self.energy += self.growth_rate * delta_time * 0.05;
        //         if self.energy > self.max_energy {
        //             self.energy = self.max_energy;
        //         }
        //     }

        //     if self.energy >= self.max_energy {
        //         self.target_energy = self.max_energy;
        //     }
        // }

        // Size changes based on energy (keep this for visual feedback)
        let target_size = 3.0 + (self.energy / self.max_energy) * 5.0;
        let size_diff = target_size - self.size;
        if size_diff.abs() > 0.1 {
            self.size += size_diff * delta_time * 2.0;
        }

        // Disable regeneration - resources should not recover
        // if self.energy < 10.0 && !self.is_depleting {
        //     self.energy += self.regeneration_rate * delta_time * 0.2;
        // }
    }

    pub fn is_available(&self) -> bool {
        self.energy > 5.0 && !self.is_depleting && self.spawn_fade > 0.5
    }

    pub fn consume(&mut self, amount: f64) -> f64 {
        let consumed = amount.min(self.energy);
        self.energy -= consumed;

        // If completely depleted, start depletion fade
        if self.energy <= 0.0 {
            self.is_depleting = true;
            self.deplete_fade = 0.0;
            self.energy = 0.0;
        }

        consumed
    }
}
