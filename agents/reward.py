"""Reward shaping for training and evaluating Nullscape agents."""
from __future__ import annotations

from typing import Any


class RewardModel:
    """Small, deterministic reward model based only on consecutive observations.

    Positive reward encourages approaching and defeating visible enemies.
    Negative reward discourages taking damage and wasting time.
    """

    def __init__(
        self,
        distance_scale: float = 0.02,
        damage_scale: float = 0.05,
        enemy_damage_scale: float = 0.01,
        enemy_defeat_bonus: float = 1.0,
        step_penalty: float = -0.001,
    ):
        if distance_scale < 0 or damage_scale < 0 or enemy_damage_scale < 0:
            raise ValueError("reward scales must be non-negative")
        self.distance_scale = distance_scale
        self.damage_scale = damage_scale
        self.enemy_damage_scale = enemy_damage_scale
        self.enemy_defeat_bonus = enemy_defeat_bonus
        self.step_penalty = step_penalty

    def calculate(self, previous: dict[str, Any], current: dict[str, Any]) -> float:
        reward = self.step_penalty

        previous_enemies = {
            enemy["id"]: enemy
            for enemy in previous.get("nearby_enemies", [])
            if enemy.get("hostile", False)
        }
        current_enemies = {
            enemy["id"]: enemy
            for enemy in current.get("nearby_enemies", [])
            if enemy.get("hostile", False)
        }

        for enemy_id, old_enemy in previous_enemies.items():
            new_enemy = current_enemies.get(enemy_id)
            old_distance = float(old_enemy["distance"])
            if new_enemy is None:
                reward += self.enemy_defeat_bonus
                continue

            reward += (old_distance - float(new_enemy["distance"])) * self.distance_scale
            reward += (
                float(old_enemy["health"]) - float(new_enemy["health"])
            ) * self.enemy_damage_scale

        health_loss = max(
            0.0,
            float(previous.get("health", 0.0)) - float(current.get("health", 0.0)),
        )
        reward -= health_loss * self.damage_scale
        return reward
