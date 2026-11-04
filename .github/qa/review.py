"""Trusted, bounded PR evidence reader. Never checks out or executes PR code.

Invoked by independent-qa.yml from the default branch, not from the PR.
"""
import base64
import io
import json
import os
import re
import urllib.error
import urllib.request
import zipfile
from pathlib import Path, PurePosixPath

from policy import (CONTEXT, FRAMES, MAX_CYCLES, REQUIRED_JOBS,
                    SCHEMA, decide, digest, identity, validate_review)

REPO = "mr-exception/salimon"
MARKER = "<!-- salimon-independent-qa:v1 -->"
HANDOFF_MARKER = "<!-- salimon-codex-review:v1 -->"
MAX_ZIP = 128 * 1024 * 1024
MAX_FILE = 32 * 1024 * 1024


class API:
    def __init__(self, token):
        self.token = token
        self.calls = 0

    def request(self, path, method="GET", data=None, binary=False):
        self.calls += 1
        if self.calls > 100:
            raise ValueError("GITHUB_RESOURCE_BUDGET: 100 calls exhausted")
        request = urllib.request.Request("https://api.github.com/repos/" + REPO + path,
            data=None if data is None else json.dumps(data).encode(), method=method,
            headers={"Authorization": "Bearer " + self.token, "Accept": "application/vnd.github+json",
                     "Content-Type": "application/json", "X-GitHub-Api-Version": "2022-11-28"})
        # urllib strips Authorization on cross-host redirects only with our handler.
        class SafeRedirect(urllib.request.HTTPRedirectHandler):
            def redirect_request(self, req, fp, code, msg, headers, newurl):
                from urllib.parse import urlparse
                host = urlparse(newurl).hostname or ""
                if not newurl.startswith("https://") or not (host.endswith(".blob.core.windows.net") or host.endswith(".githubusercontent.com")):
                    raise ValueError("Unexpected artifact redirect")
                redirected = super().redirect_request(req, fp, code, msg, headers, newurl)
                redirected.remove_header("Authorization")
                return redirected
        try:
            with urllib.request.build_opener(SafeRedirect()).open(request, timeout=30) as response:
                cap = MAX_ZIP if binary else 8 * 1024 * 1024
                result = response.read(cap + 1)
                if len(result) > cap:
                    raise ValueError("GITHUB_RESOURCE_BUDGET: response too large")
                return result if binary else json.loads(result)
        except urllib.error.HTTPError as exc:
            raise ValueError(f"GitHub {method} {path.split('?')[0]}: HTTP {exc.code}") from None

    def pages(self, path, key=None):
        result = []
        for page in range(1, 6):
            batch = self.request(f"{path}{'&' if '?' in path else '?'}per_page=100&page={page}")
            batch = batch[key] if key else batch
            result.extend(batch)
            if len(batch) < 100:
                return result
        raise ValueError("GITHUB_RESOURCE_BUDGET: pagination exceeds 500 items")

    def contents(self, path, sha):
        result = self.request(f"/contents/{path}?ref={sha}")
        if result.get("encoding") != "base64":
            raise ValueError("Missing inline source content")
        return base64.b64decode(result["content"])


def read_archive(data):
    archive = zipfile.ZipFile(io.BytesIO(data))
    entries = archive.infolist()
    if len(entries) > 2000 or sum(e.file_size for e in entries) > 256 * 1024 * 1024:
        raise ValueError("ARTIFACT_RESOURCE_BUDGET")
    names = [e.filename for e in entries]
    if len(names) != len(set(names)):
        raise ValueError("Duplicate artifact paths")
    for entry in entries:
        path = PurePosixPath(entry.filename)
        if path.is_absolute() or ".." in path.parts or "\\" in entry.filename or entry.file_size > MAX_FILE:
            raise ValueError("Unsafe artifact path or size")
    return archive


