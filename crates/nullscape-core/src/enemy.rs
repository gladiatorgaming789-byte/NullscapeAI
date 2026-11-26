use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EnemyState { Idle, Chase, Attack, Dead }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Enemy {
    pub id: u32,
    pub position: Vec3,
    pub velocity: Vec3,
    pub health: f32,
    pub state: EnemyState,
    pub attack_cooldown: f32,
}

impl Enemy {
    pub fn new(id: u32, position: Vec3) -> Self {
        Self { id, position, velocity: Vec3::ZERO, health: 40.0, state: EnemyState::Idle, attack_cooldown: 0.0 }
    }

    pub fn step(&mut self, target: Vec3, dt: f32) -> Option<f32> {
        if self.health <= 0.0 { self.state = EnemyState::Dead; return None; }
        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
        let delta = target - self.position;
        let distance = delta.length();
        if distance < 1.8 {
            self.state = EnemyState::Attack;
            self.velocity = Vec3::ZERO;
            if self.attack_cooldown <= 0.0 {
                self.attack_cooldown = 1.0;
                return Some(8.0);
            }
        } else if distance < 18.0 {
            self.state = EnemyState::Chase;
            self.velocity = delta.normalize_or_zero() * 2.6;
            self.position += self.velocity * dt;
        } else {
            self.state = EnemyState::Idle;
            self.velocity = Vec3::ZERO;
        }
        None
    }
}
