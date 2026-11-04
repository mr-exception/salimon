"""Fail-closed review policy, independent of network and model providers."""
import hashlib
import json

SCHEMA = 1
CONTEXT = "salimon/independent-qa"
MAX_CYCLES = 3
MAX_ATTEMPTS = 2
REQUIRED_JOBS = ("build", "baseline-approval")
FRAMES = ("empty-scanner", "visible-contacts", "panned-contacts", "selected-body",
          "zoom-out", "zoom-in", "heading-velocity", "opposite-vectors")


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


def identity(pr):
    return {"head": pr["head"]["sha"], "base": pr["base"]["sha"],
            "base_ref": pr["base"]["ref"], "merge": pr.get("merge_commit_sha")}


def validate_review(review, criteria, citations):
    if not isinstance(review, dict) or set(review) != {"verdict", "summary", "criteria"}:
        raise ValueError("Reviewer response has invalid fields")
    if review["verdict"] not in ("ACCEPT", "REQUEST_CHANGES", "BLOCKED", "INCONCLUSIVE"):
        raise ValueError("Invalid reviewer verdict")
    if not isinstance(review["summary"], str) or not 1 <= len(review["summary"]) <= 3000:
        raise ValueError("Reviewer summary missing or too long")
    rows = review["criteria"]
    if not isinstance(rows, list) or len(rows) != len(criteria):
        raise ValueError("Reviewer omitted criteria")
    if any(not isinstance(r, dict) or not isinstance(r.get("id"), str) for r in rows):
        raise ValueError("Malformed reviewer criterion")
    if {r.get("id") for r in rows} != set(criteria):
        raise ValueError("Reviewer criteria do not match issue")
    for row in rows:
        if set(row) != {"id", "status", "evidence", "finding", "action"}:
            raise ValueError("Invalid criterion fields")
        if row["status"] not in ("PASS", "FAIL", "UNVERIFIED"):
            raise ValueError("Invalid criterion status")
        if not isinstance(row["evidence"], list) or not row["evidence"] or any(not isinstance(c, str) or c not in citations for c in row["evidence"]):
            raise ValueError("Reviewer cited missing evidence")
        for field in ("finding", "action"):
            if not isinstance(row[field], str) or not 1 <= len(row[field]) <= 3000:
                raise ValueError("Missing actionable criterion text")
    if review["verdict"] == "ACCEPT" and any(r["status"] != "PASS" for r in rows):
        raise ValueError("Reviewer ACCEPT contradicts criterion results")
    return review


def decide(binding, current, jobs, evidence_ok, review, error=None):
    """AI approval can never override a deterministic or revision veto."""
    reasons = []
    if binding != current:
        reasons.append("STALE_REVISION: head/base/merge changed; rerun CI and review")
    if not evidence_ok:
        reasons.append("MISSING_CAPTURE: require originals, approved references, diffs, state and trace")
    if error:
        reasons.append(error)
    failed = [j for j in REQUIRED_JOBS if jobs.get(j) == "failure"]
    # A dependent approval job can be skipped because build failed. The failure
    # is actionable REQUEST_CHANGES once the reviewer has real capture evidence.
    if not reasons and review is not None and failed:
        return "REQUEST_CHANGES", ["CI_VETO: fix " + ", ".join(failed) + "; AI cannot override tests"]
    missing = [j for j in REQUIRED_JOBS if j not in jobs or jobs[j] not in ("success", "failure")]
    if missing:
        reasons.append("CI_INCOMPLETE: " + ", ".join(missing))
    if reasons or review is None:
        return "BLOCKED", reasons or ["REVIEWER_UNAVAILABLE"]
    if review["verdict"] in ("BLOCKED", "INCONCLUSIVE") or any(r["status"] == "UNVERIFIED" for r in review["criteria"]):
        return "BLOCKED", [review["summary"]]
    if review["verdict"] == "REQUEST_CHANGES" or any(r["status"] == "FAIL" for r in review["criteria"]):
        return "REQUEST_CHANGES", [review["summary"]]
    return "ACCEPT", []


def review_schema(criteria, citations):
    def obj(properties):
        return {"type": "object", "properties": properties, "required": list(properties), "additionalProperties": False}
    return obj({"verdict": {"type": "string", "enum": ["ACCEPT", "REQUEST_CHANGES", "BLOCKED", "INCONCLUSIVE"]},
                "summary": {"type": "string"},
                "criteria": {"type": "array", "items": obj({
                    "id": {"type": "string", "enum": list(criteria)},
                    "status": {"type": "string", "enum": ["PASS", "FAIL", "UNVERIFIED"]},
                    "evidence": {"type": "array", "items": {"type": "string", "enum": sorted(citations)}},
                    "finding": {"type": "string"}, "action": {"type": "string"}})}})
