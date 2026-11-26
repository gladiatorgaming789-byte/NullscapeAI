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

Then use agents/environment.py from Python. The simulation accepts one JSON action object per tick and returns observations for all agents.

The current enemy targets agent 0. Agent 1 is independently simulated and shares the same world.

## Accuracy note

This milestone establishes the deterministic architecture and placeholder mechanics. Exact Nullscape movement constants, terrain-generation rules, enemy stats, attacks, and content should be replaced with measured/reference values once source gameplay data is supplied.
