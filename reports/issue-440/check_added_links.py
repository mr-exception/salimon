"""Evidence helper: added Markdown links only; not repository-wide U37 lint.

Run from the repository root: python reports/issue-440/check_added_links.py
Defaults to the implementation PR parent (HEAD when run before this PR's commit).
Use --base <parent SHA> after commit. Does not prove semantic documentation accuracy.
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import unquote


def anchors(path):
    result = set()
    counts = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^#{1,6}\s+(.+?)\s*#*\s*$", line)
        if match:
            slug = re.sub(r"[^\w\s-]", "", match[1].lower()).replace(" ", "-")
            count = counts.get(slug, 0)
            counts[slug] = count + 1
            result.add(slug if count == 0 else f"{slug}-{count}")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", default="HEAD")
    args = parser.parse_args()
    diff = subprocess.check_output(
        ["git", "diff", "--unified=0", args.base, "--", "*.md"], text=True
    )
    added = []
    path = None
    for line in diff.splitlines():
        if line.startswith("+++ b/"):
            path = Path(line[6:])
        elif line.startswith("+") and not line.startswith("+++") and path:
            added.append((path, line[1:]))
    untracked = subprocess.check_output(
        ["git", "ls-files", "--others", "--exclude-standard"], text=True
    ).splitlines()
    for name in untracked:
        path = Path(name)
        if path.suffix == ".md":
            added.extend((path, line) for line in path.read_text().splitlines())
    errors = []
    checked = 0
    for path, line in added:
        for raw in re.findall(r"\[[^\]]*\]\(([^\s)]+)\)", line):
            if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", raw):
                continue
            target, _, anchor = unquote(raw).partition("#")
            resolved = path.parent / target if target else path
            checked += 1
            if not resolved.exists():
                errors.append(f"{path}: missing {raw}")
            elif anchor and resolved.suffix == ".md" and anchor not in anchors(resolved):
                errors.append(f"{path}: missing anchor {raw}")
    print(f"Added local Markdown links checked: {checked}; errors: {len(errors)}")
    for error in errors:
        print(error)
    return bool(errors)


if __name__ == "__main__":
    sys.exit(main())
