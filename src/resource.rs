use serde::{Deserialize, Serialize};

/// Snapshot DTO exposed to rendering and external API callers.
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
    pub target_energy: f64,
    pub is_spawning: bool,
    pub spawn_fade: f64,
    pub is_depleting: bool,
    pub deplete_fade: f64,
}
