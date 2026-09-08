#!/bin/sh
set -eu
if [ "$(id -u)" = 0 ]; then
  chown -R 1000:1000 /home/agent/.codex /home/agent/github-token /workspace
  exec gosu 1000:1000 "$0" "$@"
fi
repo="$1"
branch="$2"
prompt="$3"
export GH_TOKEN="$(cat /home/agent/github-token)"
rm /home/agent/github-token
gh auth setup-git
git clone "https://github.com/$repo.git" repo
cd repo
git checkout -b "$branch"
git config user.name 'Agent Control'
git config user.email 'agent-control@users.noreply.github.com'
before="$(git rev-parse HEAD)"
codex exec --dangerously-bypass-approvals-and-sandbox "$prompt"
after="$(git rev-parse HEAD)"
test "$before" != "$after"
test -z "$(git status --porcelain)"
issue="$(printf '%s' "$branch" | cut -d- -f2)"
test -f "reports/issue-$issue/README.md"
git push origin "HEAD:refs/heads/$branch"
