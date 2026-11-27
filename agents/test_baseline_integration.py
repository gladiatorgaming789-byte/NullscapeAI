import os
import unittest
from pathlib import Path

from agents.environment import NullscapeEnv
from agents.policy import GoalDirectedAgent

_EXE = Path(__file__).resolve().parents[1] / "target" / "debug" / (
    "nullscape-headless" + (".exe" if os.name == "nt" else "")
)


@unittest.skipUnless(_EXE.exists(), "build first: cargo build -p nullscape-core --bin nullscape-headless")
class BaselineIntegrationTests(unittest.TestCase):
    def test_ai_can_control_real_simulation(self):
        policy = GoalDirectedAgent()

        with NullscapeEnv() as env:
            observations = env.reset(seed=1337)
            start = next(o for o in observations if o["agent_id"] == 0)["position"]

            for _ in range(30):
                agent_obs = next(o for o in observations if o["agent_id"] == 0)
                observations = env.step({0: policy.act(agent_obs)})

            agent_obs = next(o for o in observations if o["agent_id"] == 0)
            self.assertEqual(agent_obs["tick"], 30)
            self.assertNotEqual(agent_obs["position"], start)


if __name__ == "__main__":
    unittest.main()
