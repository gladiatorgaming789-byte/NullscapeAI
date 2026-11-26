use nullscape_core::{Action, Simulation, SimulationConfig};
use std::collections::HashMap;
use std::io::{self, BufRead, Write};

fn main() {
    let mut sim = Simulation::new(SimulationConfig::default());
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = match line { Ok(v) => v, Err(_) => break };
        if line.trim().is_empty() { continue; }
        let raw: HashMap<String, Action> = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => { eprintln!("invalid action JSON: {e}"); continue; }
        };
        let actions = raw.into_iter().filter_map(|(k, v)| k.parse::<u32>().ok().map(|id| (id, v))).collect();
        let obs = sim.step(&actions);
        writeln!(stdout, "{}", serde_json::to_string(&obs).expect("serialize")).expect("stdout");
        stdout.flush().expect("flush");
    }
}
