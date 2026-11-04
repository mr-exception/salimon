"""Invalidate existing status on head/base/merge drift without calling an AI."""
from policy import CONTEXT, identity, digest
from review import API, REPO, records, handoffs
import json
import os
from pathlib import Path


def invalidate(api, pr):
    comments = api.pages(f"/issues/{pr['number']}/comments")
    history = records(comments)
    current = identity(pr)
    # Set pending on every new head, and replace same-head success when its base
    # advances. An old approval is never transferred to a new merge candidate.
    stale = not history or history[-1]["binding"] != current
    if not stale and history[-1]["verdict"] == "ACCEPT":
        accepted = history[-1]
        submission = next((s for s in reversed(handoffs(comments)) if s["record"].get("binding") == current), None)
        stale = not submission or digest(submission["record"]) != accepted["reviewer"]["record_sha256"]
        issue = api.request(f"/issues/{accepted['issue']['number']}")
        discussion = api.pages(f"/issues/{accepted['issue']['number']}/comments")
        stale = stale or digest({"body": issue["body"], "discussion": [c["body"] for c in discussion]}) != accepted["issue_context_sha256"]
    if stale:
        api.request(f"/statuses/{current['head']}", "POST", {"context": CONTEXT, "state": "pending",
            "description": "QA pending — current head/base CI and independent review required",
            "target_url": pr["html_url"]})


def main():
    if os.environ["GITHUB_REPOSITORY"] != REPO:
        raise ValueError("Repository boundary violation")
    api = API(os.environ["GITHUB_TOKEN"])
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    if "pull_request" in event:
        invalidate(api, api.request(f"/pulls/{event['pull_request']['number']}"))
    else:
        prs = api.pages("/pulls?state=open")
        if len(prs) > 20:
            raise ValueError("Invalidation budget: more than 20 open PRs; split scan")
        for pr in prs:
            invalidate(api, pr)


if __name__ == "__main__":
    main()
