use glam::Vec3;
use serde::{Deserialize, Serialize};

pub type AgentId = u32;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Action {
    pub move_x: f32,
    pub move_z: f32,
    pub yaw_delta: f32,
    pub jump: bool,
    pub attack: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub tick: u64,
    pub agent_id: AgentId,
    pub position: [f32; 3],
    pub velocity: [f32; 3],
    pub yaw: f32,
    pub health: f32,
    pub on_ground: bool,
    pub nearby_enemies: Vec<EnemyObservation>,
    pub terrain_height: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyObservation {
    pub id: u32,
    pub position: [f32; 3],
    pub health: f32,
    pub distance: f32,
    pub hostile: bool,
}

pub fn vec3_to_array(v: Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}
