import math
import unittest

from agents.policy import GoalDirectedAgent


class GoalDirectedAgentTests(unittest.TestCase):
    def test_no_enemy_is_neutral(self):
        agent = GoalDirectedAgent()
        observation = {
            "position": [0.0, 1.0, 0.0],
            "yaw": 0.0,
            "nearby_enemies": [],
        }
        self.assertEqual(agent.act(observation), {})

    def test_turns_toward_enemy_and_moves(self):
        agent = GoalDirectedAgent(turn_rate=0.35)
        observation = {
            "position": [0.0, 1.0, 0.0],
            "yaw": 0.0,
            "nearby_enemies": [{
                "id": 0,
                "position": [8.0, 1.0, 0.0],
                "health": 100.0,
                "distance": 8.0,
                "hostile": True,
            }],
        }
        action = agent.act(observation)
        self.assertEqual(action["move_z"], 1.0)
        self.assertAlmostEqual(action["yaw_delta"], 0.35)

    def test_attacks_when_in_range(self):
        agent = GoalDirectedAgent()
        observation = {
            "position": [0.0, 1.0, 0.0],
            "yaw": math.pi / 2,
            "nearby_enemies": [{
                "id": 0,
                "position": [2.0, 1.0, 0.0],
                "health": 20.0,
                "distance": 2.0,
                "hostile": True,
            }],
        }
        action = agent.act(observation)
        self.assertEqual(action["attack"], True)
        self.assertEqual(action["move_z"], 1.0)

    def test_dead_enemy_is_ignored(self):
        agent = GoalDirectedAgent()
        observation = {
            "position": [0.0, 1.0, 0.0],
            "yaw": 0.0,
            "nearby_enemies": [{
                "id": 0,
                "position": [1.0, 1.0, 0.0],
                "health": 0.0,
                "distance": 1.0,
                "hostile": True,
            }],
        }
        self.assertEqual(agent.act(observation), {})


if __name__ == "__main__":
    unittest.main()
