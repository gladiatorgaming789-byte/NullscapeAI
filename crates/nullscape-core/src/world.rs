use glam::Vec3;
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const CHUNK_SIZE: i32 = 32;
const WORLD_SCALE: f64 = 0.045;
const HEIGHT_SCALE: f32 = 8.0;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk {
    pub coord: (i32, i32),
    pub heights: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct World {
    pub seed: u32,
    pub chunk_radius: i32,
    chunks: HashMap<(i32, i32), Chunk>,
    noise: Perlin,
}

impl World {
    pub fn new(seed: u32, chunk_radius: i32) -> Self {
        Self { seed, chunk_radius, chunks: HashMap::new(), noise: Perlin::new(seed) }
    }

    pub fn stream_around(&mut self, position: Vec3) {
        let cx = (position.x / CHUNK_SIZE as f32).floor() as i32;
        let cz = (position.z / CHUNK_SIZE as f32).floor() as i32;
        for z in (cz - self.chunk_radius)..=(cz + self.chunk_radius) {
            for x in (cx - self.chunk_radius)..=(cx + self.chunk_radius) {
                if !self.chunks.contains_key(&(x, z)) {
                    let chunk = self.generate_chunk(x, z);
                    self.chunks.insert((x, z), chunk);
                }
            }
        }
        let r = self.chunk_radius + 1;
        self.chunks.retain(|(x, z), _| (x - cx).abs() <= r && (z - cz).abs() <= r);
    }

    fn generate_chunk(&self, cx: i32, cz: i32) -> Chunk {
        let mut heights = vec![0.0; (CHUNK_SIZE * CHUNK_SIZE) as usize];
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let wx = (cx * CHUNK_SIZE + x) as f64;
                let wz = (cz * CHUNK_SIZE + z) as f64;
                let base = self.noise.get([wx * WORLD_SCALE, wz * WORLD_SCALE]) as f32;
                let detail = self.noise.get([wx * WORLD_SCALE * 3.0, wz * WORLD_SCALE * 3.0]) as f32;
                heights[(z * CHUNK_SIZE + x) as usize] = (base * 0.7 + detail * 0.3) * HEIGHT_SCALE;
            }
        }
        Chunk { coord: (cx, cz), heights }
    }

    pub fn height_at(&mut self, x: f32, z: f32) -> f32 {
        let cx = (x / CHUNK_SIZE as f32).floor() as i32;
        let cz = (z / CHUNK_SIZE as f32).floor() as i32;
        let lx = x - cx as f32 * CHUNK_SIZE as f32;
        let lz = z - cz as f32 * CHUNK_SIZE as f32;
        if !self.chunks.contains_key(&(cx, cz)) {
            let chunk = self.generate_chunk(cx, cz);
            self.chunks.insert((cx, cz), chunk);
        }
        let chunk = self.chunks.get(&(cx, cz)).expect("chunk exists");
        let x0 = lx.floor().clamp(0.0, (CHUNK_SIZE - 1) as f32) as usize;
        let z0 = lz.floor().clamp(0.0, (CHUNK_SIZE - 1) as f32) as usize;
        let x1 = (x0 + 1).min((CHUNK_SIZE - 1) as usize);
        let z1 = (z0 + 1).min((CHUNK_SIZE - 1) as usize);
        let tx = lx.fract();
        let tz = lz.fract();
        let h00 = chunk.heights[z0 * CHUNK_SIZE as usize + x0];
        let h10 = chunk.heights[z0 * CHUNK_SIZE as usize + x1];
        let h01 = chunk.heights[z1 * CHUNK_SIZE as usize + x0];
        let h11 = chunk.heights[z1 * CHUNK_SIZE as usize + x1];
        h00.lerp(h10, tx).lerp(h01.lerp(h11, tx), tz)
    }

    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> { self.chunks.values() }
}