def captures(archive, manifest, tested_sha):
    """Use the normal first suite, never a negative or repeat summary by accident."""
    summaries = [n for n in archive.namelist() if n.endswith("/comparisons/summary.json")
                 and n.split("/")[0].startswith("navigation-captures-")]
    if len(summaries) != 1:
        raise ValueError("MISSING_CAPTURE: require exactly one primary visual summary")
    prefix = summaries[0].removesuffix("summary.json")
    summary = json.loads(archive.read(summaries[0]))
    if summary.get("source") != tested_sha or summary.get("candidate") is not False:
        raise ValueError("STALE_CAPTURE: summary revision/candidate mismatch")
    rows = summary.get("results", [])
    if len(rows) != len(FRAMES) or {r["id"] for r in rows} != set(FRAMES):
        raise ValueError("MISSING_CAPTURE: incomplete frame matrix")
    images, states, citations = [], {}, {"summary", "pr-diff", "issue", "baseline-manifest", "ci:build", "ci:baseline-approval", "e2e-results"}
    hashes = {}
    import hashlib
    for row in rows:
        frame = row["id"]
        metadata_path = prefix + frame + ".json"
        state = json.loads(archive.read(metadata_path))
        if state.get("source") != tested_sha or state.get("environment") != manifest["environment"]:
            raise ValueError("STALE_CAPTURE: state/environment mismatch")
        if state.get("state", {}).get("scene", {}).get("renderer") != manifest["renderer"]:
            raise ValueError("CAPTURE_RENDERER_DRIFT")
        states[frame] = state
        for kind, suffix in (("actual", ".png"), ("expected", "-expected.png"), ("diff", "-diff.png")):
            path = prefix + frame + suffix
            data = archive.read(path)
            if len(data) > 2 * 1024 * 1024 or not data.startswith(b"\x89PNG\r\n\x1a\n"):
                raise ValueError("Invalid or oversized screenshot")
            checksum = hashlib.sha256(data).hexdigest()
            if kind != "diff" and checksum != row[kind]:
                raise ValueError("CAPTURE_HASH_MISMATCH")
            if kind == "expected" and checksum != manifest["images"][frame + ".png"]:
                raise ValueError("UNAPPROVED_REFERENCE")
            citation = f"image:{frame}:{kind}"
            citations.add(citation)
            hashes[path] = checksum
            images.append((citation, data))
        citations.add("state:" + frame)
        hashes[metadata_path] = hashlib.sha256(archive.read(metadata_path)).hexdigest()
    trace = prefix.removesuffix("comparisons/") + "trace.zip"
    trace_data = archive.read(trace)
    if not zipfile.is_zipfile(io.BytesIO(trace_data)):
        raise ValueError("MISSING_TRACE")
    hashes[trace] = hashlib.sha256(trace_data).hexdigest()
    citations.add("trace")
    return summary, images, citations, hashes, states


def blocking_reviews(reviews):
    latest = {}
    for row in sorted(reviews, key=lambda r: r["id"]):
        if row["state"] in ("APPROVED", "CHANGES_REQUESTED", "DISMISSED"):
            latest[row["user"]["login"]] = row["state"]
    return sorted(user for user, state in latest.items() if state == "CHANGES_REQUESTED")


def baseline_approval(api, pr, manifest_bytes, comments):
    """Independent of PR-supplied npm scripts: AI cannot approve references."""
    import hashlib
    checksum = hashlib.sha256(manifest_bytes).hexdigest()
    try:
        if (api.contents("client/tests/navigation-baselines/manifest.json", pr["base"]["sha"]) == manifest_bytes
                and api.contents("client/tests/navigation-baselines/manifest.json", "main") == manifest_bytes):
            return {"digest": checksum, "approval": "inherited from exact base"}
    except ValueError as exc:
        if "HTTP 404" not in str(exc):
            raise
    for comment in comments:
        if (comment.get("body", "").strip() == "Approve navigation baselines: " + checksum
                and comment.get("user", {}).get("type") == "User"
                and comment.get("author_association") in ("OWNER", "MEMBER", "COLLABORATOR")):
            return {"digest": checksum, "approval": "explicit independent human", "comment": comment["html_url"]}
    raise ValueError("BASELINE_APPROVAL_REQUIRED: human must approve exact manifest digest")


def test_results(archive):
    result = json.loads(archive.read("results.json"))
    def specs(suite):
        return suite.get("specs", []) + [s for child in suite.get("suites", []) for s in specs(child)]
    tests = [{"title": spec["title"], "project": test["projectName"], "status": r["status"],
              "errors": [e.get("message", "")[:2000] for e in r.get("errors", [])]}
             for suite in result.get("suites", []) for spec in specs(suite)
             for test in spec.get("tests", []) for r in test.get("results", [])]
    if not tests or len(tests) > 200:
        raise ValueError("MISSING_E2E_RESULTS or E2E_RESOURCE_BUDGET")
    return {"stats": result["stats"], "tests": tests}


