use hecs::World;
use rand::{prelude::*, rngs::StdRng, SeedableRng};
use std::collections::HashSet;

mod components;
mod spatial;

pub use components::*;
pub use spatial::SpatialGrid;

pub const PREDATOR_TRAIT_THRESHOLD: f64 = 0.5;
const ATTACK_DRIVE_THRESHOLD: f64 = 0.55;

#[derive(Clone, Copy)]
struct NearbyResource {
    x: f64,
    y: f64,
    energy: f64,
    distance: f64,
}

#[derive(Clone, Copy)]
struct NearbyAgent {
    x: f64,
    y: f64,
    distance: f64,
    energy_current: f64,
    energy_max: f64,
    age: f64,
    last_reproduction: f64,
    reproduction_threshold: f64,
    size: f64,
    aggression: f64,
    is_predator: f64,
    hunting_speed: f64,
    attack_power: f64,
    defense: f64,
    stealth: f64,
    stamina: f64,
}

impl NearbyAgent {
    fn predator_drive(&self) -> f64 {
        self.is_predator.max(self.aggression * 0.75)
    }

    fn combat_strength(&self) -> f64 {
        self.attack_power * 1.2 + self.defense * 0.8 + self.size + self.stamina * 0.3
    }

    fn can_reproduce(&self) -> bool {
        EcsWorld::is_reproduction_candidate_from_values(
            self.energy_current,
            self.energy_max,
            self.age,
            self.last_reproduction,
            self.reproduction_threshold,
        )
    }
}

#[derive(Clone, Copy)]
struct AgentDecision {
    state: AgentStateEnum,
    target: Option<(f64, f64)>,
    speed_multiplier: f64,
}

#[derive(Clone, Copy)]
struct DecisionContext {
    sense_range: f64,
    consumption_radius: f64,
    fighting_radius: f64,
    can_reproduce: bool,
    own_energy_ratio: f64,
    own_predator_drive: f64,
    own_combat_strength: f64,
    own_aggression: f64,
    own_hunting_speed: f64,
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
            let predator_drive = predator_genes
                .is_predator
                .max(predator_genes.aggression * 0.75);
            if self.world.get::<&AgentTag>(predator_entity).is_err()
                || claimed_prey.contains(&predator_entity)
                || predator_drive < ATTACK_DRIVE_THRESHOLD
                || predator_energy.current <= 35.0
            {
                continue;
            }

            let predator_strength = predator_genes.attack_power * 1.2
                + predator_genes.defense * 0.8
                + predator_genes.size
                + predator_genes.stamina * 0.3;
            let hunting_radius =
                predator_genes.size * 6.0 + predator_genes.attack_power * 5.0 + 4.0;
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
                if prey_energy.current <= 0.0 {
                    continue;
                }

                let distance = ((predator_pos.x - prey_pos.x).powi(2)
                    + (predator_pos.y - prey_pos.y).powi(2))
                .sqrt();
                let prey_strength = prey_genes.attack_power * 1.2
                    + prey_genes.defense * 0.8
                    + prey_genes.size
                    + prey_genes.stamina * 0.3
                    + prey_genes.stealth * 0.35;
                let prey_drive = prey_genes.is_predator.max(prey_genes.aggression * 0.75);
                let matchup_advantage =
                    predator_strength + predator_drive - prey_strength - prey_drive * 0.35;

