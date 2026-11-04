"""Invalidate existing status on head/base/merge drift without calling an AI."""
from policy import CONTEXT, identity
from review import API, REPO, records
import json
import os
from pathlib import Path


def invalidate(api, pr):
    history = records(api.pages(f"/issues/{pr['number']}/comments"))
    current = identity(pr)
    # Set pending on every new head, and replace same-head success when its base
    # advances. An old approval is never transferred to a new merge candidate.
    if not history or history[-1]["binding"] != current:
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
        if len(prs) > 80:
            raise ValueError("Invalidation budget: more than 80 open PRs; split scan")
        for pr in prs:
            invalidate(api, pr)


if __name__ == "__main__":
    main()
