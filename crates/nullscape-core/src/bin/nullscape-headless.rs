use nullscape_core::{protocol::handle_line, Simulation, SimulationConfig};
use std::io::{self, BufRead, Write};

/// Reads one JSON request per line on stdin and writes exactly one JSON
/// response per line on stdout. See `nullscape_core::protocol` for the format.
fn main() {
    let mut sim = Simulation::new(SimulationConfig::default());
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let response = handle_line(&mut sim, &line);
        // A closed pipe means the client went away; exit quietly instead of panicking.
        if writeln!(stdout, "{response}").and_then(|_| stdout.flush()).is_err() {
            break;
        }
    }
}
