"""Declarative native E2E runner for Salimon (Python standard library only)."""

import json
import queue
import re
import subprocess
import threading
import time
from pathlib import Path

from e2e_artifacts import Artifacts


PROTOCOL = 1
SETUPS = {"fragment-pile", "resource-approach", "landed-earth", "cockpit-earth", "orbit-earth", "orbit-moon", "open-space", "eva-approach"}
OPS = {"inspect", "step", "look", "aim_fragment", "key", "interact", "landing", "thruster"}
COMPARISONS = {"equals", "not_equals", "gt", "gte", "lt", "lte", "approx", "exists", "contains"}


class ScenarioError(Exception):
    pass


def positive_int(value, name):
    if type(value) is not int or value <= 0:
        raise ScenarioError(f"{name} must be a positive integer")
    return value


def parse_scenario(path):
    try:
        data = json.loads(Path(path).read_text(encoding="utf-8"),
                          parse_constant=lambda value: (_ for _ in ()).throw(ValueError(f"nonfinite number: {value}")))
    except (OSError, ValueError) as exc:
        raise ScenarioError(f"cannot read scenario: {exc}") from exc
    if not isinstance(data, dict) or set(data) - {"setup", "timeout_ms", "steps"}:
        raise ScenarioError("scenario must be an object with setup, timeout_ms, and steps")
    setup = data.get("setup", {})
    if not isinstance(setup, dict) or set(setup) - {"scenario", "seed", "step_ms"}:
        raise ScenarioError("setup must contain only scenario, seed, and step_ms")
    name = setup.get("scenario", "landed-earth")
    seed = setup.get("seed", 0)
    step_ms = setup.get("step_ms", 16)
    if name not in SETUPS or type(seed) is not int or not 0 <= seed < 2**64 or type(step_ms) is not int or not 1 <= step_ms <= 100:
        raise ScenarioError("invalid setup scenario, seed, or step_ms")
    timeout = positive_int(data.get("timeout_ms", 60000), "timeout_ms")
    steps = data.get("steps")
    if not isinstance(steps, list) or not steps:
        raise ScenarioError("steps must be a nonempty array")
    for index, step in enumerate(steps, 1):
        if not isinstance(step, dict) or len({"action", "wait", "assert", "screenshot"} & step.keys()) != 1:
            raise ScenarioError(f"step {index}: specify exactly one action, wait, assert, or screenshot")
        kind = next(iter({"action", "wait", "assert", "screenshot"} & step.keys()))
        permitted = {kind, "timeout_ms"} | ({"frames"} if kind == "wait" else set())
        if set(step) - permitted:
            raise ScenarioError(f"step {index}: unknown fields")
        if "timeout_ms" in step:
            positive_int(step["timeout_ms"], f"step {index} timeout_ms")
        if kind == "screenshot":
            if not isinstance(step[kind], str) or not re.fullmatch(r"[A-Za-z0-9_-]{1,80}", step[kind]):
                raise ScenarioError(f"step {index}: screenshot needs a safe checkpoint name")
        elif kind == "action":
            action = step[kind]
            if not isinstance(action, dict) or action.get("op") not in OPS or "protocol" in action or "id" in action:
                raise ScenarioError(f"step {index}: invalid action")
        else:
            condition = step[kind]
            if not isinstance(condition, dict) or not isinstance(condition.get("path"), str) or not condition["path"]:
                raise ScenarioError(f"step {index}: condition needs a path")
            comparisons = COMPARISONS & condition.keys()
            if len(comparisons) != 1 or set(condition) - {"path", "tolerance"} - COMPARISONS:
                raise ScenarioError(f"step {index}: condition needs exactly one comparison")
            if "exists" in comparisons and type(condition["exists"]) is not bool:
                raise ScenarioError(f"step {index}: exists must be boolean")
            if "tolerance" in condition and ("approx" not in comparisons or type(condition["tolerance"]) not in (int, float) or condition["tolerance"] < 0):
                raise ScenarioError(f"step {index}: invalid tolerance")
            if kind == "wait":
                frames = positive_int(step.get("frames", 1), f"step {index} frames")
                if frames > 600:
                    raise ScenarioError(f"step {index}: frames must be <= 600")
    return {"setup": {"scenario": name, "seed": seed, "step_ms": step_ms}, "timeout_ms": timeout, "steps": steps}


MISSING = object()


def inspection_value(state, path):
    current = state
    for part in path.split("."):
        try:
            current = current[int(part)] if isinstance(current, list) else current[part]
        except (KeyError, IndexError, ValueError, TypeError):
            return MISSING
    return current


