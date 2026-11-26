"""Run with: python -m unittest agents.test_environment  (after building the headless binary)."""
import os
import unittest
from pathlib import Path

from agents.environment import NullscapeEnv, NullscapeError

_EXE = Path(__file__).resolve().parents[1] / "target" / "debug" / (
    "nullscape-headless" + (".exe" if os.name == "nt" else ""))


@unittest.skipUnless(_EXE.exists(), "build first: cargo build -p nullscape-core --bin nullscape-headless")
class EnvTests(unittest.TestCase):
    def test_partial_action_and_errors_do_not_desync(self):
        with NullscapeEnv() as env:
            self.assertEqual(env.step({0: {"jump": True}})[0]["tick"], 1)  # partial action
            with self.assertRaises(NullscapeError):
                env.step({0: {"jmp": True}})  # typo -> error, not a hang
            with self.assertRaises(NullscapeError):
                env.step({9: {}})  # unknown agent
            self.assertEqual(env.step({})[0]["tick"], 2)  # still in sync, tick not advanced by errors

    def test_reset(self):
        with NullscapeEnv() as env:
            for _ in range(5):
                env.step({0: {"move_z": -1.0}})
            obs = env.reset(seed=7)
            self.assertEqual(obs[0]["tick"], 0)
            self.assertEqual(env.step({})[0]["tick"], 1)


if __name__ == "__main__":
    unittest.main()