def criteria_from_issue(body):
    match = re.search(r"^## Acceptance criteria\s*\n(.*?)(?=^## |\Z)", body, re.M | re.S)
    rows = re.findall(r"^- (.+(?:\n(?![-#]).+)*)", match[1], re.M) if match else []
    if not rows or len(rows) > 20:
        raise ValueError("ISSUE_CRITERIA_REQUIRED: use 1–20 acceptance bullets")
    return {f"AC-{i + 1}": row.strip() for i, row in enumerate(rows)}


def handoffs(comments):
    """Only existing authorized repository users can attest a session review."""
    result = []
    for comment in comments:
        body = comment.get("body", "")
        if (not body.startswith(HANDOFF_MARKER) or comment.get("user", {}).get("type") != "User"
                or comment.get("author_association") not in ("OWNER", "MEMBER", "COLLABORATOR")):
            continue
        try:
            record = json.loads(body.rsplit("\n```json\n", 1)[1].split("\n```", 1)[0])
            if record.get("schema_version") == SCHEMA:
                result.append({"record": record, "comment_url": comment["html_url"]})
        except (ValueError, IndexError, KeyError, AttributeError):
            continue
    return result


def session_review(packet, criteria, citations, submissions):
    """Verify a Codex session verdict; never invoke a model or provision agents."""
    for submission in reversed(submissions):
        record = submission["record"]
        if record.get("binding") != packet["binding"]:
            continue
        if (record.get("criteria_sha256") != digest(criteria)
                or record.get("issue_sha256") != digest(packet["issue"]["body"])
                or record.get("evidence_sha256") != digest(packet["evidence"]["sha256"])
                or record.get("run") != packet["run"] or record.get("packet_sha256") != digest(packet)):
            raise ValueError("STALE_REVIEW_HANDOFF: issue/criteria/evidence/CI changed")
        reviewer = record.get("reviewer", {})
        if (not isinstance(reviewer.get("id"), str) or not reviewer["id"]
                or not isinstance(record.get("implementer"), str) or not record["implementer"]
                or reviewer["id"] == record["implementer"]
                or reviewer.get("capability") != "distinct Codex subagent"):
            raise ValueError("INDEPENDENT_REVIEW_REQUIRED: distinct actual session reviewer needed")
        verdict = validate_review(record["review"], criteria, citations)
        return verdict, {"reviewer": reviewer["id"], "capability": reviewer["capability"],
                         "execution": "Codex session; CI only verifies the submitted record",
                         "independence": "authorized submitter attestation, not cryptographic agent authentication",
                         "comment_url": submission["comment_url"], "record_sha256": digest(record)}
    raise ValueError("REVIEWER_UNAVAILABLE: request a distinct Codex session review and submit its current-evidence record")


def records(comments):
    result = []
    for comment in comments:
        if comment.get("user", {}).get("login") != "github-actions[bot]" or comment.get("user", {}).get("type") != "Bot":
            continue
        body = comment.get("body", "")
        if not body.startswith(MARKER):
            continue
        try:
            value = json.loads(body.rsplit("\n```json\n", 1)[1].split("\n```", 1)[0])
            if value.get("schema_version") == SCHEMA:
                result.append(value)
        except (ValueError, IndexError):
            continue
    return result


