use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Resource {
    pub x: f64,
    pub y: f64,
    pub energy: f64,
    pub max_energy: f64,
    pub size: f64,
    pub growth_rate: f64,
    pub regeneration_rate: f64,
    pub age: f64,
    pub target_energy: f64, // Target energy for smooth transitions
    pub is_spawning: bool,  // Whether resource is spawning (fading in)
    pub spawn_fade: f64,    // Spawn fade timer (0.0 = invisible, 1.0 = fully visible)
    pub is_depleting: bool, // Whether resource is being depleted (fading out)
    pub deplete_fade: f64,  // Deplete fade timer (0.0 = fully visible, 1.0 = invisible)
}

impl Resource {
    pub fn new(x: f64, y: f64) -> Self {
        let mut rng = thread_rng();
        let initial_energy = rng.gen_range(15.0..40.0); // Much lower initial energy
        let max_energy = rng.gen_range(30.0..60.0); // Much lower max energy

        Self {
            x,
            y,
            energy: 0.0, // Start with 0 energy and grow smoothly
            max_energy,
            size: 3.0,                                   // Start small and grow
            growth_rate: rng.gen_range(0.1..0.5),        // Much slower growth
            regeneration_rate: rng.gen_range(0.02..0.1), // Much slower regeneration
            age: 0.0,
            target_energy: initial_energy,
            is_spawning: true, // Start in spawning state
            spawn_fade: 0.0,   // Start invisible
            is_depleting: false,
            deplete_fade: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.age += delta_time;

        // Handle spawning fade-in
        if self.is_spawning {
            self.spawn_fade += delta_time * 2.0; // Fade in over 0.5 seconds
            if self.spawn_fade >= 1.0 {
                self.spawn_fade = 1.0;
                self.is_spawning = false;
            }
        }

        // Handle depletion fade-out
        if self.is_depleting {
            self.deplete_fade += delta_time * 3.0; // Fade out over 0.33 seconds
            if self.deplete_fade >= 1.0 {
                self.deplete_fade = 1.0;
                // Resource is fully depleted and invisible
            }
        }

        // Smooth energy growth towards target
        if !self.is_spawning && !self.is_depleting {
            let energy_diff = self.target_energy - self.energy;
            if energy_diff.abs() > 0.1 {
                let growth_direction = if energy_diff > 0.0 { 1.0 } else { -1.0 };
                let growth_amount = self.growth_rate * delta_time * growth_direction;

                if energy_diff.abs() < growth_amount.abs() {
                    self.energy = self.target_energy;
                } else {
                    self.energy += growth_amount;
                }
            }

            // Natural growth towards max energy (much slower and limited)
            if self.energy < self.max_energy {
                self.energy += self.growth_rate * delta_time * 0.05; // Even slower growth
                if self.energy > self.max_energy {
                    self.energy = self.max_energy;
                }
            }

            // Stop growing once at max energy
            if self.energy >= self.max_energy {
                self.target_energy = self.max_energy; // Lock target to max
            }
        }

        // Size changes based on energy (smooth)
        let target_size = 3.0 + (self.energy / self.max_energy) * 5.0;
        let size_diff = target_size - self.size;
        if size_diff.abs() > 0.1 {
            self.size += size_diff * delta_time * 2.0; // Smooth size transitions
        }

        // Regeneration when depleted (much slower)
        if self.energy < 10.0 && !self.is_depleting {
            self.energy += self.regeneration_rate * delta_time * 0.2; // Reduced from 1.0
        }
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

    pub fn is_available(&self) -> bool {
        self.energy > 5.0 && !self.is_depleting && self.spawn_fade > 0.5
    }

    pub fn distance_to(&self, x: f64, y: f64) -> f64 {
        let dx = self.x - x;
        let dy = self.y - y;
        (dx * dx + dy * dy).sqrt()
    }
}
