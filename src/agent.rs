use crate::genes::Genes;
use serde::{Deserialize, Serialize};

/// Snapshot DTO exposed to rendering and external API callers.
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
    pub death_fade: f64,
    pub death_reason: Option<DeathReason>,
    pub is_dying: bool,
    pub spawn_fade: f64,
    pub spawn_position: Option<(f64, f64)>,
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
