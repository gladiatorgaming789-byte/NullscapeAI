"""AI policy interfaces and a deterministic baseline agent for Nullscape.

The policy layer deliberately lives outside the Rust simulation. Rust remains
authoritative for physics, combat, and world state; a policy only observes
state and chooses the next action.
"""
from __future__ import annotations

from abc import ABC, abstractmethod
from math import atan2, pi
from typing import Any


class AgentPolicy(ABC):
    """Interface implemented by an AI controller."""

    @abstractmethod
    def act(self, observation: dict[str, Any]) -> dict[str, Any]:
        """Choose one action from one observation."""
        raise NotImplementedError


class GoalDirectedAgent(AgentPolicy):
    """Deterministic baseline that approaches the nearest visible enemy."""

    def __init__(self, attack_range: float = 2.5, turn_rate: float = 0.35):
        if attack_range <= 0:
            raise ValueError("attack_range must be positive")
        if turn_rate <= 0:
            raise ValueError("turn_rate must be positive")
        self.attack_range = attack_range
        self.turn_rate = turn_rate

    @staticmethod
    def _wrap_angle(angle: float) -> float:
        return (angle + pi) % (2 * pi) - pi

    def act(self, observation: dict[str, Any]) -> dict[str, Any]:
        enemies = [
            enemy for enemy in observation.get("nearby_enemies", [])
            if enemy.get("hostile", False) and enemy.get("health", 0.0) > 0.0
        ]

        if not enemies:
            return {}

        enemy = min(enemies, key=lambda item: (item["distance"], item["id"]))
        px, _, pz = observation["position"]
        ex, _, ez = enemy["position"]
        desired_yaw = atan2(ex - px, ez - pz)
        yaw_error = self._wrap_angle(desired_yaw - observation["yaw"])

        action: dict[str, Any] = {
            "yaw_delta": max(-self.turn_rate, min(self.turn_rate, yaw_error)),
            "move_z": 1.0,
        }
        if enemy["distance"] <= self.attack_range:
            action["attack"] = True
        return action
