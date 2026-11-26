use crate::{api::{vec3_to_array, Action, AgentId, EnemyObservation, Observation}, enemy::{Enemy, EnemyState}, math::horizontal, world::World};
use glam::{Vec3, EulerRot, Quat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// How far above the terrain surface a descending agent still counts as
/// standing on it. Lets agents walk down slopes without flickering airborne.
const GROUND_SNAP: f32 = 0.1;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub seed: u32,
    pub fixed_dt: f32,
    pub gravity: f32,
    pub move_speed: f32,
    pub jump_speed: f32,
    pub max_health: f32,
    pub agent_count: u32,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self { seed: 1337, fixed_dt: 1.0 / 60.0, gravity: 22.0, move_speed: 5.5, jump_speed: 8.5, max_health: 100.0, agent_count: 2 }
    }
}

#[derive(Clone, Debug)]
pub struct Agent {
    pub id: AgentId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub yaw: f32,
    pub health: f32,
    pub on_ground: bool,
}

pub struct Simulation {
    pub config: SimulationConfig,
    pub tick: u64,
    pub world: World,
    pub agents: HashMap<AgentId, Agent>,
    pub enemies: Vec<Enemy>,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        let world = World::new(config.seed, 2);
        let mut agents = HashMap::new();
        for id in 0..config.agent_count {
            let x = id as f32 * 3.0;
            let y = world.height_at(x, 0.0) + 1.0;
            agents.insert(id, Agent { id, position: Vec3::new(x, y, 0.0), velocity: Vec3::ZERO, yaw: 0.0, health: config.max_health, on_ground: true });
        }
        let enemy_y = world.height_at(8.0, 0.0) + 1.0;
        Self { config, tick: 0, world, agents, enemies: vec![Enemy::new(0, Vec3::new(8.0, enemy_y, 0.0))] }
    }

    pub fn step(&mut self, actions: &HashMap<AgentId, Action>) -> Vec<Observation> {
        let dt = self.config.fixed_dt;
        let targets: HashMap<AgentId, Vec3> = self.agents.iter().map(|(&id, a)| (id, a.position)).collect();

        for agent in self.agents.values_mut() {
            let action = actions.get(&agent.id).copied().unwrap_or_default();
            agent.yaw += action.yaw_delta;
            let rotation = Quat::from_euler(EulerRot::XYZ, 0.0, agent.yaw, 0.0);
            let input = Vec3::new(action.move_x.clamp(-1.0, 1.0), 0.0, action.move_z.clamp(-1.0, 1.0));
            let wish = horizontal(rotation * input).normalize_or_zero();
            agent.velocity.x = wish.x * self.config.move_speed;
            agent.velocity.z = wish.z * self.config.move_speed;

            if action.jump && agent.on_ground {
                agent.velocity.y = self.config.jump_speed;
                agent.on_ground = false;
            }

            agent.velocity.y -= self.config.gravity * dt;
            agent.position += agent.velocity * dt;

            // `on_ground` is recomputed every tick. Previously it was only ever
            // cleared by jumping, so walking off a ledge left it stuck at true
            // (air-jumps, wrong observations).
            let ground = self.world.height_at(agent.position.x, agent.position.z) + 1.0;
            let penetrating = agent.position.y <= ground;
            let resting_or_descending_near_ground =
                agent.velocity.y <= 0.0 && agent.position.y <= ground + GROUND_SNAP;
            if penetrating || resting_or_descending_near_ground {
                agent.position.y = ground;
                agent.velocity.y = 0.0;
                agent.on_ground = true;
            } else {
                agent.on_ground = false;
            }

            if action.attack {
                for enemy in &mut self.enemies {
                    if enemy.state != EnemyState::Dead && enemy.position.distance(agent.position) <= 2.5 {
                        enemy.health = (enemy.health - 20.0).max(0.0);
                    }
                }
            }
            agent.health = agent.health.clamp(0.0, self.config.max_health);
        }

        let target = targets.get(&0).copied().unwrap_or(Vec3::ZERO);
        let damage_events: f32 = self.enemies.iter_mut().filter_map(|enemy| enemy.step(target, dt)).sum();
        if damage_events > 0.0 {
            if let Some(agent) = self.agents.get_mut(&0) {
                agent.health = (agent.health - damage_events).max(0.0);
            }
        }

        self.tick += 1;
        let stream_position = self.agents.get(&0).map(|a| a.position).unwrap_or(Vec3::ZERO);
        self.world.stream_around(stream_position);
        self.observations()
    }

    pub fn observations(&self) -> Vec<Observation> {
        let mut observations: Vec<Observation> = self.agents.values().map(|agent| {
            let mut nearby_enemies: Vec<EnemyObservation> = self.enemies.iter()
                .filter(|e| e.state != EnemyState::Dead)
                .map(|e| EnemyObservation {
                    id: e.id,
                    position: vec3_to_array(e.position),
                    health: e.health,
                    distance: e.position.distance(agent.position),
                    hostile: true,
                })
                .filter(|e| e.distance <= 25.0)
                .collect();
            nearby_enemies.sort_by_key(|e| e.id);

            Observation {
                tick: self.tick,
                agent_id: agent.id,
                position: vec3_to_array(agent.position),
                velocity: vec3_to_array(agent.velocity),
                yaw: agent.yaw,
                health: agent.health,
                on_ground: agent.on_ground,
                nearby_enemies,
                terrain_height: self.terrain_height(agent.position.x, agent.position.z),
            }
        }).collect();

        // HashMap iteration order is randomized. Sorting makes replay fingerprints stable.
        observations.sort_by_key(|o| o.agent_id);
        observations
    }

    fn terrain_height(&self, x: f32, z: f32) -> f32 {
        self.world.height_at(x, z)
    }

    pub fn deterministic_fingerprint(&self) -> String {
        serde_json::to_string(&self.observations()).expect("observations serialize")
    }
}
