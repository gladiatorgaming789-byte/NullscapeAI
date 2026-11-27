"""Training-oriented episode wrapper around the Nullscape JSONL environment."""
from __future__ import annotations

from typing import Any

from agents.environment import NullscapeEnv
from agents.reward import RewardModel


class NullscapeEpisode:
    """Expose reset/step as an AI-friendly episode API.

    The wrapper does not alter the underlying simulation. It selects one agent,
    computes a deterministic reward from consecutive observations, and marks
    an episode done when that agent reaches zero health.
    """

    def __init__(
        self,
        env: NullscapeEnv | None = None,
        agent_id: int = 0,
        reward_model: RewardModel | None = None,
    ):
        if agent_id < 0:
            raise ValueError("agent_id must be non-negative")
        self.env = env or NullscapeEnv()
        self.agent_id = agent_id
        self.reward_model = reward_model or RewardModel()
        self.observation: dict[str, Any] | None = None
        self.done = False

    @staticmethod
    def _agent_observation(observations: list[dict[str, Any]], agent_id: int) -> dict[str, Any]:
        for observation in observations:
            if observation["agent_id"] == agent_id:
                return observation
        raise RuntimeError(f"agent {agent_id} is missing from environment observations")

    def reset(self, seed: int | None = None) -> dict[str, Any]:
        observations = self.env.reset(seed=seed)
        self.observation = self._agent_observation(observations, self.agent_id)
        self.done = False
        return self.observation

    def step(self, action: dict[str, Any]) -> tuple[dict[str, Any], float, bool]:
        if self.done:
            raise RuntimeError("episode is done; call reset() before step()")
        if self.observation is None:
            raise RuntimeError("episode is not reset")

        previous = self.observation
        observations = self.env.step({self.agent_id: action})
        current = self._agent_observation(observations, self.agent_id)
        reward = self.reward_model.calculate(previous, current)
        self.observation = current
        self.done = current["health"] <= 0.0
        return current, reward, self.done

    def close(self) -> None:
        self.env.close()

    def __enter__(self) -> "NullscapeEpisode":
        self.reset()
        return self

    def __exit__(self, *exc_info) -> None:
        self.close()
