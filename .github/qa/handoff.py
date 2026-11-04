"""Build a local review comment from an actual Codex review and evidence packet.

This command does not contact a model, GitHub, or any external service.
"""
import argparse
import json
from pathlib import Path

from policy import digest, validate_review
from review import HANDOFF_MARKER


def make_handoff(packet, verdict, reviewer, implementer):
    if not reviewer or not implementer or reviewer == implementer:
        raise ValueError("A distinct actual reviewer and implementer are required")
    validate_review(verdict, packet["criteria"], set(packet["citations"]))
    return {"schema_version": 1, "binding": packet["binding"], "run": packet["run"],
            "packet_sha256": digest(packet),
            "criteria_sha256": digest(packet["criteria"]), "issue_sha256": digest(packet["issue"]["body"]),
            "evidence_sha256": digest(packet["evidence"]["sha256"]), "implementer": implementer,
            "reviewer": {"id": reviewer, "capability": "distinct Codex subagent"}, "review": verdict}


def comment(record):
    return (HANDOFF_MARKER + "\nCodex session independent review; submitter attests actual separate agent execution.\n\n```json\n"
            + json.dumps(record, indent=2) + "\n```\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("packet", "review", "output"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--reviewer", required=True)
    parser.add_argument("--implementer", required=True)
    args = parser.parse_args()
    packet = json.loads(args.packet.read_text())
    result = json.loads(args.review.read_text())
    # Actual saved image-review records can wrap their structured verdict. Check
    # their hashes/source before adapting them to the current handoff contract.
    if "review" in result:
        if result["source"] != packet["evidence"]["tested_sha"] or result["capture_sha256"] != packet["evidence"]["sha256"] or result["reviewer"] != args.reviewer:
            raise ValueError("Reviewer record does not bind these exact captures/source/identity")
        result = result["review"]
    record = make_handoff(packet, result, args.reviewer, args.implementer)
    args.output.write_text(comment(record))


if __name__ == "__main__":
    main()
