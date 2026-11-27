import unittest

from agents.reward import RewardModel


class RewardModelTests(unittest.TestCase):
    def setUp(self):
        self.reward = RewardModel(
            distance_scale=0.02,
            damage_scale=0.05,
            enemy_damage_scale=0.01,
            enemy_defeat_bonus=1.0,
            step_penalty=-0.001,
        )

    def test_approaching_enemy_is_positive(self):
        previous = {
            "health": 100.0,
            "nearby_enemies": [{
                "id": 0, "health": 100.0, "distance": 10.0, "hostile": True,
            }],
        }
        current = {
            "health": 100.0,
            "nearby_enemies": [{
                "id": 0, "health": 100.0, "distance": 9.0, "hostile": True,
            }],
        }
        self.assertAlmostEqual(self.reward.calculate(previous, current), 0.019)

    def test_damage_and_enemy_damage_are_accounted_for(self):
        previous = {
            "health": 100.0,
            "nearby_enemies": [{
                "id": 0, "health": 100.0, "distance": 3.0, "hostile": True,
            }],
        }
        current = {
            "health": 90.0,
            "nearby_enemies": [{
                "id": 0, "health": 80.0, "distance": 3.0, "hostile": True,
            }],
        }
        self.assertAlmostEqual(self.reward.calculate(previous, current), 0.099)

    def test_defeated_enemy_gives_bonus(self):
        previous = {
            "health": 100.0,
            "nearby_enemies": [{
                "id": 0, "health": 20.0, "distance": 2.0, "hostile": True,
            }],
        }
        current = {"health": 100.0, "nearby_enemies": []}
        self.assertAlmostEqual(self.reward.calculate(previous, current), 0.999)


if __name__ == "__main__":
    unittest.main()