def matches(state, condition):
    current = inspection_value(state, condition["path"])
    key = next(iter(COMPARISONS & condition.keys()))
    expected = condition[key]
    if key == "exists":
        return (current is not MISSING) == expected
    if current is MISSING:
        raise ScenarioError(f"missing inspection path: {condition['path']}")
    if key == "contains":
        if not isinstance(current, (list, dict, str)):
            raise ScenarioError("contains requires an array, object, or string")
        try:
            return expected in current
        except TypeError as exc:
            raise ScenarioError(f"invalid contains comparison: {exc}") from exc
    if key == "equals":
        return current == expected
    if key == "not_equals":
        return current != expected
    try:
        if type(current) not in (int, float) or type(expected) not in (int, float):
            raise TypeError("numeric comparison requires numbers")
        return {"gt": lambda: current > expected, "gte": lambda: current >= expected,
                "lt": lambda: current < expected, "lte": lambda: current <= expected,
                "approx": lambda: abs(current - expected) <= condition.get("tolerance", 1e-6)}[key]()
    except (TypeError, OverflowError) as exc:
        raise ScenarioError(f"invalid comparison at {condition['path']}: {exc}") from exc


class Game:
    def __init__(self, command, setup, deadline, artifacts=None):
        self.command = command
        self.artifacts = artifacts
        self.setup = setup
        self.deadline = deadline
        self.responses = queue.Queue()
        self.stderr = []
        self.process = None
        self.next_id = 0

    def __enter__(self):
        self.process = subprocess.Popen(
            self.command + ["--e2e", "--scenario", self.setup["scenario"],
                            "--seed", str(self.setup["seed"]), "--step-ms", str(self.setup["step_ms"])],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            text=True, bufsize=1,
        )
        self.readers = [threading.Thread(target=self._read_stdout, daemon=True),
                        threading.Thread(target=self._read_stderr, daemon=True)]
        for reader in self.readers:
            reader.start()
        try:
            first = self._receive("ready signal")
            expected = f"SALIMON_E2E_READY scenario={self.setup['scenario']} seed={self.setup['seed']} step_ms={self.setup['step_ms']}"
            if first != expected:
                raise ScenarioError(f"unexpected ready signal: {first[:200]}")
            ready = self._json(self._receive("ready event"))
            if ready != {"protocol": PROTOCOL, "event": "ready", **self.setup, "timeout_ms": 5000}:
                # The event calls the scenario field 'scenario', just like setup.
                raise ScenarioError(f"invalid ready event: {ready}")
            return self
        except BaseException:
            self.__exit__(None, None, None)
            raise

    def _read_stdout(self):
        try:
            for line in self.process.stdout:
                if self.artifacts:
                    self.artifacts.log("stdout.log", line.rstrip())
                self.responses.put(line.strip())
        finally:
            self.responses.put(None)

    def _read_stderr(self):
        for line in self.process.stderr:
            if self.artifacts:
                self.artifacts.log("stderr.log", line.rstrip())
            self.stderr.append(line.rstrip())
            if len(self.stderr) > 30:
                self.stderr.pop(0)

    def _receive(self, label, deadline=None):
        remaining = (min(self.deadline, deadline) if deadline else self.deadline) - time.monotonic()
        if remaining <= 0:
            raise ScenarioError(f"timeout waiting for {label}")
        try:
            line = self.responses.get(timeout=remaining)
        except queue.Empty as exc:
            raise ScenarioError(f"timeout waiting for {label}") from exc
        if line is None:
            raise ScenarioError(f"game exited before {label} (exit {self.process.poll()}): {'; '.join(self.stderr[-5:])}")
        return line

    @staticmethod
    def _json(line):
        try:
            value = json.loads(line)
        except ValueError as exc:
            raise ScenarioError(f"non-JSON protocol line: {line[:200]}") from exc
        if not isinstance(value, dict):
            raise ScenarioError("protocol response must be an object")
        return value

    def request(self, action, deadline=None):
        self.next_id += 1
        request = {"protocol": PROTOCOL, "id": self.next_id, **action}
        if self.artifacts:
            self.artifacts.log("protocol.jsonl", json.dumps({"direction": "request", "message": request}))
        try:
            self.process.stdin.write(json.dumps(request, allow_nan=False) + "\n")
            self.process.stdin.flush()
        except (OSError, ValueError, TypeError) as exc:
            raise ScenarioError(f"cannot send command: {exc}") from exc
        response = self._json(self._receive(f"response to {action['op']}", deadline))
        if self.artifacts:
            self.artifacts.log("protocol.jsonl", json.dumps({"direction": "response", "message": response}))
        if response.get("protocol") != PROTOCOL or response.get("id") != self.next_id or type(response.get("ok")) is not bool:
            raise ScenarioError(f"invalid protocol response: {response}")
        if not response["ok"]:
            raise ScenarioError(f"game rejected {action['op']}: {response.get('error')}")
        if "result" not in response:
            raise ScenarioError("successful response missing result")
        return response["result"]

    def __exit__(self, *_):
        if self.process is None:
            return
        if self.process.poll() is None:
            self.process.terminate()
        try:
            self.process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        for reader in self.readers:
            reader.join(timeout=1)
        self.process.stdin.close()
        self.process.stdout.close()
        self.process.stderr.close()


