use glam::{FloatExt, Vec3};
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

    /// Terrain height at an integer lattice point. This is the single source of
    /// truth for terrain: chunk generation and `height_at` both sample it, so
    /// the surface cannot disagree with itself across chunk borders.
    fn corner_height(&self, ix: i32, iz: i32) -> f32 {
        let wx = ix as f64;
        let wz = iz as f64;
        let base = self.noise.get([wx * WORLD_SCALE, wz * WORLD_SCALE]) as f32;
        let detail = self.noise.get([wx * WORLD_SCALE * 3.0, wz * WORLD_SCALE * 3.0]) as f32;
        (base * 0.7 + detail * 0.3) * HEIGHT_SCALE
    }

    fn generate_chunk(&self, cx: i32, cz: i32) -> Chunk {
        let mut heights = vec![0.0; (CHUNK_SIZE * CHUNK_SIZE) as usize];
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                heights[(z * CHUNK_SIZE + x) as usize] =
                    self.corner_height(cx * CHUNK_SIZE + x, cz * CHUNK_SIZE + z);
            }
        }
        Chunk { coord: (cx, cz), heights }
    }

    /// Bilinearly interpolated terrain height.
    ///
    /// Samples the four surrounding lattice corners directly instead of
    /// reading a cached chunk, so the surface is continuous across chunk
    /// borders. It needs no chunk cache, which makes it `&self`, cheap
    /// (4 corner samples instead of generating a 32x32 chunk), and free of
    /// side effects on the streaming state.
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let fx = x.floor();
        let fz = z.floor();
        let tx = x - fx;
        let tz = z - fz;
        let ix = fx as i32;
        let iz = fz as i32;
        let h00 = self.corner_height(ix, iz);
        let h10 = self.corner_height(ix.saturating_add(1), iz);
        let h01 = self.corner_height(ix, iz.saturating_add(1));
        let h11 = self.corner_height(ix.saturating_add(1), iz.saturating_add(1));
        h00.lerp(h10, tx).lerp(h01.lerp(h11, tx), tz)
    }

    pub fn chunks(&self) -> impl Iterator<Item = &Chunk> { self.chunks.values() }
}
