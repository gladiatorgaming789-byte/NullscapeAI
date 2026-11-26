use nullscape_core::world::{World, CHUNK_SIZE};

/// The terrain must be continuous everywhere, including across chunk borders.
/// Before the fix there was a ~1.0 unit step at every border (the last cell of
/// a chunk was not interpolated towards the next chunk).
#[test]
fn terrain_is_continuous_across_chunk_borders() {
    let w = World::new(1337, 2);
    let eps = 0.001;
    let mut worst = 0.0f32;
    for k in -20..20 {
        let border = (k * CHUNK_SIZE) as f32;
        for &z in &[-37.25, 0.0, 5.0, 19.9, 63.5] {
            let jump = (w.height_at(border - eps, z) - w.height_at(border + eps, z)).abs();
            worst = worst.max(jump);
        }
        // and across z borders
        for &x in &[-12.5, 3.0, 41.75] {
            let jump = (w.height_at(x, border - eps) - w.height_at(x, border + eps)).abs();
            worst = worst.max(jump);
        }
    }
    // Max terrain slope is well under 1 unit/unit; 0.002 apart => < 0.01.
    assert!(worst < 0.01, "step of {worst} across a chunk border");
}

#[test]
fn height_at_matches_generated_chunk_cells() {
    let mut w = World::new(99, 1);
    w.stream_around(glam::Vec3::ZERO);
    let chunks: Vec<_> = w.chunks().cloned().collect();
    assert!(!chunks.is_empty());
    for c in chunks {
        for &(lx, lz) in &[(0, 0), (5, 9), (31, 31), (17, 2)] {
            let x = (c.coord.0 * CHUNK_SIZE + lx) as f32;
            let z = (c.coord.1 * CHUNK_SIZE + lz) as f32;
            let cell = c.heights[(lz * CHUNK_SIZE + lx) as usize];
            assert!((w.height_at(x, z) - cell).abs() < 1e-5);
        }
    }
}

#[test]
fn height_at_has_no_side_effects_and_is_repeatable() {
    let w = World::new(7, 1);
    assert_eq!(w.chunks().count(), 0);
    let a = w.height_at(-123.4, 987.6);
    let b = w.height_at(-123.4, 987.6);
    assert_eq!(a, b);
    assert_eq!(w.chunks().count(), 0, "height_at must not populate the chunk cache");
}

#[test]
fn extreme_coordinates_do_not_panic() {
    let w = World::new(1, 1);
    for &v in &[f32::MAX, f32::MIN, 3.0e9, -3.0e9] {
        let _ = w.height_at(v, v);
    }
}
