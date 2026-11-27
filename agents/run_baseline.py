"""Run the deterministic Nullscape AI against the headless simulation."""
from __future__ import annotations

import argparse

from agents.environment import NullscapeEnv
from agents.policy import GoalDirectedAgent


def main() -> None:
    parser = argparse.ArgumentParser(description="Run a baseline AI in Nullscape.")
    parser.add_argument("--ticks", type=int, default=300)
    parser.add_argument("--seed", type=int, default=1337)
    args = parser.parse_args()

    if args.ticks < 1:
        parser.error("--ticks must be positive")

    policy = GoalDirectedAgent()

    with NullscapeEnv() as env:
        observations = env.reset(seed=args.seed)
        for _ in range(args.ticks):
            agent_obs = next(item for item in observations if item["agent_id"] == 0)
            action = policy.act(agent_obs)
            observations = env.step({0: action})
            agent_obs = next(item for item in observations if item["agent_id"] == 0)
            print(
                f"tick={agent_obs['tick']} "
                f"position={tuple(round(v, 2) for v in agent_obs['position'])} "
                f"health={agent_obs['health']:.1f} "
                f"enemies={len(agent_obs['nearby_enemies'])} "
                f"action={action}"
            )


if __name__ == "__main__":
    main()