def run(api, number, requested_run=None):
    pr = api.request(f"/pulls/{number}")
    if pr["state"] != "open" or pr["base"]["repo"]["full_name"] != REPO:
        raise ValueError("Only open mr-exception/salimon PRs can be reviewed")
    binding = identity(pr)
    comments = api.pages(f"/issues/{number}/comments")
    history = records(comments)
    submissions = handoffs(comments)
    report = {"schema_version": SCHEMA, "pr": number, "binding": binding,
        "cycle": 1 + sum(bool(r.get("review_consumed")) for r in history),
        "review_consumed": False, "verdict": "BLOCKED", "reasons": [], "criteria": [],
        "jobs": {}, "reviewer": None, "evidence": {}, "merge_authority": "human only"}
    report["runtime_sha256"] = digest([Path(__file__).read_text(), Path(__file__).with_name("policy.py").read_text()])
    try:
        issue_refs = re.findall(r"\b(?:Closes|Fixes|Resolves)\s+#(\d+)\b", pr.get("body") or "", re.I)
        if len(set(issue_refs)) != 1:
            raise ValueError("ISSUE_REFERENCE_REQUIRED: exactly one local Closes #N")
        issue = api.request(f"/issues/{issue_refs[0]}")
        criteria = criteria_from_issue(issue.get("body") or "")
        discussion = api.pages(f"/issues/{issue_refs[0]}/comments")
        reviews = api.pages(f"/pulls/{number}/reviews")
        blockers = blocking_reviews(reviews)
        report["issue"] = {"number": int(issue_refs[0]), "body_sha256": digest(issue["body"])}
        report["issue_context_sha256"] = digest({"body": issue["body"], "discussion": [c["body"] for c in discussion]})
        report["criteria"] = [{"id": cid, "status": "UNVERIFIED", "finding": "Review not completed"} for cid in criteria]
        if requested_run:
            ci = api.request(f"/actions/runs/{requested_run}")
        else:
            runs = api.pages(f"/actions/workflows/navigation-client.yml/runs?event=pull_request&head_sha={binding['head']}", "workflow_runs")
            if not runs:
                raise ValueError("CI_MISSING: run Navigation client on current PR head")
            ci = max(runs, key=lambda r: r["id"])
        report["run"] = {"id": ci["id"], "attempt": ci["run_attempt"], "url": ci["html_url"]}
        if ci["head_sha"] != binding["head"] or ci["event"] != "pull_request" or ci["path"] != ".github/workflows/navigation-client.yml" or ci["status"] != "completed":
            raise ValueError("STALE_CI: need completed Navigation client PR run at current head")
        if ci.get("pull_requests") and not any(p["number"] == number for p in ci["pull_requests"]):
            raise ValueError("CI_PR_MISMATCH")
        workflow_path = ".github/workflows/navigation-client.yml"
        if api.contents(workflow_path, binding["head"]) != api.contents(workflow_path, "main"):
            raise ValueError("CI_DEFINITION_CHANGED: human must review workflow changes; automated acceptance blocked")
        trusted_package = json.loads(api.contents("client/package.json", "main"))
        head_package = json.loads(api.contents("client/package.json", binding["head"]))
        for name, script in trusted_package["scripts"].items():
            if name in ("build", "test") or name.startswith("test:"):
                if head_package["scripts"].get(name) != script:
                    raise ValueError("CI_DEFINITION_CHANGED: mandatory npm scripts require human review")
        jobs = api.pages(f"/actions/runs/{ci['id']}/jobs?filter=latest", "jobs")
        report["jobs"] = {j["name"]: j["conclusion"] for j in jobs if j["name"] in REQUIRED_JOBS}
        # The uploaded primary summary records GITHUB_SHA, the tested PR merge ref.
        tested_sha = binding["merge"]
        if not tested_sha:
            raise ValueError("MERGE_REF_UNAVAILABLE: resolve conflicts and rerun CI")
        manifest_bytes = api.contents("client/tests/navigation-baselines/manifest.json", binding["head"])
        manifest = json.loads(manifest_bytes)
        report["baseline_approval"] = baseline_approval(api, pr, manifest_bytes, comments)
        files = api.pages(f"/pulls/{number}/files")
        if len(files) >= 500 or any(f.get("status") != "removed" and "patch" not in f and f.get("changes", 0) > 0
                                    and not f["filename"].endswith((".png", ".zip", ".ttf")) for f in files):
            raise ValueError("DIFF_INCOMPLETE: split PR or supply reviewable source diff")
        diff = [{k: f.get(k) for k in ("filename", "status", "patch", "sha")} for f in files]
        if len(json.dumps(diff)) > 200_000:
            raise ValueError("DIFF_RESOURCE_BUDGET: split PR")
        report["fingerprint"] = digest({"binding": binding, "run": report["run"], "jobs": report["jobs"], "runtime": report["runtime_sha256"],
            "manifest": manifest, "issue": issue["body"], "discussion": [c["body"] for c in discussion], "diff": diff, "reviews": reviews,
            "handoffs": submissions})
        artifacts = api.pages(f"/actions/runs/{ci['id']}/artifacts", "artifacts")
        expected = "navigation-evidence-" + tested_sha
        selected = [a for a in artifacts if a["name"] == expected and not a["expired"]]
        if len(selected) != 1 or selected[0]["size_in_bytes"] > MAX_ZIP:
            raise ValueError("MISSING_CAPTURE: require unexpired current-merge navigation artifact")
        archive = read_archive(api.request(f"/actions/artifacts/{selected[0]['id']}/zip", binary=True))
        summary, images, citations, hashes, states = captures(archive, manifest, tested_sha)
        if report["jobs"].get("build") == "success" and any(r.get("pass") is not True for r in summary["results"]):
            raise ValueError("CI_EVIDENCE_CONTRADICTION: passing build contains failed visual assertions")
        e2e = test_results(archive)
        hashes["results.json"] = __import__("hashlib").sha256(archive.read("results.json")).hexdigest()
        if sum(len(data) for _, data in images) > 8 * 1024 * 1024:
            raise ValueError("REVIEW_IMAGES_RESOURCE_BUDGET: 8 MiB aggregate limit")
        report["evidence_ok"] = True
        report["evidence"] = {"artifact_id": selected[0]["id"], "tested_sha": tested_sha,
            "url": ci["html_url"] + f"/artifacts/{selected[0]['id']}", "sha256": hashes, "summary": summary,
            "manifest_sha256": __import__("hashlib").sha256(manifest_bytes).hexdigest()}
        report["fingerprint"] = digest({"inputs": report["fingerprint"], "evidence": report["evidence"]})
        prior = next((r for r in history if r.get("fingerprint") == report["fingerprint"]), None)
        if prior:
            # Revalidate live artifacts before restoring a verdict. Expired,
            # deleted or replaced evidence cannot inherit an earlier ACCEPT.
            verdict, reasons = decide(binding, identity(api.request(f"/pulls/{number}")), report["jobs"],
                True, prior.get("review"), prior.get("error"))
            prior = {**prior, "verdict": verdict, "reasons": reasons}
            return publish(api, number, prior, comment=False)
        if report["cycle"] > MAX_CYCLES:
            raise ValueError("CYCLE_LIMIT: three session review cycles exhausted; human must investigate stuck loop")
        if blockers:
            raise ValueError("BLOCKING_REVIEW: resolve change requests from " + ", ".join(blockers))
        packet = {"binding": binding, "run": report["run"], "issue": {"number": issue_refs[0], "body": issue["body"],
            "discussion": [c["body"] for c in discussion]}, "criteria": criteria, "diff": diff,
            "jobs": report["jobs"], "evidence": report["evidence"], "states": states, "e2e": e2e,
            "baseline_manifest": manifest, "baseline_approval": report["baseline_approval"],
            "reviews": reviews, "citations": sorted(citations)}
        if len(json.dumps(packet)) > 350_000:
            raise ValueError("REVIEW_PACKET_RESOURCE_BUDGET")
        Path("qa-output").mkdir(exist_ok=True)
        Path("qa-output/handoff-request.json").write_text(json.dumps(packet, indent=2) + "\n")
        verdict, reviewer = session_review(packet, criteria, citations, submissions)
        report.update(review=verdict, reviewer=reviewer, criteria=verdict["criteria"], review_consumed=True)
    except (ValueError, KeyError, TypeError, AttributeError, IndexError, OSError, zipfile.BadZipFile) as exc:
        # No credentials or raw authentication headers are logged.
        report["error"] = str(exc)[:500]
        if not report.get("evidence_ok"):
            report["fingerprint"] = digest({"binding": binding, "issue": report.get("issue"), "run": report.get("run"),
                "error": report["error"], "runtime": report["runtime_sha256"], "handoffs": submissions})
        prior = next((r for r in history if r.get("fingerprint") == report["fingerprint"]), None)
        if prior:
            return publish(api, number, prior, comment=False)
    current = identity(api.request(f"/pulls/{number}"))
    report["verdict"], report["reasons"] = decide(binding, current, report["jobs"],
        report.get("evidence_ok", False), report.get("review"), report.get("error"))
    return publish(api, number, report)


