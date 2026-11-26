use nullscape_core::{Action, Simulation, SimulationConfig};
use std::collections::HashMap;

#[test]
fn same_seed_and_actions_are_identical() {
    let cfg = SimulationConfig { seed: 424242, ..Default::default() };
    let mut a = Simulation::new(cfg);
    let mut b = Simulation::new(cfg);
    for i in 0..120 {
        let action = Action { move_z: if i < 60 { -1.0 } else { 0.5 }, jump: i == 10, attack: i > 90, ..Default::default() };
        let mut actions = HashMap::new();
        actions.insert(0, action);
        actions.insert(1, Action::default());
        a.step(&actions);
        b.step(&actions);
        assert_eq!(a.deterministic_fingerprint(), b.deterministic_fingerprint());
    }
}

#[test]
fn different_seeds_change_terrain() {
    let a = Simulation::new(SimulationConfig { seed: 1, ..Default::default() });
    let b = Simulation::new(SimulationConfig { seed: 2, ..Default::default() });
    let ha = a.world.height_at(12.5, -7.25);
    let hb = b.world.height_at(12.5, -7.25);
    assert_ne!(ha, hb);
}