def run(path, command, artifact_root="artifacts/e2e", screenshot_command=None):
    start = time.monotonic()
    result = {"schema_version": 1, "scenario": str(path), "status": "failed",
              "steps_completed": 0, "steps": [], "command": command}
    artifacts = None
    game = None
    try:
        # Even parse/startup failures have a result and diagnostic directory.
        artifacts = Artifacts({"source": str(path)}, artifact_root, screenshot_command)
        artifacts.open_logs()
        result["artifact_directory"] = str(artifacts.directory)
        scenario = parse_scenario(path)
        artifacts.write_json("scenario.json", scenario)
        result["setup"] = scenario["setup"]
        deadline = start + scenario["timeout_ms"] / 1000
        game = Game(command, scenario["setup"], deadline, artifacts)
        with game:
            try:
                for index, step in enumerate(scenario["steps"], 1):
                    step_start = time.monotonic()
                    record = {"index": index, "step": step, "status": "failed",
                              "started_ms": round((step_start - start) * 1000)}
                    result["steps"].append(record)
                    step_deadline = min(deadline, step_start + step.get("timeout_ms", 5000) / 1000)
                    try:
                        if "action" in step:
                            record["response"] = game.request(step["action"], step_deadline)
                            record["state"] = game.request({"op": "inspect"}, step_deadline)
                        elif "screenshot" in step:
                            record["state"] = game.request({"op": "inspect"}, step_deadline)
                            record["screenshot"] = artifacts.screenshot(
                                f"step-{index:03d}-{step['screenshot']}", max(0.001, step_deadline - time.monotonic()))
                            if record["screenshot"]["status"] != "captured":
                                raise ScenarioError(f"screenshot {step['screenshot']}: {record['screenshot']}")
                        else:
                            condition = step.get("assert", step.get("wait"))
                            attempts = 0
                            while True:
                                state = game.request({"op": "inspect"}, step_deadline)
                                record["state"] = state
                                attempts += 1
                                actual = inspection_value(state, condition["path"])
                                assertion = {"condition": condition, "attempts": attempts,
                                             "actual_present": actual is not MISSING, "passed": False}
                                if actual is not MISSING:
                                    assertion["actual"] = actual
                                record["assertion"] = assertion
                                assertion["passed"] = matches(state, condition)
                                if assertion["passed"]:
                                    break
                                if "assert" in step:
                                    raise ScenarioError(f"assertion failed: {condition}")
                                if time.monotonic() >= step_deadline:
                                    raise ScenarioError(f"wait timed out: {condition}")
                                game.request({"op": "step", "frames": step.get("frames", 1)}, step_deadline)
                        record["status"] = "passed"
                        result["steps_completed"] = index
                    except ScenarioError as exc:
                        record["error"] = str(exc)
                        raise ScenarioError(f"step {index}: {exc}") from exc
                    finally:
                        record["duration_ms"] = round((time.monotonic() - step_start) * 1000)
                        artifacts.write_json(f"step-{index:03d}.json", record)
            except (ScenarioError, OSError, subprocess.SubprocessError):
                # Capture before process termination. Avoid a new protocol request after
                # a timeout: the outstanding response would desynchronize the channel.
                result["failure_screenshot"] = artifacts.screenshot("failure")
                last_state = next((record["state"] for record in reversed(result["steps"])
                                   if "state" in record), None)
                if last_state is not None:
                    result["failure_state_file"] = artifacts.write_json("failure-state.json", last_state)
                raise
        result["status"] = "passed"
    except (ScenarioError, OSError, subprocess.SubprocessError) as exc:
        result["error"] = str(exc)
        if artifacts and "failure_screenshot" not in result:
            result["failure_screenshot"] = {"status": "unavailable", "reason": "game did not become ready"}
    finally:
        result["duration_ms"] = round((time.monotonic() - start) * 1000)
        if game and game.process:
            result["game_exit_code"] = game.process.poll()
        if artifacts:
            artifacts.close()
            try:
                result["result_file"] = str(artifacts.directory / "result.json")
                artifacts.write_json("result.json", result)
            except OSError as exc:
                result["status"] = "failed"
                result["artifact_error"] = str(exc)
    return result