def publish(api, number, report, comment=True):
    # Recheck immediately before side effects. A status is always attached to its
    # original head SHA; a concurrent push cannot transfer approval to a new head.
    current = identity(api.request(f"/pulls/{number}"))
    if current != report["binding"]:
        report = {**report, "verdict": "BLOCKED", "reasons": ["STALE_REVISION: rerun CI and review"]}
    if report["verdict"] == "ACCEPT":
        current_submissions = handoffs(api.pages(f"/issues/{number}/comments"))
        latest = next((s for s in reversed(current_submissions) if s["record"].get("binding") == report["binding"]), None)
        if not latest or digest(latest["record"]) != report["reviewer"].get("record_sha256") or latest["comment_url"] != report["reviewer"]["comment_url"]:
            report = {**report, "verdict": "BLOCKED", "reasons": ["STALE_REVIEW_HANDOFF: submitted review was changed or deleted"]}
        issue = api.request(f"/issues/{report['issue']['number']}")
        discussion = api.pages(f"/issues/{report['issue']['number']}/comments")
        issue_context = digest({"body": issue["body"], "discussion": [c["body"] for c in discussion]})
        if issue_context != report["issue_context_sha256"] or blocking_reviews(api.pages(f"/pulls/{number}/reviews")):
            report = {**report, "verdict": "BLOCKED", "reasons": ["STALE_REQUIREMENTS or BLOCKING_REVIEW: revalidate review"]}
        ci = api.request(f"/actions/runs/{report['run']['id']}")
        latest_jobs = api.pages(f"/actions/runs/{ci['id']}/jobs?filter=latest", "jobs")
        observed = {j["name"]: j["conclusion"] for j in latest_jobs if j["name"] in REQUIRED_JOBS}
        if ci["status"] != "completed" or ci["run_attempt"] != report["run"]["attempt"] or observed != report["jobs"]:
            report = {**report, "verdict": "BLOCKED", "reasons": ["STALE_CI: check rerun or results changed during review"]}
    Path("qa-output").mkdir(exist_ok=True)
    Path("qa-output/report.json").write_text(json.dumps(report, indent=2) + "\n")
    text = (f"{MARKER}\nIndependent QA cycle {report['cycle']}: **{report['verdict']}**\n\n"
            f"Head `{report['binding']['head']}`; base `{report['binding']['base']}`. Human retains merge authority.\n\n")
    for row in report["criteria"]:
        text += f"- {row['id']} {row['status']}: {row['finding']} Action: {row.get('action', 'Resolve blocker and rerun.')}\n"
    text += "\n" + "\n".join(report["reasons"]) + "\n\n```json\n" + json.dumps(report, separators=(",", ":")) + "\n```"
    if len(text) > 60_000:
        raise ValueError("REPORT_RESOURCE_BUDGET: refusing truncated durable cycle record")
    if comment:
        posted = api.request(f"/issues/{number}/comments", "POST", {"body": text})
        target = posted["html_url"]
    else:
        target = f"https://github.com/{REPO}/pull/{number}"
    api.request(f"/statuses/{report['binding']['head']}", "POST", {
        "state": "success" if report["verdict"] == "ACCEPT" else "failure", "context": CONTEXT,
        "description": f"{report['verdict']} — cycle {report['cycle']}; exact head/base required"[:140], "target_url": target})
    print(f"{CONTEXT}: {report['verdict']}; cycle {report['cycle']}")
    return report


