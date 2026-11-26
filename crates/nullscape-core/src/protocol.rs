//! JSONL request/response protocol used by the headless binary.
//!
//! Invariant: **every request line produces exactly one response line.**
//! Clients can therefore block on one `readline()` per request without risk of
//! desynchronising, even when they send something invalid.
//!
//! Requests (one JSON object per line):
//! - Actions: `{"0": {"move_z": -1.0, "jump": true}, "1": {}}`. Keys are agent
//!   ids, values are [`Action`]s with all fields optional. Agents that are
//!   omitted idle. `{}` advances one tick with every agent idle.
//! - Commands: `{"cmd": "reset"}` or `{"cmd": "reset", "seed": 42}` restarts
//!   the simulation (optionally with a new terrain seed) and returns the tick-0
//!   observations.
//!
//! Responses:
//! - Success: the JSON array of per-agent observations, ordered by agent id.
//! - Failure: `{"error": "<message>"}`. A failed request never advances or
//!   otherwise modifies the simulation.

use crate::{Action, AgentId, Simulation};
use serde_json::{json, Map, Value};
use std::collections::HashMap;

/// Handles one request line and returns the one-line JSON response.
pub fn handle_line(sim: &mut Simulation, line: &str) -> String {
    match process(sim, line) {
        Ok(response) => response,
        Err(message) => json!({ "error": message }).to_string(),
    }
}

fn process(sim: &mut Simulation, line: &str) -> Result<String, String> {
    let value: Value = serde_json::from_str(line).map_err(|e| format!("invalid JSON: {e}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| "request must be a JSON object".to_string())?;

    if let Some(cmd) = object.get("cmd") {
        return command(sim, object, cmd);
    }

    // Validate every action before stepping so a bad request has no effect.
    let mut actions: HashMap<AgentId, Action> = HashMap::new();
    for (key, raw) in object {
        let id: AgentId = key
            .parse()
            .map_err(|_| format!("invalid agent id {key:?}: expected a non-negative integer"))?;
        if id >= sim.config.agent_count {
            return Err(format!(
                "unknown agent id {id}: this simulation has {} agents (ids 0..{})",
                sim.config.agent_count,
                sim.config.agent_count.saturating_sub(1)
            ));
        }
        let action: Action = serde_json::from_value(raw.clone())
            .map_err(|e| format!("invalid action for agent {id}: {e}"))?;
        actions.insert(id, action);
    }

    to_json(&sim.step(&actions))
}

fn command(sim: &mut Simulation, object: &Map<String, Value>, cmd: &Value) -> Result<String, String> {
    match cmd.as_str() {
        Some("reset") => {
            let mut config = sim.config;
            if let Some(seed) = object.get("seed") {
                config.seed = seed
                    .as_u64()
                    .and_then(|s| u32::try_from(s).ok())
                    .ok_or_else(|| "seed must be an integer in 0..=4294967295".to_string())?;
            }
            *sim = Simulation::new(config);
            to_json(&sim.observations())
        }
        Some(other) => Err(format!("unknown command {other:?} (supported: \"reset\")")),
        None => Err("\"cmd\" must be a string".to_string()),
    }
}

/// Serialises directly (not via `serde_json::Value`) so successful responses
/// keep struct field order and `f32` formatting identical to the original
/// headless output.
fn to_json<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|e| format!("failed to serialize response: {e}"))
}
