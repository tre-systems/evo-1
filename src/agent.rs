use crate::genes::Genes;
use crate::resource::Resource;
use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    pub x: f64,
    pub y: f64,
    pub dx: f64,
    pub dy: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub age: f64,
    pub genes: Genes,
    pub target_x: Option<f64>,
    pub target_y: Option<f64>,
    pub state: AgentState,
    pub last_reproduction: f64,
    pub kills: u32,
    pub generation: u32,
    pub death_fade: f64, // Fade out timer when dying (0.0 = alive, 1.0 = fully faded)
    pub death_reason: Option<DeathReason>, // Why the agent died
    pub is_dying: bool,  // Whether the agent is in death animation
    pub spawn_fade: f64, // Fade in timer for new agents (0.0 = invisible, 1.0 = fully visible)
    pub spawn_position: Option<(f64, f64)>, // Position where agent was spawned
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    Seeking,
    Hunting,
    Feeding,
    Reproducing,
    Fighting,
    Fleeing,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DeathReason {
    Starvation,
    OldAge,
    KilledByPredator,
    Combat,
    NaturalCauses,
}

impl Agent {
    pub fn new(x: f64, y: f64, genes: Genes, generation: u32) -> Self {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);

        Self {
            x,
            y,
            dx: angle.cos() * genes.speed,
            dy: angle.sin() * genes.speed,
            energy: 80.0, // Increased from 50.0 - better starting energy
            max_energy: 100.0,
            age: 0.0,
            genes,
            target_x: None,
            target_y: None,
            state: AgentState::Seeking,
            last_reproduction: 0.0,
            kills: 0,
            generation,
            death_fade: 0.0,
            death_reason: None,
            is_dying: false,
            spawn_fade: 0.0, // Start invisible and fade in
            spawn_position: Some((x, y)),
        }
    }

    pub fn update(
        &mut self,
        delta_time: f64,
        resources: &[Resource],
        agents: &[Agent],
        canvas_width: f64,
        canvas_height: f64,
    ) -> Option<usize> {
        self.age += delta_time;

        // Handle spawn fade-in for new agents
        if self.spawn_fade < 1.0 {
            self.spawn_fade += delta_time * 3.0; // Fade in over 0.33 seconds
            if self.spawn_fade >= 1.0 {
                self.spawn_fade = 1.0;
            }
        }

        // Handle death fade-out
        if self.is_dying {
            self.death_fade += delta_time * 2.0; // Fade out over 0.5 seconds
            if self.death_fade >= 1.0 {
                return None; // Agent is fully dead
            }
            // Don't update behavior when dying, just fade out
            return None;
        }

        // Much higher energy consumption - agents should die quickly without food
        let base_energy_cost = (self.genes.size * 0.05 + self.genes.speed * 0.02) * delta_time;
        let metabolism_factor = self.genes.metabolism;
        let environmental_factor = 1.0 + (self.x / canvas_width + self.y / canvas_height) * 0.001;
        let total_energy_cost = base_energy_cost * metabolism_factor * environmental_factor;
        self.energy -= total_energy_cost / self.genes.energy_efficiency;

        // Check for death and start death animation
        if self.energy <= 0.0 {
            self.is_dying = true;
            self.death_reason = Some(DeathReason::Starvation);
            self.death_fade = 0.0;
            return None;
        }

        if self.age > 200.0 {
            self.is_dying = true;
            self.death_reason = Some(DeathReason::OldAge);
            self.death_fade = 0.0;
            return None;
        }

        // Complex behavioral decision making
        self.update_behavior_state(resources, agents);

        let mut consumed_resource = None;

        // Update behavior based on current state
        match self.state {
            AgentState::Seeking => self.seek_targets(resources, agents),
            AgentState::Hunting => self.hunt_target(delta_time),
            AgentState::Feeding => consumed_resource = self.feed_on_resource(resources),
            AgentState::Reproducing => self.reproduce(),
            AgentState::Fighting => self.fight_agent(agents),
            AgentState::Fleeing => self.flee_from_danger(delta_time),
        }

        // Move agent with complex physics
        self.move_agent(delta_time, canvas_width, canvas_height);

        // Check for reproduction with more complex conditions
        if self.can_reproduce() {
            self.state = AgentState::Reproducing;
        }

        // Reduced learning calculations - only run occasionally
        if self.age % 1.0 < delta_time {
            // Only run every 1 second instead of every 10 seconds (10x faster)
            self.perform_learning_calculations(delta_time);
        }

        consumed_resource
    }

    fn update_behavior_state(&mut self, resources: &[Resource], agents: &[Agent]) {
        // Complex decision making based on environment
        let mut threat_level = 0.0;
        let mut resource_abundance = 0.0;
        let mut _population_density = 0.0;

        // Calculate environmental factors
        for agent in agents {
            if agent.id() != self.id() {
                let distance = self.distance_to(agent.x, agent.y);
                if distance < 100.0 {
                    _population_density += 1.0 / (distance + 1.0);
                    if agent.genes.size > self.genes.size * 1.2 {
                        threat_level += 1.0 / (distance + 1.0);
                    }
                }
            }
        }

        for resource in resources {
            let distance = resource.distance_to(self.x, self.y);
            if distance < 200.0 {
                resource_abundance += resource.energy / (distance + 1.0);
            }
        }

        // Complex behavioral adaptation - REMOVED SPEED MULTIPLICATION
        // This was causing exponential speed growth
        let _stress_factor = threat_level * 0.5 + (1.0 - resource_abundance / 1000.0) * 0.3;
        // self.genes.speed *= (1.0 + stress_factor * 0.1).min(2.0); // REMOVED THIS LINE
    }

    fn perform_learning_calculations(&mut self, _delta_time: f64) {
        // Simplified learning calculations - much less expensive
        let input = self.energy / self.max_energy;
        let weight1 = self.genes.speed;
        let weight2 = self.genes.size;
        let weight3 = self.genes.aggression;

        let hidden1 = (input * weight1 + self.age * weight2).tanh();
        let hidden2 = (hidden1 * weight3 + self.energy * weight1).tanh();
        let output = 1.0 / (1.0 + (-hidden2 * weight2 - input * weight3).exp()); // sigmoid function

        // Apply learning to genes (subtle adaptation) - only if output is high
        // REMOVED GENE MODIFICATIONS - these were causing instability
        if output > 0.7 {
            // self.genes.speed *= 1.0001; // REMOVED
            // self.genes.size *= 1.0001; // REMOVED
        }
    }

    fn seek_targets(&mut self, resources: &[Resource], agents: &[Agent]) {
        let mut best_target = None;
        let mut best_score = f64::NEG_INFINITY;

        // PREDATOR BEHAVIOR: Hunt prey first if predator
        if self.is_predator() {
            for agent in agents {
                if agent.id() != self.id() && agent.is_prey() {
                    let distance = self.distance_to(agent.x, agent.y);
                    if distance <= self.genes.sense_range * self.genes.territory_size / 100.0 {
                        // Calculate hunting score based on predator genes
                        let energy_score = agent.energy / 100.0;
                        let distance_penalty = distance / self.genes.sense_range;
                        let stealth_bonus = self.genes.stealth;
                        let intelligence_bonus = self.genes.intelligence;

                        let score = energy_score
                            * (1.0 - distance_penalty)
                            * (1.0 + stealth_bonus)
                            * (1.0 + intelligence_bonus);

                        if score > best_score {
                            best_score = score;
                            best_target = Some((agent.x, agent.y, true, "prey"));
                        }
                    }
                }
            }
        }

        // PREY BEHAVIOR: Look for predators to flee from
        if self.is_prey() {
            for agent in agents {
                if agent.id() != self.id() && agent.is_predator() {
                    let distance = self.distance_to(agent.x, agent.y);
                    if distance <= self.genes.sense_range {
                        // Flee from predators
                        self.state = AgentState::Fleeing;
                        let flee_x = self.x - (agent.x - self.x) * 2.0;
                        let flee_y = self.y - (agent.y - self.y) * 2.0;
                        self.target_x = Some(flee_x);
                        self.target_y = Some(flee_y);
                        return;
                    }
                }
            }
        }

        // Look for resources (both predators and prey eat resources)
        for resource in resources {
            if resource.is_available() {
                let distance = resource.distance_to(self.x, self.y);
                if distance <= self.genes.sense_range {
                    let score = resource.energy / (distance + 1.0);
                    if score > best_score {
                        best_score = score;
                        best_target = Some((resource.x, resource.y, false, "resource"));
                    }
                }
            }
        }

        // Look for other agents (predator vs predator fights)
        if self.is_predator() {
            for agent in agents {
                if agent.id() != self.id() && agent.is_predator() {
                    let distance = self.distance_to(agent.x, agent.y);
                    if distance <= self.genes.sense_range * 0.5 {
                        let size_ratio = agent.genes.size / self.genes.size;
                        let energy_ratio = agent.energy / self.energy;
                        let attack_ratio = self.genes.attack_power / agent.genes.attack_power;

                        // Fight other predators if we have advantage
                        if size_ratio < 0.8
                            && energy_ratio > 0.7
                            && attack_ratio > 1.2
                            && self.genes.aggression > 0.6
                        {
                            let score = agent.energy / (distance + 1.0)
                                * self.genes.aggression
                                * attack_ratio;
                            if score > best_score {
                                best_score = score;
                                best_target = Some((agent.x, agent.y, true, "predator"));
                            }
                        } else if size_ratio > 1.2 && attack_ratio < 0.8 {
                            // Flee from stronger predator
                            self.state = AgentState::Fleeing;
                            let flee_x = self.x - (agent.x - self.x) * 1.5;
                            let flee_y = self.y - (agent.y - self.y) * 1.5;
                            self.target_x = Some(flee_x);
                            self.target_y = Some(flee_y);
                            return;
                        }
                    }
                }
            }
        }

        if let Some((tx, ty, _is_agent, _target_type)) = best_target {
            self.target_x = Some(tx);
            self.target_y = Some(ty);
            self.state = AgentState::Hunting;
        } else {
            // Random movement if no targets
            self.random_movement();
        }
    }

    fn hunt_target(&mut self, _delta_time: f64) {
        if let (Some(tx), Some(ty)) = (self.target_x, self.target_y) {
            let dx = tx - self.x;
            let dy = ty - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < 5.0 {
                // Close enough to interact
                self.state = AgentState::Feeding;
            } else {
                // Move towards target with predator-specific speed
                let base_speed = self.genes.speed;
                let hunting_speed = if self.is_predator() {
                    base_speed * self.genes.hunting_speed * 2.0
                } else {
                    base_speed * 2.0
                };

                self.dx = (dx / distance) * hunting_speed;
                self.dy = (dy / distance) * hunting_speed;
            }
        } else {
            self.state = AgentState::Seeking;
        }
    }

    fn feed_on_resource(&mut self, resources: &[Resource]) -> Option<usize> {
        if let (Some(_tx), Some(_ty)) = (self.target_x, self.target_y) {
            for (i, resource) in resources.iter().enumerate() {
                if resource.distance_to(self.x, self.y) < 5.0 {
                    // Consume the resource and gain energy - much more energy from resources
                    self.energy += 50.0 * self.genes.energy_efficiency; // Increased from 20.0
                    if self.energy > self.max_energy {
                        self.energy = self.max_energy;
                    }

                    // Boost reproduction chance when eating
                    if self.energy > 15.0 && self.age > 2.0 {
                        self.state = AgentState::Reproducing;
                    } else {
                        self.state = AgentState::Seeking;
                    }
                    self.target_x = None;
                    self.target_y = None;

                    return Some(i);
                }
            }
        }

        // If we can't find the target resource, go back to seeking
        self.state = AgentState::Seeking;
        self.target_x = None;
        self.target_y = None;

        None
    }

    fn fight_agent(&mut self, agents: &[Agent]) {
        if let (Some(_tx), Some(_ty)) = (self.target_x, self.target_y) {
            for agent in agents {
                if agent.distance_to(self.x, self.y) < 5.0 {
                    // Enhanced combat mechanics using predator genes
                    let my_attack = self.genes.attack_power * self.genes.size * self.energy * 0.01;
                    let my_defense = self.genes.defense * self.genes.size;
                    let their_attack =
                        agent.genes.attack_power * agent.genes.size * agent.energy * 0.01;
                    let their_defense = agent.genes.defense * agent.genes.size;

                    // Calculate combat outcome
                    let my_effective_power = my_attack / (their_defense + 1.0);
                    let their_effective_power = their_attack / (my_defense + 1.0);

                    // Add intelligence and stamina factors
                    let my_intelligence_bonus = self.genes.intelligence * 0.5;
                    let my_stamina_bonus = self.genes.stamina * 0.3;
                    let their_intelligence_bonus = agent.genes.intelligence * 0.5;
                    let their_stamina_bonus = agent.genes.stamina * 0.3;

                    let my_total_power =
                        my_effective_power * (1.0 + my_intelligence_bonus + my_stamina_bonus);
                    let their_total_power = their_effective_power
                        * (1.0 + their_intelligence_bonus + their_stamina_bonus);

                    if my_total_power > their_total_power {
                        // Win the fight - predators get more energy from prey
                        let energy_gain = if self.is_predator() && agent.is_prey() {
                            agent.energy * 1.2 // Predators get more energy from prey
                        } else {
                            agent.energy * 0.6 // Less energy from predator fights
                        };

                        self.energy += energy_gain;
                        self.kills += 1;

                        // Predators get bonus energy from successful hunts
                        if self.is_predator() {
                            self.energy += 20.0 * self.genes.attack_power;
                        }

                        // Boost reproduction chance after successful hunt
                        if self.energy > 15.0 && self.age > 2.0 {
                            self.state = AgentState::Reproducing;
                        }

                        // Check if opponent died from combat
                        if agent.energy <= 0.0 {
                            // Mark opponent as killed by predator or combat
                            if self.is_predator() && agent.is_prey() {
                                // This will be handled in the simulation loop
                            }
                        }
                    } else {
                        // Lose the fight
                        let damage = their_total_power * 0.1;
                        self.energy -= damage;

                        // Check if we died from combat
                        if self.energy <= 0.0 {
                            self.is_dying = true;
                            self.death_reason = Some(DeathReason::Combat);
                            self.death_fade = 0.0;
                            // Don't return None here, let the main update loop handle it
                        }

                        self.state = AgentState::Fleeing;
                    }
                    break;
                }
            }
        }
        self.state = AgentState::Seeking;
        self.target_x = None;
        self.target_y = None;
    }

    fn flee_from_danger(&mut self, _delta_time: f64) {
        if let (Some(tx), Some(ty)) = (self.target_x, self.target_y) {
            let dx = tx - self.x;
            let dy = ty - self.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > self.genes.sense_range {
                self.state = AgentState::Seeking;
                self.target_x = None;
                self.target_y = None;
            } else {
                let speed = self.genes.speed * 3.0; // Faster when fleeing
                self.dx = (dx / distance) * speed;
                self.dy = (dy / distance) * speed;
            }
        }
    }

    fn reproduce(&mut self) {
        // Reproduction costs energy - higher cost to prevent overpopulation
        self.energy *= 0.7; // Increased from 0.9 - more punishing
        self.last_reproduction = self.age;
        self.state = AgentState::Seeking;
    }

    fn move_agent(&mut self, delta_time: f64, canvas_width: f64, canvas_height: f64) {
        // Apply movement
        self.x += self.dx * delta_time;
        self.y += self.dy * delta_time;

        // Boundary wrapping - these will be set by the simulation
        // For now, use reasonable defaults
        let max_x = canvas_width;
        let max_y = canvas_height;
        if self.x < 0.0 {
            self.x = max_x;
        }
        if self.x > max_x {
            self.x = 0.0;
        }
        if self.y < 0.0 {
            self.y = max_y;
        }
        if self.y > max_y {
            self.y = 0.0;
        }

        // Add some randomness to movement
        let mut rng = thread_rng();
        if rng.gen::<f64>() < 0.01 {
            let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
            self.dx += angle.cos() * 0.1;
            self.dy += angle.sin() * 0.1;
        }

        // Normalize direction vector
        let length = (self.dx * self.dx + self.dy * self.dy).sqrt();
        if length > 0.0 {
            self.dx /= length;
            self.dy /= length;
        }
    }

    fn random_movement(&mut self) {
        let mut rng = thread_rng();
        let angle = rng.gen_range(0.0..2.0 * std::f64::consts::PI);
        self.dx = angle.cos() * self.genes.speed;
        self.dy = angle.sin() * self.genes.speed;
    }

    pub fn can_reproduce(&self) -> bool {
        self.energy > 10.0 && self.age > 2.0 && self.age - self.last_reproduction > 1.0
        // Much faster reproduction - reproduce when they have energy
    }

    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn id(&self) -> u64 {
        // Simple ID based on position and generation
        ((self.x * 1000.0) as u64) ^ ((self.y * 1000.0) as u64) ^ (self.generation as u64)
    }

    pub fn is_alive(&self) -> bool {
        self.energy > 0.0 && self.age < 200.0
    }

    pub fn is_predator(&self) -> bool {
        self.genes.is_predator > 0.5
    }

    pub fn is_prey(&self) -> bool {
        !self.is_predator()
    }

    pub fn create_offspring(&self, other: &Agent) -> Self {
        let new_genes = self
            .genes
            .inherit_from(&other.genes, self.genes.mutation_rate);
        let mut rng = thread_rng();

        // Position offspring near parent
        let offset_x = rng.gen_range(-10.0..10.0);
        let offset_y = rng.gen_range(-10.0..10.0);
        let spawn_x = self.x + offset_x;
        let spawn_y = self.y + offset_y;

        let mut offspring = Self::new(spawn_x, spawn_y, new_genes, self.generation + 1);

        // Set spawn position for proper fade-in
        offspring.spawn_position = Some((spawn_x, spawn_y));
        offspring.spawn_fade = 0.0; // Start invisible

        offspring
    }
}
