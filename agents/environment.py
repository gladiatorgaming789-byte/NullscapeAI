"""JSONL client for the headless Nullscape environment."""
from __future__ import annotations
import json
import subprocess
from pathlib import Path

class NullscapeEnv:
    def __init__(self, executable: str = "target/debug/nullscape-headless"):
        self.process = subprocess.Popen(
            [executable],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            cwd=Path(__file__).resolve().parents[1],
            bufsize=1,
        )

    def step(self, actions: dict[int, dict]) -> list[dict]:
        if not self.process.stdin or not self.process.stdout:
            raise RuntimeError("environment process is not running")
        payload = {str(k): v for k, v in actions.items()}
        self.process.stdin.write(json.dumps(payload) + "\n")
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        if not line:
            raise RuntimeError("environment exited unexpectedly")
        return json.loads(line)

    def close(self) -> None:
        if self.process.stdin:
            self.process.stdin.close()
        self.process.terminate()
