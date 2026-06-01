use rand::prelude::*;
use rand_distr::StandardNormal;

pub use crate::ecs::Genes;

impl Default for Genes {
    fn default() -> Self {
        Self::new()
    }
}

impl Genes {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        Self {
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

    pub fn inherit_from(&self, other: &Genes, mutation_rate: f64) -> Self {
        let mut rng = thread_rng();

        Self {
            speed: self.mutate_gene(self.speed, other.speed, mutation_rate, &mut rng, 0.1, 3.0),
            sense_range: self.mutate_gene(
                self.sense_range,
                other.sense_range,
                mutation_rate,
                &mut rng,
                5.0,
                150.0,
            ),
            size: self.mutate_gene(self.size, other.size, mutation_rate, &mut rng, 0.3, 2.5),
            energy_efficiency: self.mutate_gene(
                self.energy_efficiency,
                other.energy_efficiency,
                mutation_rate,
                &mut rng,
                0.1,
                2.5,
            ),
            reproduction_threshold: self.mutate_gene(
                self.reproduction_threshold,
                other.reproduction_threshold,
                mutation_rate,
                &mut rng,
                10.0,
                200.0,
            ),
            mutation_rate: self.mutate_gene(
                self.mutation_rate,
                other.mutation_rate,
                mutation_rate,
                &mut rng,
                0.001,
                0.3,
            ),
            aggression: self.mutate_gene(
                self.aggression,
                other.aggression,
                mutation_rate,
                &mut rng,
                0.0,
                1.0,
            ),
            color_hue: self.mutate_gene(
                self.color_hue,
                other.color_hue,
                mutation_rate,
                &mut rng,
                0.0,
                360.0,
            ),
            is_predator: self.mutate_gene(
                self.is_predator,
                other.is_predator,
                mutation_rate,
                &mut rng,
                0.0,
                1.0,
            ),
            hunting_speed: self.mutate_gene(
                self.hunting_speed,
                other.hunting_speed,
                mutation_rate,
                &mut rng,
                0.5,
                3.0,
            ),
            attack_power: self.mutate_gene(
                self.attack_power,
                other.attack_power,
                mutation_rate,
                &mut rng,
                0.1,
                3.0,
            ),
            defense: self.mutate_gene(
                self.defense,
                other.defense,
                mutation_rate,
                &mut rng,
                0.1,
                3.0,
            ),
            stealth: self.mutate_gene(
                self.stealth,
                other.stealth,
                mutation_rate,
                &mut rng,
                0.0,
                1.0,
            ),
            pack_mentality: self.mutate_gene(
                self.pack_mentality,
                other.pack_mentality,
                mutation_rate,
                &mut rng,
                0.0,
                1.0,
            ),
            territory_size: self.mutate_gene(
                self.territory_size,
                other.territory_size,
                mutation_rate,
                &mut rng,
                10.0,
                300.0,
            ),
            metabolism: self.mutate_gene(
                self.metabolism,
                other.metabolism,
                mutation_rate,
                &mut rng,
                0.1,
                3.0,
            ),
            intelligence: self.mutate_gene(
                self.intelligence,
                other.intelligence,
                mutation_rate,
                &mut rng,
                0.1,
                3.0,
            ),
            stamina: self.mutate_gene(
                self.stamina,
                other.stamina,
                mutation_rate,
                &mut rng,
                0.1,
                3.0,
            ),
            personal_space: self.mutate_gene(
                self.personal_space,
                other.personal_space,
                mutation_rate,
                &mut rng,
                5.0,
                100.0,
            ),
        }
    }

    fn mutate_gene(
        &self,
        gene1: f64,
        gene2: f64,
        mutation_rate: f64,
        rng: &mut ThreadRng,
        min: f64,
        max: f64,
    ) -> f64 {
        let blend_factor = rng.gen_range(0.3..0.7);
        let mut gene = gene1 * blend_factor + gene2 * (1.0 - blend_factor);

        if rng.gen::<f64>() < mutation_rate {
            gene += rng.sample::<f64, _>(StandardNormal) * 0.05;
        }

        gene.clamp(min, max)
    }
}
