use rand::prelude::*;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Genes {
    pub speed: f64,                  // Movement speed multiplier
    pub sense_range: f64,            // How far the agent can sense resources and other agents
    pub size: f64,                   // Physical size (affects energy consumption and reproduction)
    pub energy_efficiency: f64,      // How efficiently the agent uses energy
    pub reproduction_threshold: f64, // Energy needed to reproduce
    pub mutation_rate: f64,          // How likely genes are to mutate
    pub aggression: f64,             // How likely to attack other agents
    pub color_hue: f64,              // Visual trait for identification

    // NEW PREDATOR GENES
    pub is_predator: f64,    // Probability of being a predator (0.0-1.0)
    pub hunting_speed: f64,  // Speed multiplier when hunting
    pub attack_power: f64,   // Damage dealt when attacking
    pub defense: f64,        // Resistance to attacks
    pub stealth: f64,        // Ability to sneak up on prey
    pub pack_mentality: f64, // Tendency to hunt in groups
    pub territory_size: f64, // Size of hunting territory
    pub metabolism: f64,     // How fast energy is consumed
    pub intelligence: f64,   // Learning and adaptation ability
    pub stamina: f64,        // How long can chase prey
}

impl Default for Genes {
    fn default() -> Self {
        Self::new()
    }
}

impl Genes {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        Self {
            speed: rng.gen_range(0.8..1.5),             // Reduced from 0.5..2.0
            sense_range: rng.gen_range(30.0..80.0),     // Reduced from 20.0..100.0
            size: rng.gen_range(0.9..1.3),              // Reduced from 0.8..1.5
            energy_efficiency: rng.gen_range(0.8..1.2), // Reduced from 0.7..1.3
            reproduction_threshold: rng.gen_range(60.0..120.0), // Reduced from 50.0..150.0
            mutation_rate: rng.gen_range(0.02..0.08),   // Reduced from 0.01..0.1
            aggression: rng.gen_range(0.2..0.8),        // Reduced from 0.0..1.0
            color_hue: rng.gen_range(0.0..360.0),

