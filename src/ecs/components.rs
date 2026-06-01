use hecs::Entity;
use serde::{Deserialize, Serialize};

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

impl Resource {
    pub fn update(&mut self, delta_time: f64) {
        self.age += delta_time;

        if self.is_spawning {
            self.spawn_fade = (self.spawn_fade + delta_time * 2.0).min(1.0);
            self.is_spawning = self.spawn_fade < 1.0;
        }

        if self.is_depleting {
            self.deplete_fade = (self.deplete_fade + delta_time * 3.0).min(1.0);
        }

        let target_size = 3.0 + (self.energy / self.max_energy) * 5.0;
        let size_diff = target_size - self.size;
        if size_diff.abs() > 0.1 {
            self.size += size_diff * delta_time * 2.0;
        }
    }

    pub fn is_available(&self) -> bool {
        self.energy > 5.0 && !self.is_depleting && self.spawn_fade > 0.5
    }

    pub fn consume(&mut self, amount: f64) -> f64 {
        let consumed = amount.min(self.energy);
        self.energy -= consumed;

        if self.energy <= 0.0 {
            self.is_depleting = true;
            self.deplete_fade = 0.0;
            self.energy = 0.0;
        }

        consumed
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Size {
    pub value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentTag;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceTag;

#[derive(Clone, Debug)]
pub enum FrameEvent {
    ResourceConsumed {
        agent: Entity,
        resource: Entity,
        amount: f64,
    },
    AgentBorn {
        agent: Entity,
        generation: u32,
    },
    AgentDied {
        agent: Entity,
        reason: DeathReason,
    },
    AgentKilled {
        predator: Entity,
        prey: Entity,
    },
}
