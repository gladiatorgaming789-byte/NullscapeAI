use nullscape_core::{Action, Simulation, SimulationConfig};
use std::collections::HashMap;

fn act(a: Action) -> HashMap<u32, Action> {
    let mut m = HashMap::new();
    m.insert(0, a);
    m
}

/// `on_ground` must always agree with where the agent actually is.
#[test]
fn on_ground_flag_matches_reality_while_walking() {
    for seed in [1337u32, 1, 2, 3] {
        let mut sim = Simulation::new(SimulationConfig { seed, ..Default::default() });
        for i in 0..4000 {
            // wander with a slow turn so we cross lots of terrain, with some jumping
            let a = Action { move_z: -1.0, yaw_delta: 0.01, jump: i % 97 == 0, ..Default::default() };
            let o = sim.step(&act(a)).remove(0);
            let ground = o.terrain_height + 1.0;
            let height_above = o.position[1] - ground;
            if o.on_ground {
                assert!(height_above.abs() < 1e-4,
                    "seed {seed} tick {i}: on_ground but {height_above} above terrain");
            } else {
                assert!(height_above > 0.0,
                    "seed {seed} tick {i}: airborne flag but {height_above} above terrain");
            }
        }
    }
}

/// Regression test: `on_ground` used to stay `true` after walking off a ledge
/// (it was only ever cleared by jumping), which allowed jumping in mid-air.
#[test]
fn cannot_jump_after_leaving_the_ground() {
    let mut sim = Simulation::new(SimulationConfig::default());
    sim.step(&act(Action::default())); // settle on the ground
    sim.agents.get_mut(&0).unwrap().position.y += 5.0; // ground suddenly 5 below
    let o = sim.step(&act(Action::default())).remove(0);
    assert!(!o.on_ground, "agent 5 units above terrain reported on_ground");
    let o = sim.step(&act(Action { jump: true, ..Default::default() })).remove(0);
    assert!(!o.on_ground);
    assert!(o.velocity[1] < 0.0, "mid-air jump gave vy = {}", o.velocity[1]);
}

#[test]
fn can_still_jump_from_the_ground_and_land_again() {
    let mut sim = Simulation::new(SimulationConfig::default());
    let o = sim.step(&act(Action::default())).remove(0);
    assert!(o.on_ground);
    let o = sim.step(&act(Action { jump: true, ..Default::default() })).remove(0);
    assert!(!o.on_ground);
    assert!(o.velocity[1] > 0.0);
    let mut landed = false;
    for _ in 0..120 {
        if sim.step(&act(Action::default())).remove(0).on_ground { landed = true; break; }
    }
    assert!(landed, "agent never landed");
}
