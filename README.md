# NullscapeAI

Standalone Nullscape recreation focused on an AI-playable deterministic simulation.

## Milestone 1

- Rust simulation core independent from rendering
- Deterministic seeded procedural terrain with streamed chunks
- Gravity, movement, jumping, terrain collision
- Two simultaneous agents
- One enemy with idle/chase/attack behavior
- Basic melee combat
- JSONL headless agent API
- Bevy 0.15 visualization client
- Determinism tests

## Requirements

- Rust stable
- Python 3.10+ (optional, for agents)
- A graphics-capable environment for the Bevy client

## Run tests

cargo test -p nullscape-core

## Run the visual client

cargo run -p nullscape-client

Controls for agent 0: WASD, Space to jump, F to attack.

## Headless AI environment

Build the headless binary:

cargo build -p nullscape-core --bin nullscape-headless

Then use agents/environment.py from Python:

```python
from agents.environment import NullscapeEnv

with NullscapeEnv() as env:
    obs = env.step({0: {"move_z": -1.0, "jump": True}})  # unspecified fields default to neutral
    obs = env.reset(seed=42)                              # restart, optionally with a new seed
```

The wire protocol is JSONL with a strict **one response line per request line**:

| Request | Response |
|---|---|
| `{"0": {"move_z": -1.0, "jump": true}, "1": {}}` | JSON array of observations (all agents, sorted by id). Action fields are all optional; omitted agents idle; `{}` advances one tick. |
| `{"cmd": "reset"}` / `{"cmd": "reset", "seed": 42}` | Tick-0 observations of a restarted simulation. |
| anything invalid (bad JSON, unknown agent id, unknown field, wrong type, ...) | `{"error": "<message>"}`; the simulation is **not** advanced. |

Unknown fields are rejected on purpose so typos like `"jmp"` are reported rather than ignored.

The current enemy targets agent 0. Agent 1 is independently simulated and shares the same world.

## Accuracy note

This milestone establishes the deterministic architecture and placeholder mechanics. Exact Nullscape movement constants, terrain-generation rules, enemy stats, attacks, and content should be replaced with measured/reference values once source gameplay data is supplied.