                if distance <= hunting_radius
                    && (prey_genes.is_predator < PREDATOR_TRAIT_THRESHOLD
                        || predator_drive > prey_drive + 0.15)
                    && matchup_advantage > 0.35
                {
                    to_kill.push((predator_entity, prey_entity));
                    claimed_prey.insert(prey_entity);
                    break;
                }
            }
        }

        let killed_prey: HashSet<_> = to_kill.iter().map(|(_, prey)| *prey).collect();
        for (predator_entity, prey_entity) in to_kill {
            if killed_prey.contains(&predator_entity) {
                continue;
            }

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
        let (
            sense_range,
            personal_space,
            pos_x,
            pos_y,
            own_energy_current,
            own_energy_max,
            own_age,
            own_last_reproduction,
            own_speed,
            own_size,
            own_reproduction_threshold,
            own_aggression,
            own_is_predator,
            own_hunting_speed,
            own_attack_power,
            own_defense,
            own_stealth,
            own_stamina,
        ) = {
            let Ok(genes) = self.world.get::<&Genes>(entity) else {
                return;
            };
            let Ok(pos) = self.world.get::<&Position>(entity) else {
                return;
            };
            let Ok(energy) = self.world.get::<&Energy>(entity) else {
                return;
            };
            let Ok(age) = self.world.get::<&Age>(entity) else {
                return;
            };
            let Ok(state) = self.world.get::<&AgentState>(entity) else {
                return;
            };
            (
                genes.sense_range,
                genes.personal_space,
                pos.x,
                pos.y,
                energy.current,
                energy.max,
                age.value,
                state.last_reproduction,
                genes.speed,
                genes.size,
                genes.reproduction_threshold,
                genes.aggression,
                genes.is_predator,
                genes.hunting_speed,
                genes.attack_power,
                genes.defense,
                genes.stealth,
                genes.stamina,
            )
        };

        let nearby_entities = self
            .spatial_grid
            .get_nearby_entities(pos_x, pos_y, sense_range);

        let mut nearby_resources = Vec::new();
        let mut nearby_agents = Vec::new();
        for nearby_entity in nearby_entities {
            if self.world.get::<&ResourceTag>(nearby_entity).is_ok() {
                if let Ok(resource_pos) = self.world.get::<&Position>(nearby_entity) {
                    if let Ok(resource) = self.world.get::<&Resource>(nearby_entity) {
                        if resource.is_available() {
                            let distance = ((pos_x - resource_pos.x).powi(2)
                                + (pos_y - resource_pos.y).powi(2))
                            .sqrt();
                            nearby_resources.push(NearbyResource {
                                x: resource_pos.x,
                                y: resource_pos.y,
                                energy: resource.energy,
                                distance,
                            });
                        }
                    }
                }
                continue;
            }

            if nearby_entity == entity || self.world.get::<&AgentTag>(nearby_entity).is_err() {
                continue;
            }

            let Ok(agent_pos) = self.world.get::<&Position>(nearby_entity) else {
                continue;
            };
            let Ok(agent_energy) = self.world.get::<&Energy>(nearby_entity) else {
                continue;
            };
            if agent_energy.current <= 0.0 {
                continue;
            }
            let Ok(agent_age) = self.world.get::<&Age>(nearby_entity) else {
                continue;
            };
            let Ok(agent_state) = self.world.get::<&AgentState>(nearby_entity) else {
                continue;
            };
            let Ok(agent_genes) = self.world.get::<&Genes>(nearby_entity) else {
                continue;
            };

            let distance = ((pos_x - agent_pos.x).powi(2) + (pos_y - agent_pos.y).powi(2)).sqrt();
            nearby_agents.push(NearbyAgent {
                x: agent_pos.x,
                y: agent_pos.y,
                distance,
                energy_current: agent_energy.current,
                energy_max: agent_energy.max,
                age: agent_age.value,
                last_reproduction: agent_state.last_reproduction,
                reproduction_threshold: agent_genes.reproduction_threshold,
                size: agent_genes.size,
                aggression: agent_genes.aggression,
                is_predator: agent_genes.is_predator,
                hunting_speed: agent_genes.hunting_speed,
                attack_power: agent_genes.attack_power,
                defense: agent_genes.defense,
                stealth: agent_genes.stealth,
                stamina: agent_genes.stamina,
            });
        }

        let own_energy_ratio = if own_energy_max > 0.0 {
            own_energy_current / own_energy_max
        } else {
            0.0
        };
        let own_predator_drive = own_is_predator.max(own_aggression * 0.75);
        let own_combat_strength =
            own_attack_power * 1.2 + own_defense * 0.8 + own_size + own_stamina * 0.3;
        let can_reproduce = Self::is_reproduction_candidate_from_values(
            own_energy_current,
            own_energy_max,
            own_age,
            own_last_reproduction,
            own_reproduction_threshold,
        );
        let consumption_radius = own_size * 5.0 + 4.0;
        let fighting_radius = own_size * 6.0 + own_attack_power * 5.0 + 4.0;
        let decision_context = DecisionContext {
            sense_range,
            consumption_radius,
            fighting_radius,
            can_reproduce,
            own_energy_ratio,
            own_predator_drive,
            own_combat_strength,
            own_aggression,
            own_hunting_speed,
        };

        let mut best_threat: Option<(NearbyAgent, f64)> = None;
        for candidate in &nearby_agents {
            let proximity = (1.0 - candidate.distance / sense_range).clamp(0.0, 1.0);
            let strength_gap = candidate.combat_strength() - own_combat_strength;
            let predator_pressure =
                candidate.predator_drive() - own_predator_drive * 0.6 - own_stealth * 0.15;
            let aggression_pressure = candidate.aggression - own_aggression * 0.6;
            let low_energy_risk = if own_energy_ratio < 0.45 { 0.5 } else { 0.0 };
            let score = strength_gap.max(0.0) * 0.9
                + predator_pressure.max(0.0) * 1.4
                + aggression_pressure.max(0.0) * 0.35
                + proximity * 0.8
                + low_energy_risk;

            let is_better = match best_threat {
                Some((_, best_score)) => score > best_score,
                None => true,
            };
            if is_better {
                best_threat = Some((*candidate, score));
            }
        }

        let decision = if let Some((threat, score)) = best_threat {
            let immediate_danger = threat.distance < personal_space * 1.8
                || threat.combat_strength() > own_combat_strength + 0.5;
            if score > 1.0 && (immediate_danger || own_energy_ratio < 0.75) {
                let dx = pos_x - threat.x;
                let dy = pos_y - threat.y;
                let distance = (dx * dx + dy * dy).sqrt().max(1.0);
                let escape_distance = sense_range.min(160.0) * (1.0 + (1.0 - own_energy_ratio));
                let escape_x = (pos_x + dx / distance * escape_distance).clamp(0.0, canvas_width);
                let escape_y = (pos_y + dy / distance * escape_distance).clamp(0.0, canvas_height);
                AgentDecision {
                    state: AgentStateEnum::Fleeing,
                    target: Some((escape_x, escape_y)),
                    speed_multiplier: 1.2 + own_stamina * 0.25,
                }
            } else {
                Self::choose_non_fleeing_decision(
                    &nearby_resources,
                    &nearby_agents,
                    decision_context,
                )
            }
        } else {
            Self::choose_non_fleeing_decision(&nearby_resources, &nearby_agents, decision_context)
        };

        let mut avoidance_dx = 0.0;
        let mut avoidance_dy = 0.0;
        for neighbor in &nearby_agents {
            let dx = pos_x - neighbor.x;
            let dy = pos_y - neighbor.y;
            let distance = neighbor.distance;
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
        let action_factor = match decision.state {
            AgentStateEnum::Fleeing | AgentStateEnum::Fighting => 1.25,
            AgentStateEnum::Hunting => 1.1,
            AgentStateEnum::Reproducing => 1.05,
            AgentStateEnum::Feeding | AgentStateEnum::Seeking => 1.0,
        };
        let total_energy_cost =
            base_energy_cost * metabolism_factor * environmental_factor * action_factor;
        energy.current -= total_energy_cost / genes.energy_efficiency;

        state.state = decision.state;
        if let Some((tx, ty)) = decision.target {
            state.target_x = Some(tx);
            state.target_y = Some(ty);

            let dx = tx - pos.x;
            let dy = ty - pos.y;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > 0.0 {
                vel.dx = (dx / distance) * genes.speed * decision.speed_multiplier;
                vel.dy = (dy / distance) * genes.speed * decision.speed_multiplier;
            }
        } else {
            state.target_x = None;
            state.target_y = None;

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
        let max_speed = own_speed * decision.speed_multiplier.max(1.0) * 1.35;
        if current_speed > max_speed {
            vel.dx = (vel.dx / current_speed) * max_speed;
            vel.dy = (vel.dy / current_speed) * max_speed;
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

    fn choose_non_fleeing_decision(
        nearby_resources: &[NearbyResource],
        nearby_agents: &[NearbyAgent],
        context: DecisionContext,
    ) -> AgentDecision {
        let can_attack = (context.own_predator_drive >= ATTACK_DRIVE_THRESHOLD
            && context.own_energy_ratio > 0.25)
            || (context.own_aggression > 0.72 && context.own_energy_ratio > 0.45);

        if can_attack {
            let mut best_prey: Option<(NearbyAgent, f64)> = None;
            for prey in nearby_agents {
                if prey.energy_current <= 0.0 {
                    continue;
                }

                let distance_factor = (1.0 - prey.distance / context.sense_range).clamp(0.0, 1.0);
                let energy_ratio = if prey.energy_max > 0.0 {
                    prey.energy_current / prey.energy_max
                } else {
                    0.0
                };
                let advantage = context.own_combat_strength + context.own_predator_drive
                    - prey.combat_strength()
                    - prey.stealth * 0.35;
                let weaker_prey_bonus = (1.0 - energy_ratio).clamp(0.0, 1.0) * 0.35;
                let prey_value = (prey.energy_current / 100.0).min(1.0) * 0.45;
                let predator_mismatch =
                    (context.own_predator_drive - prey.predator_drive()).max(-0.5);
                let score = advantage * 0.75
                    + predator_mismatch * 0.55
                    + (context.own_aggression - 0.45) * 0.45
                    + distance_factor * 0.7
                    + prey_value
                    + weaker_prey_bonus
                    - prey.hunting_speed * 0.05;

                let is_better = match best_prey {
                    Some((_, best_score)) => score > best_score,
                    None => true,
                };
                if is_better {
                    best_prey = Some((*prey, score));
                }
            }

            if let Some((prey, score)) = best_prey {
                if score > 0.55 {
                    return AgentDecision {
                        state: if prey.distance <= context.fighting_radius {
                            AgentStateEnum::Fighting
                        } else {
                            AgentStateEnum::Hunting
                        },
                        target: Some((prey.x, prey.y)),
                        speed_multiplier: context.own_hunting_speed.max(1.0),
                    };
                }
            }
        }

        if context.can_reproduce && context.own_energy_ratio > 0.68 {
            let mut best_mate: Option<(NearbyAgent, f64)> = None;
            for mate in nearby_agents {
                if !mate.can_reproduce() {
                    continue;
                }

                let distance_factor = (1.0 - mate.distance / context.sense_range).clamp(0.0, 1.0);
                let mate_energy_ratio = if mate.energy_max > 0.0 {
                    mate.energy_current / mate.energy_max
                } else {
                    0.0
                };
                let score = distance_factor * 0.7 + mate_energy_ratio * 0.3;
                let is_better = match best_mate {
                    Some((_, best_score)) => score > best_score,
                    None => true,
                };
                if is_better {
                    best_mate = Some((*mate, score));
                }
            }

            if let Some((mate, score)) = best_mate {
                if score > 0.25 {
                    return AgentDecision {
                        state: AgentStateEnum::Reproducing,
                        target: Some((mate.x, mate.y)),
                        speed_multiplier: 0.9,
                    };
                }
            }
        }

        let mut best_resource: Option<(NearbyResource, f64)> = None;
        for resource in nearby_resources {
            let hunger_bonus = (1.0 - context.own_energy_ratio).clamp(0.0, 1.0) * 0.7;
            let distance_factor = (1.0 - resource.distance / context.sense_range).clamp(0.0, 1.0);
            let score =
                resource.energy / (resource.distance + 8.0) + distance_factor * 0.4 + hunger_bonus;
            let is_better = match best_resource {
                Some((_, best_score)) => score > best_score,
                None => true,
            };
            if is_better {
                best_resource = Some((*resource, score));
            }
        }

        if let Some((resource, _)) = best_resource {
            return AgentDecision {
                state: if resource.distance <= context.consumption_radius * 1.5 {
                    AgentStateEnum::Feeding
                } else {
                    AgentStateEnum::Hunting
                },
                target: Some((resource.x, resource.y)),
                speed_multiplier: if resource.distance <= context.consumption_radius {
                    0.45
                } else {
                    1.0
                },
            };
        }

        AgentDecision {
            state: AgentStateEnum::Seeking,
            target: None,
            speed_multiplier: 1.0,
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
        Self::is_reproduction_candidate_from_values(
            energy.current,
            energy.max,
            age.value,
            state.last_reproduction,
            genes.reproduction_threshold,
        )
    }

    pub fn is_reproduction_candidate_from_values(
        energy_current: f64,
        energy_max: f64,
        age: f64,
        last_reproduction: f64,
        reproduction_threshold: f64,
    ) -> bool {
        energy_max > 0.0
            && energy_current >= reproduction_threshold
            && energy_current / energy_max > 0.6
            && age > 8.0
            && age - last_reproduction > 10.0
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

    fn baseline_genes() -> Genes {
        let mut genes = Genes::new();
        genes.speed = 2.0;
        genes.sense_range = 140.0;
        genes.size = 1.0;
        genes.energy_efficiency = 1.0;
        genes.reproduction_threshold = 60.0;
        genes.mutation_rate = 0.03;
        genes.aggression = 0.2;
        genes.is_predator = 0.0;
        genes.hunting_speed = 1.2;
        genes.attack_power = 0.8;
        genes.defense = 1.0;
        genes.stealth = 0.2;
        genes.pack_mentality = 0.2;
        genes.territory_size = 80.0;
        genes.metabolism = 1.0;
        genes.intelligence = 1.0;
        genes.stamina = 1.0;
        genes.personal_space = 10.0;
        genes
    }

    fn spawn_test_resource(world: &mut EcsWorld, x: f64, y: f64) {
        world.world.spawn((
            Position { x, y },
            Resource {
                energy: 80.0,
                max_energy: 100.0,
                size: 5.0,
                growth_rate: 0.3,
                regeneration_rate: 0.1,
                age: 0.0,
                target_energy: 100.0,
                is_spawning: false,
                spawn_fade: 1.0,
                is_depleting: false,
                deplete_fade: 0.0,
            },
            Size { value: 5.0 },
            ResourceTag,
        ));
    }

    fn set_energy_and_age(world: &mut EcsWorld, entity: hecs::Entity, energy: f64, age: f64) {
        world
            .world
            .get::<&mut Energy>(entity)
            .expect("agent energy")
            .current = energy;
        world
            .world
            .get::<&mut Age>(entity)
            .expect("agent age")
            .value = age;
    }

    #[test]
    fn agent_targets_nearby_resource_as_feeding() {
        let mut world = EcsWorld::new_with_population_and_seed(100.0, 100.0, 10, 10, 0, 0, Some(1));
        let agent = world.spawn_agent(50.0, 50.0, baseline_genes(), 0);
        spawn_test_resource(&mut world, 53.0, 50.0);

        world.update_spatial_grid();
        world.update_single_agent_optimized(agent, 1.0 / 60.0, 100.0, 100.0);

        let state = world.world.get::<&AgentState>(agent).expect("agent state");
        assert_eq!(state.state, AgentStateEnum::Feeding);
        assert!(state.target_x.is_some());
    }

    #[test]
    fn agent_flees_from_stronger_nearby_threat() {
        let mut world = EcsWorld::new_with_population_and_seed(100.0, 100.0, 10, 10, 0, 0, Some(2));

        let mut prey_genes = baseline_genes();
        prey_genes.defense = 0.4;
        prey_genes.stealth = 0.0;
        let prey = world.spawn_agent(50.0, 50.0, prey_genes, 0);
        set_energy_and_age(&mut world, prey, 35.0, 20.0);

        let mut threat_genes = baseline_genes();
        threat_genes.is_predator = 1.0;
        threat_genes.aggression = 0.95;
        threat_genes.attack_power = 2.4;
        threat_genes.defense = 1.8;
        threat_genes.size = 1.6;
        world.spawn_agent(56.0, 50.0, threat_genes, 0);

        world.update_spatial_grid();
        world.update_single_agent_optimized(prey, 1.0 / 60.0, 100.0, 100.0);

        let state = world.world.get::<&AgentState>(prey).expect("agent state");
        assert_eq!(state.state, AgentStateEnum::Fleeing);
        assert!(state.target_x.is_some());
    }

    #[test]
    fn predator_targets_weak_prey_as_fighting() {
        let mut world = EcsWorld::new_with_population_and_seed(100.0, 100.0, 10, 10, 0, 0, Some(3));

        let mut predator_genes = baseline_genes();
        predator_genes.is_predator = 1.0;
        predator_genes.aggression = 0.9;
        predator_genes.attack_power = 2.2;
        predator_genes.hunting_speed = 1.8;
        let predator = world.spawn_agent(50.0, 50.0, predator_genes, 0);
        set_energy_and_age(&mut world, predator, 85.0, 20.0);

        let mut prey_genes = baseline_genes();
        prey_genes.defense = 0.4;
        prey_genes.stealth = 0.0;
        world.spawn_agent(56.0, 50.0, prey_genes, 0);

        world.update_spatial_grid();
        world.update_single_agent_optimized(predator, 1.0 / 60.0, 100.0, 100.0);

        let state = world
            .world
            .get::<&AgentState>(predator)
            .expect("agent state");
        assert_eq!(state.state, AgentStateEnum::Fighting);
        assert!(state.target_x.is_some());
    }

    #[test]
    fn eligible_agent_seeks_nearby_mate() {
        let mut world = EcsWorld::new_with_population_and_seed(100.0, 100.0, 10, 10, 0, 0, Some(4));
        let first = world.spawn_agent(50.0, 50.0, baseline_genes(), 0);
        let second = world.spawn_agent(65.0, 50.0, baseline_genes(), 0);
        set_energy_and_age(&mut world, first, 95.0, 30.0);
        set_energy_and_age(&mut world, second, 95.0, 30.0);

        world.update_spatial_grid();
        world.update_single_agent_optimized(first, 1.0 / 60.0, 100.0, 100.0);

        let state = world.world.get::<&AgentState>(first).expect("agent state");
        assert_eq!(state.state, AgentStateEnum::Reproducing);
        assert!(state.target_x.is_some());
    }

    #[test]
    fn predator_kill_records_events_and_removes_prey() {
        let mut world = EcsWorld::new_with_population(100.0, 100.0, 10, 10, 0, 0);

        let mut predator_genes = baseline_genes();
        predator_genes.is_predator = 1.0;
        predator_genes.hunting_speed = 10.0;
        predator_genes.aggression = 0.9;
        predator_genes.attack_power = 2.3;
        predator_genes.defense = 1.6;

        let mut prey_genes = baseline_genes();
        prey_genes.is_predator = 0.0;
        prey_genes.defense = 0.4;
        prey_genes.stealth = 0.0;

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
