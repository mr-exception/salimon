"""Controlled PR/API fixtures, never advertised as live GitHub or live AI."""
import copy
import hashlib
import io
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import zipfile
import urllib.error

import policy
import review
from invalidate import invalidate

ROOT = Path(__file__).resolve().parents[2]
BASELINES = ROOT / "client/tests/navigation-baselines"
MANIFEST = json.loads((BASELINES / "manifest.json").read_bytes())
HEAD, BASE, MERGE = "a" * 40, "b" * 40, "c" * 40
CRITERIA = {"AC-1": "Velocity is light green, never red"}


def response(verdict="ACCEPT", status="PASS"):
    return {"verdict": verdict, "summary": "Velocity matches the approved reference" if status == "PASS" else "Red velocity indicator",
            "criteria": [{"id": "AC-1", "status": status, "evidence": ["image:heading-velocity:actual"],
                          "finding": "Correct light green" if status == "PASS" else "Velocity is red",
                          "action": "Keep current behavior" if status == "PASS" else "Restore #b6f5a2 velocity stroke"}]}


def artifact(source=MERGE, omit=None):
    buf = io.BytesIO()
    trace = io.BytesIO()
    with zipfile.ZipFile(trace, "w") as z:
        z.writestr("trace.trace", "fixture, not a browser trace")
    prefix = "navigation-captures-fixture/comparisons/"
    rows = []
    with zipfile.ZipFile(buf, "w") as z:
        for frame in policy.FRAMES:
            data = (BASELINES / (frame + ".png")).read_bytes()
            sha = hashlib.sha256(data).hexdigest()
            rows.append({"id": frame, "pass": True, "changedPixels": 0, "rawChangedPixels": 0,
                         "totalPixels": 1024000, "expected": sha, "actual": sha})
            for suffix in (".png", "-expected.png", "-diff.png"):
                if frame + suffix != omit:
                    z.writestr(prefix + frame + suffix, data)
            z.writestr(prefix + frame + ".json", json.dumps({"source": source, "environment": MANIFEST["environment"],
                "state": {"scene": {"renderer": MANIFEST["renderer"]}}}))
        z.writestr(prefix + "summary.json", json.dumps({"source": source, "candidate": False, "results": rows}))
        z.writestr("navigation-captures-fixture/trace.zip", trace.getvalue())
        z.writestr("results.json", json.dumps({"stats": {"unexpected": 0, "expected": 1}, "suites": [{"specs": [{"title": "fixture navigation",
            "tests": [{"projectName": "fixture", "results": [{"status": "passed"}]}]}]}]}))
    return buf.getvalue()


class FakeGitHub:
    def __init__(self):
        self.pr = {"number": 999, "state": "open", "body": "Closes #998", "html_url": "https://github.com/mr-exception/salimon/pull/999",
            "head": {"sha": HEAD}, "base": {"sha": BASE, "ref": "main", "repo": {"full_name": review.REPO}}, "merge_commit_sha": MERGE}
        self.ci = {"id": 1, "head_sha": HEAD, "event": "pull_request", "path": ".github/workflows/navigation-client.yml",
            "status": "completed", "run_attempt": 1, "html_url": "https://github.com/mr-exception/salimon/actions/runs/1",
            "pull_requests": [{"number": 999}]}
        self.jobs = [{"name": j, "conclusion": "success"} for j in policy.REQUIRED_JOBS]
        self.comments, self.statuses, self.calls = [], [], []
        self.artifact = artifact()
        self.artifacts = [{"id": 10, "name": "navigation-evidence-" + MERGE, "expired": False, "size_in_bytes": len(self.artifact)}]
        self.reads = 0
        self.change_on_read = None
        self.reviews = []

    def request(self, path, method="GET", data=None, binary=False):
        self.calls.append((path, method))
        if path == "/pulls/999":
            self.reads += 1
            if self.change_on_read and self.reads >= self.change_on_read:
                self.pr["base"]["sha"] = "d" * 40
            return copy.deepcopy(self.pr)
        if path == "/issues/998":
            return {"body": "## Acceptance criteria\n- Velocity is light green, never red\n\n## Verification\nCapture"}
        if path == "/actions/runs/1":
            return self.ci
        if path == "/actions/artifacts/10/zip":
            return self.artifact
        if path == "/issues/999/comments" and method == "POST":
            self.comments.append({"body": data["body"], "user": {"login": "github-actions[bot]", "type": "Bot"}})
            return {"html_url": self.pr["html_url"] + "#issuecomment-1"}
        if path.startswith("/statuses/") and method == "POST":
            self.statuses.append((path.removeprefix("/statuses/"), data))
            return {}
        raise AssertionError((path, method))

    def pages(self, path, key=None):
        if path.startswith("/issues/"):
            return self.comments if path == "/issues/999/comments" else []
        if path.endswith("/files"):
            return [{"filename": "client/src/styles.css", "patch": "velocity fixture", "sha": HEAD, "status": "modified", "changes": 1}]
        if path.endswith("/reviews"):
            return self.reviews
        if path.endswith("/jobs?filter=latest"):
            return self.jobs
        if path.endswith("/artifacts"):
            return self.artifacts
        if path.startswith("/actions/workflows/"):
            return [self.ci]
        raise AssertionError(path)

    def contents(self, path, sha):
        if path == ".github/workflows/navigation-client.yml":
            return b"trusted workflow fixture"
        if path == "client/package.json":
            return b'{"scripts":{"build":"tsc -b && vite build","test":"fixture test","test:browser":"playwright test"}}'
        return (BASELINES / "manifest.json").read_bytes()


