"""Replay real, independently reviewed browser captures against a fake PR API.

Only GitHub transport is simulated; actual session verdicts use the production handoff verifier.
Verdicts must come from a distinct reviewer inspecting the supplied originals.
This is a controlled fixture, never a live GitHub acceptance claim.
"""
import argparse
import io
import json
import os
from pathlib import Path
import tempfile
import zipfile

import review
from invalidate import invalidate
from policy import REQUIRED_JOBS
from handoff import make_handoff, comment
from test_review import FakeGitHub


def archive_directory(root):
    data = io.BytesIO()
    with zipfile.ZipFile(data, "w", zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(root.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(root))
    return data.getvalue()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("wrong", "fixed", "wrong_review", "fixed_review", "output"):
        parser.add_argument("--" + name.replace("_", "-"), type=Path, required=True)
    args = parser.parse_args()
    for name in ("wrong", "fixed", "wrong_review", "fixed_review", "output"):
        setattr(args, name, getattr(args, name).resolve())
    output = args.output.resolve(); output.mkdir(parents=True, exist_ok=True)
    sources = {}
    for condition in ("wrong", "fixed"):
        directory = getattr(args, condition).resolve()
        summaries = list(directory.glob("navigation-captures-*/comparisons/summary.json"))
        if len(summaries) != 1:
            raise ValueError("Expected one real browser summary per condition")
        sources[condition] = json.loads(summaries[0].read_text())["source"]
    if sources["wrong"] == sources["fixed"]:
        raise ValueError("Repair must be captured at a different actual source SHA")
    api, transitions, reports = FakeGitHub(), [], {}
    def bind(condition):
        source = sources[condition]
        api.pr["head"]["sha"] = source; api.pr["merge_commit_sha"] = source
        api.ci["head_sha"] = source; api.ci["run_attempt"] = 1 if condition == "wrong" else 2
        api.artifact = archive_directory(getattr(args, condition).resolve())
        api.artifacts = [{"id": 10, "name": "navigation-evidence-" + source, "expired": False, "size_in_bytes": len(api.artifact)}]
        api.jobs = [{"name": name, "conclusion": "success" if condition == "fixed" else ("failure" if name == "build" else "skipped")}
                    for name in REQUIRED_JOBS]
    def submit_review(condition):
        record = json.loads(getattr(args, condition + "_review").read_text())
        packet = json.loads(Path("qa-output/handoff-request.json").read_text())
        if (record["source"] != sources[condition] or not record["reviewer"].startswith("/root/")
                or record["capture_sha256"] != packet["evidence"]["sha256"]):
            raise ValueError("Actual reviewer must bind these exact source/capture bytes")
        handoff = make_handoff(packet, record["review"], record["reviewer"], "/root")
        api.comments.append({"body": comment(handoff), "user": {"type": "User", "login": "fixture-submitter"},
                             "author_association": "OWNER", "html_url": "fixture-session-handoff-" + condition})
    old = Path.cwd()
    with tempfile.TemporaryDirectory() as scratch:
        os.chdir(scratch)
        try:
            bind("wrong")
            reports["awaiting_wrong_review"] = review.run(api, 999, 1)
            submit_review("wrong")
            reports["wrong"] = review.run(api, 999, 1)
            transitions.append({"phase": "wrong indicator", "head": sources["wrong"], "verdict": reports["wrong"]["verdict"], "status": api.statuses[-1][1]["state"]})
            api.pr["head"]["sha"] = sources["fixed"]; api.pr["merge_commit_sha"] = sources["fixed"]
            invalidate(api, api.pr)
            transitions.append({"phase": "repair before new CI", "head": sources["fixed"], "status": api.statuses[-1][1]["state"]})
            reports["stale"] = review.run(api, 999, 1)
            transitions.append({"phase": "old CI cannot approve repair", "verdict": reports["stale"]["verdict"], "status": api.statuses[-1][1]["state"]})
            bind("fixed")
            reports["awaiting_fixed_review"] = review.run(api, 999, 1)
            submit_review("fixed")
            reports["fixed"] = review.run(api, 999, 1)
            transitions.append({"phase": "new source tests and capture", "head": sources["fixed"], "verdict": reports["fixed"]["verdict"], "status": api.statuses[-1][1]["state"]})
            api.pr["base"]["sha"] = "d" * 40
            invalidate(api, api.pr)
            transitions.append({"phase": "parent advances", "status": api.statuses[-1][1]["state"]})
        finally:
            os.chdir(old)
    expected = ["REQUEST_CHANGES", "BLOCKED", "ACCEPT"]
    if [reports[key]["verdict"] for key in ("wrong", "stale", "fixed")] != expected:
        raise ValueError("Controlled review demo failed: " + json.dumps(transitions))
    record = {"schema_version": 1, "scope": "real captures and distinct AI image review; simulated PR/GitHub status transport",
              "sources": sources, "transitions": transitions, "reports": reports,
              "comments": api.comments, "statuses": api.statuses, "ai_execution": "actual Codex session reviewer; no external AI API", "automatic_merge": False}
    (output / "demo.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(transitions, indent=2))


if __name__ == "__main__":
    main()
