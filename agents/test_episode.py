import unittest

from agents.episode import NullscapeEpisode


class FakeEnv:
    def __init__(self):
        self.reset_calls = 0
        self.step_calls = []

    def reset(self, seed=None):
        self.reset_calls += 1
        return [
            {"agent_id": 0, "health": 100.0, "nearby_enemies": []},
            {"agent_id": 1, "health": 100.0, "nearby_enemies": []},
        ]

    def step(self, actions):
        self.step_calls.append(actions)
        return [
            {"agent_id": 0, "health": 100.0, "nearby_enemies": []},
            {"agent_id": 1, "health": 100.0, "nearby_enemies": []},
        ]

    def close(self):
        pass


class EpisodeTests(unittest.TestCase):
    def test_reset_selects_requested_agent(self):
        episode = NullscapeEpisode(env=FakeEnv(), agent_id=1)
        observation = episode.reset(seed=42)
        self.assertEqual(observation["agent_id"], 1)
        self.assertFalse(episode.done)

    def test_step_forwards_only_selected_agent(self):
        env = FakeEnv()
        episode = NullscapeEpisode(env=env, agent_id=1)
        episode.reset()
        observation, reward, done = episode.step({"move_z": 1.0})
        self.assertEqual(env.step_calls, [{1: {"move_z": 1.0}}])
        self.assertEqual(observation["agent_id"], 1)
        self.assertLess(reward, 0.0)
        self.assertFalse(done)

    def test_step_after_done_is_rejected(self):
        env = FakeEnv()
        episode = NullscapeEpisode(env=env)
        episode.reset()
        episode.done = True
        with self.assertRaises(RuntimeError):
            episode.step({})


if __name__ == "__main__":
    unittest.main()