class PolicyTests(unittest.TestCase):
    def test_all_vetoes(self):
        binding = {"head": HEAD, "base": BASE, "merge": MERGE}
        for field in binding:
            changed = {**binding, field: "d" * 40}
            self.assertEqual(policy.decide(binding, changed, {j: "success" for j in policy.REQUIRED_JOBS}, True, response())[0], "BLOCKED")
        for missing in policy.REQUIRED_JOBS:
            self.assertEqual(policy.decide(binding, binding, {j: "success" for j in policy.REQUIRED_JOBS if j != missing}, True, response())[0], "BLOCKED")
        self.assertEqual(policy.decide(binding, binding, {"build": "failure", "baseline-approval": "skipped"}, True, response())[0], "REQUEST_CHANGES")
        self.assertEqual(policy.decide(binding, binding, {}, False, response())[0], "BLOCKED")
        self.assertEqual(policy.decide(binding, binding, {j: "success" for j in policy.REQUIRED_JOBS}, True, None)[0], "BLOCKED")

    def test_response_rejects_missing_or_invented_criteria_evidence(self):
        citations = {"image:heading-velocity:actual"}
        for mutation in (lambda r: r.update(criteria=[]),
                         lambda r: r.update(criteria=[None]),
                         lambda r: r["criteria"][0].update(id=[]),
                         lambda r: r["criteria"][0].update(id="AC-9"),
                         lambda r: r["criteria"][0].update(evidence=["made-up screenshot"]),
                         lambda r: r["criteria"][0].update(status="UNVERIFIED"),
                         lambda r: r["criteria"][0].update(action="")):
            candidate = response(); mutation(candidate)
            with self.assertRaises(ValueError):
                policy.validate_review(candidate, CRITERIA, citations)

    def test_issue_parser(self):
        body = "## Acceptance criteria\n- First\n  continued\n- Second\n\n## Verification\n- Not a criterion"
        self.assertEqual(list(review.criteria_from_issue(body)), ["AC-1", "AC-2"])
        with self.assertRaises(ValueError):
            review.criteria_from_issue("no criteria")

    def test_unsafe_duplicate_and_oversized_archives(self):
        for name in ("../capture.png", "/capture.png", "a\\capture.png"):
            buf = io.BytesIO()
            with zipfile.ZipFile(buf, "w") as z: z.writestr(name, b"x")
            with self.assertRaises(ValueError): review.read_archive(buf.getvalue())
        with patch.object(review, "MAX_FILE", 10):
            with self.assertRaises(ValueError): review.read_archive(artifact())

    def test_stale_and_missing_captures(self):
        with self.assertRaisesRegex(ValueError, "STALE_CAPTURE"):
            review.captures(review.read_archive(artifact(source=HEAD)), MANIFEST, MERGE)
        with self.assertRaises(KeyError):
            review.captures(review.read_archive(artifact(omit="heading-velocity.png")), MANIFEST, MERGE)
        wrong_manifest = copy.deepcopy(MANIFEST)
        wrong_manifest["images"]["heading-velocity.png"] = "0" * 64
        with self.assertRaisesRegex(ValueError, "UNAPPROVED_REFERENCE"):
            review.captures(review.read_archive(artifact()), wrong_manifest, MERGE)


class RuntimeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.old = Path.cwd(); os.chdir(self.temp.name)
        self.api = FakeGitHub()

    def tearDown(self):
        os.chdir(self.old); self.temp.cleanup()

    def execute(self, result=None):
        with patch.object(review, "ai_review", return_value=(result or response(), {"provider": "TEST DOUBLE", "attempts": 1})) as provider:
            report = review.run(self.api, 999, 1)
        return report, provider.call_count

    def test_controlled_wrong_repair_new_sha_and_idempotence(self):
        self.api.jobs = [{"name": "build", "conclusion": "failure"}, {"name": "baseline-approval", "conclusion": "skipped"}]
        wrong, _ = self.execute(response("REQUEST_CHANGES", "FAIL"))
        self.assertEqual(wrong["verdict"], "REQUEST_CHANGES")
        self.assertIn("Restore #b6f5a2", self.api.comments[-1]["body"])
        old_head = self.api.pr["head"]["sha"]
        self.api.pr["head"]["sha"] = "e" * 40
        invalidate(self.api, self.api.pr)
        self.assertEqual(self.api.statuses[-1][1]["state"], "pending")
        stale, calls = self.execute()
        self.assertEqual((stale["verdict"], calls), ("BLOCKED", 0))
        self.api.ci.update(head_sha="e" * 40, run_attempt=2)
        self.api.jobs = [{"name": j, "conclusion": "success"} for j in policy.REQUIRED_JOBS]
        fixed, _ = self.execute()
        self.assertEqual(fixed["verdict"], "ACCEPT")
        self.assertNotEqual(old_head, fixed["binding"]["head"])
        comments = len(self.api.comments)
        duplicate, calls = self.execute()
        self.assertEqual((duplicate["verdict"], calls, len(self.api.comments)), ("ACCEPT", 0, comments))
        self.assertEqual(self.api.statuses[-1][1]["state"], "success")
        self.assertFalse(any("merge" in p or "PATCH" == m for p, m in self.api.calls))

    def test_ai_accept_cannot_override_failure(self):
        self.api.jobs[0]["conclusion"] = "failure"
        report, _ = self.execute()
        self.assertEqual(report["verdict"], "REQUEST_CHANGES")

    def test_unavailable_reviewer(self):
        with patch.dict(os.environ, {"OPENAI_API_KEY": ""}), patch.object(review, "ai_review", side_effect=ValueError("REVIEWER_UNAVAILABLE")):
            report = review.run(self.api, 999, 1)
        self.assertEqual(report["verdict"], "BLOCKED")

        self.assertFalse(report["provider_attempted"])

    def test_missing_capture_and_expired_artifact(self):
        for data in (artifact(omit="heading-velocity.png"), None):
            self.api = FakeGitHub()
            if data: self.api.artifact = data
            else: self.api.artifacts[0]["expired"] = True
            report, calls = self.execute()
            self.assertEqual((report["verdict"], calls), ("BLOCKED", 0))

    def test_mid_review_and_pre_publication_parent_changes(self):
        for change in (2, 3):
            self.api = FakeGitHub(); self.api.change_on_read = change
            report, _ = self.execute()
            self.assertEqual(report["verdict"], "BLOCKED")
            self.assertEqual(self.api.statuses[-1][1]["state"], "failure")

    def test_cycle_limit_and_spoofed_history(self):
        for _ in range(3):
            self.api.comments.append({"body": review.MARKER + '\n```json\n' + json.dumps({"schema_version": 1, "provider_attempted": True}) + '\n```',
                                      "user": {"login": "github-actions[bot]", "type": "Bot"}})
        report, calls = self.execute()
        self.assertEqual((report["verdict"], calls), ("BLOCKED", 0))
        self.assertIn("CYCLE_LIMIT", report["error"])
        for c in self.api.comments: c["user"] = {"login": "attacker", "type": "User"}
        report, calls = self.execute()
        self.assertEqual((report["verdict"], calls), ("ACCEPT", 1))

    def test_current_report_is_machine_readable(self):
        report, _ = self.execute()
        self.assertEqual(json.loads(Path("qa-output/report.json").read_text()), report)
        self.assertEqual(report["evidence"]["tested_sha"], MERGE)
        self.assertEqual(len(report["evidence"]["sha256"]), 34)

    def test_idempotence_revalidates_expired_capture(self):
        self.assertEqual(self.execute()[0]["verdict"], "ACCEPT")
        self.api.artifacts[0]["expired"] = True
        report, calls = self.execute()
        self.assertEqual((report["verdict"], calls), ("BLOCKED", 0))

    def test_blocking_human_review(self):
        self.api.reviews = [{"id": 1, "state": "CHANGES_REQUESTED", "user": {"login": "owner"}}]
        report, calls = self.execute()
        self.assertEqual((report["verdict"], calls), ("BLOCKED", 0))
        self.api.reviews.append({"id": 2, "state": "COMMENTED", "user": {"login": "owner"}})
        self.assertEqual(review.blocking_reviews(self.api.reviews), ["owner"])
        self.api.reviews.append({"id": 3, "state": "APPROVED", "user": {"login": "owner"}})
        self.assertEqual(review.blocking_reviews(self.api.reviews), [])

    def test_ci_workflow_change_needs_human_review(self):
        original = self.api.contents
        self.api.contents = lambda path, sha: b"no-op workflow" if path.endswith("navigation-client.yml") and sha != "main" else original(path, sha)
        report, calls = self.execute()
        self.assertEqual((report["verdict"], calls), ("BLOCKED", 0))
        self.assertIn("CI_DEFINITION_CHANGED", report["error"])

    def test_malformed_provider_response_blocks(self):
        with patch.object(review, "ai_review", side_effect=AttributeError("malformed provider")):
            report = review.run(self.api, 999, 1)
        self.assertEqual(report["verdict"], "BLOCKED")

    def test_independent_baseline_approval_cannot_be_bypassed(self):
        api = FakeGitHub()
        original = api.contents
        api.contents = lambda path, sha: b"old manifest" if sha == BASE else original(path, sha)
        manifest = original("manifest", HEAD)
        checksum = hashlib.sha256(manifest).hexdigest()
        for comments in ([], [{"body": "Approve navigation baselines: " + checksum, "user": {"type": "Bot"}, "author_association": "OWNER"}]):
            with self.assertRaisesRegex(ValueError, "BASELINE_APPROVAL_REQUIRED"):
                review.baseline_approval(api, api.pr, manifest, comments)
        approved = [{"body": "Approve navigation baselines: " + checksum, "user": {"type": "User"},
                     "author_association": "OWNER", "html_url": "human-fixture"}]
        self.assertEqual(review.baseline_approval(api, api.pr, manifest, approved)["approval"], "explicit independent human")

    def test_noop_mandatory_script_is_blocked(self):
        original = self.api.contents
        self.api.contents = lambda path, sha: b'{"scripts":{"build":"true"}}' if path == "client/package.json" and sha == HEAD else original(path, sha)
        report, calls = self.execute()
        self.assertEqual((report["verdict"], calls), ("BLOCKED", 0))

    def test_stacked_parent_cannot_self_approve_new_manifest(self):
        original = self.api.contents
        self.api.contents = lambda path, sha: b"different trusted manifest" if path.endswith("manifest.json") and sha == "main" else original(path, sha)
        with self.assertRaisesRegex(ValueError, "BASELINE_APPROVAL_REQUIRED"):
            review.baseline_approval(self.api, self.api.pr, original("manifest", HEAD), [])


