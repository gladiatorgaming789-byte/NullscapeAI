"""JSONL client for the headless Nullscape environment.

Protocol (see ``crates/nullscape-core/src/protocol.rs``): every request line
gets exactly one response line. A failed request raises :class:`NullscapeError`
and leaves the simulation untouched, so the client can keep using the same
environment afterwards.
"""
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path


class NullscapeError(RuntimeError):
    """The environment rejected a request (bad JSON, unknown agent, bad field...)."""


class NullscapeEnv:
    def __init__(self, executable: str | os.PathLike | None = None):
        root = Path(__file__).resolve().parents[1]
        if executable is None:
            name = "nullscape-headless" + (".exe" if os.name == "nt" else "")
            executable = root / "target" / "debug" / name
        self.process = subprocess.Popen(
            [str(executable)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
            cwd=root,
            bufsize=1,
        )

    def _request(self, payload: dict) -> list[dict]:
        if not self.process.stdin or not self.process.stdout:
            raise RuntimeError("environment process is not running")
        if self.process.poll() is not None:
            raise RuntimeError("environment exited unexpectedly")
        self.process.stdin.write(json.dumps(payload) + "\n")
        self.process.stdin.flush()
        line = self.process.stdout.readline()
        if not line:
            raise RuntimeError("environment exited unexpectedly")
        data = json.loads(line)
        if isinstance(data, dict) and "error" in data:
            raise NullscapeError(data["error"])
        return data

    def step(self, actions: dict[int, dict]) -> list[dict]:
        """Advance one tick. Action dicts may omit any field (``{"jump": True}``)."""
        return self._request({str(k): v for k, v in actions.items()})

    def reset(self, seed: int | None = None) -> list[dict]:
        """Restart the simulation (optionally with a new terrain seed); returns tick-0 observations."""
        payload: dict = {"cmd": "reset"}
        if seed is not None:
            payload["seed"] = seed
        return self._request(payload)

    def close(self) -> None:
        if self.process.stdin:
            self.process.stdin.close()
        self.process.terminate()
        try:
            self.process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        if self.process.stdout:
            self.process.stdout.close()

    def __enter__(self) -> "NullscapeEnv":
        return self

    def __exit__(self, *exc_info) -> None:
        self.close()