def main():
    api = API(os.environ["GITHUB_TOKEN"])
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    if os.environ["GITHUB_REPOSITORY"] != REPO:
        raise ValueError("Repository boundary violation")
    requested = event.get("inputs", {}).get("pr")
    if requested:
        if not re.fullmatch(r"[1-9]\d*", requested):
            raise ValueError("Invalid PR number")
        run(api, int(requested))
    elif "issue" in event:
        new_body = event.get("comment", {}).get("body", "")
        previous_body = event.get("changes", {}).get("body", {}).get("from", "")
        if "pull_request" in event["issue"] and (new_body.startswith(HANDOFF_MARKER) or previous_body.startswith(HANDOFF_MARKER)):
            run(api, event["issue"]["number"])
    else:
        ci = event["workflow_run"]
        if ci["event"] != "pull_request" or ci["repository"]["full_name"] != REPO:
            return
        prs = ci.get("pull_requests", [])
        if not prs:
            # GitHub may omit associations on fork workflow_run payloads. Exact
            # current head and artifact merge SHA still bind all evidence.
            prs = [pr for pr in api.pages("/pulls?state=open") if pr["head"]["sha"] == ci["head_sha"]]
        for pr in prs:
            run(api, pr["number"], ci["id"])


if __name__ == "__main__":
    main()