class ProviderTransportTests(unittest.TestCase):
    def test_structured_multimodal_request_and_transient_retry(self):
        payload = {"status": "completed", "id": "fixture-response", "output": [{"content": [
            {"type": "output_text", "text": json.dumps(response())}]}]}
        class HTTPResponse(io.BytesIO):
            def __enter__(self): return self
            def __exit__(self, *args): self.close()
        failure = urllib.error.HTTPError("https://api.openai.com/v1/responses", 429, "fixture", {}, None)
        with patch.dict(os.environ, {"OPENAI_API_KEY": "fixture-only-key", "SALIMON_REVIEW_MODEL": "gpt-4.1"}), \
             patch.object(review.urllib.request, "urlopen", side_effect=[failure, HTTPResponse(json.dumps(payload).encode())]) as http, \
             patch.object(review.time, "sleep"):
            result, provider = review.ai_review({"issue": "fixture"}, [("image:heading-velocity:actual", b"fixture-image")], CRITERIA, {"image:heading-velocity:actual"})
        self.assertEqual((result["verdict"], provider["attempts"], http.call_count), ("ACCEPT", 2, 2))
        sent = json.loads(http.call_args.args[0].data)
        self.assertFalse(sent["store"])
        self.assertTrue(sent["text"]["format"]["strict"])
        self.assertEqual(sent["input"][0]["content"][-1]["type"], "input_image")
        self.assertNotIn("tools", sent)

    def test_permanent_and_retry_exhaustion_are_bounded(self):
        for code, expected in ((401, 1), (429, 2), (503, 2)):
            failure = urllib.error.HTTPError("https://api.openai.com/v1/responses", code, "fixture", {}, None)
            with patch.dict(os.environ, {"OPENAI_API_KEY": "fixture-only-key"}), \
                 patch.object(review.urllib.request, "urlopen", side_effect=failure) as http, patch.object(review.time, "sleep"):
                with self.assertRaisesRegex(ValueError, "REVIEWER_UNAVAILABLE"):
                    review.ai_review({}, [], CRITERIA, {"issue"})
                self.assertEqual(http.call_count, expected)


if __name__ == "__main__":
    unittest.main()