            // NEW PREDATOR GENES
            is_predator: rng.gen_range(0.0..0.3), // 30% chance of being predator
            hunting_speed: rng.gen_range(1.0..2.0), // Hunters are faster
            attack_power: rng.gen_range(0.5..1.5), // Attack strength
            defense: rng.gen_range(0.5..1.5),     // Defense against attacks
            stealth: rng.gen_range(0.0..1.0),     // Stealth ability
            pack_mentality: rng.gen_range(0.0..1.0), // Group hunting tendency
            territory_size: rng.gen_range(50.0..150.0), // Hunting territory
            metabolism: rng.gen_range(0.8..1.4),  // Energy consumption rate
            intelligence: rng.gen_range(0.5..1.5), // Learning ability
            stamina: rng.gen_range(0.5..1.5),     // Chase endurance
        }
    }

    pub fn inherit_from(&self, other: &Genes, mutation_rate: f64) -> Self {
        let mut rng = thread_rng();

        Self {
            speed: self.mutate_gene(self.speed, other.speed, mutation_rate, &mut rng),
            sense_range: self.mutate_gene(
                self.sense_range,
                other.sense_range,
                mutation_rate,
                &mut rng,
            ),
            size: self.mutate_gene(self.size, other.size, mutation_rate, &mut rng),
            energy_efficiency: self.mutate_gene(
                self.energy_efficiency,
                other.energy_efficiency,
                mutation_rate,
                &mut rng,
            ),
            reproduction_threshold: self.mutate_gene(
                self.reproduction_threshold,
                other.reproduction_threshold,
                mutation_rate,
                &mut rng,
            ),
            mutation_rate: self.mutate_gene(
                self.mutation_rate,
                other.mutation_rate,
                mutation_rate,
                &mut rng,
            ),
            aggression: self.mutate_gene(
                self.aggression,
                other.aggression,
                mutation_rate,
                &mut rng,
            ),
            color_hue: self.mutate_gene(self.color_hue, other.color_hue, mutation_rate, &mut rng),

            // NEW PREDATOR GENES
            is_predator: self.mutate_gene(
                self.is_predator,
                other.is_predator,
                mutation_rate,
                &mut rng,
            ),
            hunting_speed: self.mutate_gene(
                self.hunting_speed,
                other.hunting_speed,
                mutation_rate,
                &mut rng,
            ),
            attack_power: self.mutate_gene(
                self.attack_power,
                other.attack_power,
                mutation_rate,
                &mut rng,
            ),
            defense: self.mutate_gene(self.defense, other.defense, mutation_rate, &mut rng),
            stealth: self.mutate_gene(self.stealth, other.stealth, mutation_rate, &mut rng),
            pack_mentality: self.mutate_gene(
                self.pack_mentality,
                other.pack_mentality,
                mutation_rate,
                &mut rng,
            ),
            territory_size: self.mutate_gene(
                self.territory_size,
                other.territory_size,
                mutation_rate,
                &mut rng,
            ),
            metabolism: self.mutate_gene(
                self.metabolism,
                other.metabolism,
                mutation_rate,
                &mut rng,
            ),
            intelligence: self.mutate_gene(
                self.intelligence,
                other.intelligence,
                mutation_rate,
                &mut rng,
            ),
            stamina: self.mutate_gene(self.stamina, other.stamina, mutation_rate, &mut rng),
        }
    }

    fn mutate_gene(&self, gene1: f64, gene2: f64, mutation_rate: f64, rng: &mut ThreadRng) -> f64 {
        // Blend genes from both parents
        let blend_factor = rng.gen_range(0.3..0.7);
        let mut gene = gene1 * blend_factor + gene2 * (1.0 - blend_factor);

        // Apply mutation
        if rng.gen::<f64>() < mutation_rate {
            let mutation_dist = Normal::new(0.0, 0.05).unwrap(); // Reduced from 0.1
            let mutation = mutation_dist.sample(rng);
            gene += mutation;
        }

        // Clamp values to reasonable ranges
        match gene {
            speed if speed < 0.1 => 0.1,
            speed if speed > 3.0 => 3.0, // Reduced from 5.0
            sense_range if sense_range < 5.0 => 5.0,
            sense_range if sense_range > 150.0 => 150.0, // Reduced from 200.0
            size if size < 0.3 => 0.3,
            size if size > 2.5 => 2.5, // Reduced from 3.0
            energy_efficiency if energy_efficiency < 0.1 => 0.1,
            energy_efficiency if energy_efficiency > 2.5 => 2.5, // Reduced from 3.0
            reproduction_threshold if reproduction_threshold < 10.0 => 10.0,
            reproduction_threshold if reproduction_threshold > 200.0 => 200.0, // Reduced from 300.0
            mutation_rate if mutation_rate < 0.001 => 0.001,
            mutation_rate if mutation_rate > 0.3 => 0.3, // Reduced from 0.5
            aggression if aggression < 0.0 => 0.0,
            aggression if aggression > 1.0 => 1.0,
            color_hue if color_hue < 0.0 => 0.0,
            color_hue if color_hue > 360.0 => 360.0,

            // NEW PREDATOR GENE CLAMPING
            is_predator if is_predator < 0.0 => 0.0,
            is_predator if is_predator > 1.0 => 1.0,
            hunting_speed if hunting_speed < 0.5 => 0.5,
            hunting_speed if hunting_speed > 3.0 => 3.0,
            attack_power if attack_power < 0.1 => 0.1,
            attack_power if attack_power > 3.0 => 3.0,
            defense if defense < 0.1 => 0.1,
            defense if defense > 3.0 => 3.0,
            stealth if stealth < 0.0 => 0.0,
            stealth if stealth > 1.0 => 1.0,
            pack_mentality if pack_mentality < 0.0 => 0.0,
            pack_mentality if pack_mentality > 1.0 => 1.0,
            territory_size if territory_size < 10.0 => 10.0,
            territory_size if territory_size > 300.0 => 300.0,
            metabolism if metabolism < 0.1 => 0.1,
            metabolism if metabolism > 3.0 => 3.0,
            intelligence if intelligence < 0.1 => 0.1,
            intelligence if intelligence > 3.0 => 3.0,
            stamina if stamina < 0.1 => 0.1,
            stamina if stamina > 3.0 => 3.0,
            _ => gene,
        }
    }

    pub fn get_fitness_score(&self) -> f64 {
        // Calculate overall fitness based on gene combinations
        let speed_score = self.speed * 0.2;
        let sense_score = (self.sense_range / 100.0) * 0.15;
        let efficiency_score = self.energy_efficiency * 0.2;
        let size_score = (1.0 / self.size) * 0.1; // Smaller is better for energy efficiency
        let reproduction_score = (1.0 / self.reproduction_threshold) * 50.0 * 0.1;

        // NEW PREDATOR FITNESS SCORES
        let predator_bonus = if self.is_predator > 0.5 { 0.5 } else { 0.0 }; // Predators get bonus
        let hunting_score = self.hunting_speed * 0.1;
        let attack_score = self.attack_power * 0.1;
        let defense_score = self.defense * 0.1;
        let intelligence_score = self.intelligence * 0.1;
        let stamina_score = self.stamina * 0.1;

        speed_score
            + sense_score
            + efficiency_score
            + size_score
            + reproduction_score
            + predator_bonus
            + hunting_score
            + attack_score
            + defense_score
            + intelligence_score
            + stamina_score
    }
}
