#!/usr/bin/env bash
# Isolated archived-layout CPU replay. Run serially without builds/render loads.
set -euo pipefail
fixture_root=${1:?unused fixture checkout}
test_target=${2:?dedicated Cargo target path}
evidence_root=${3:?fresh evidence output}
repo_root=$(pwd)
mkdir -p "$evidence_root"
git worktree add --detach "$fixture_root" 2ecb271ff4af43ad102367cdea91c8cb08e375ec
git -C "$fixture_root" apply "$repo_root/reports/issue-592/fixture-before.patch"
build_test() {
  local lane=$1
  CARGO_TARGET_DIR="$test_target" cargo test --release --manifest-path "$fixture_root/Cargo.toml" -p salimon-client --locked --no-run --message-format=json > "$evidence_root/build-$lane.jsonl"
  python3 - "$evidence_root/build-$lane.jsonl" "$evidence_root/$lane-tests" <<'PY'
import json,shutil,sys
for line in open(sys.argv[1]):
 r=json.loads(line)
 if r.get('reason')=='compiler-artifact' and r.get('executable') and r.get('profile',{}).get('test'):shutil.copy2(r['executable'],sys.argv[2])
PY
}
build_test before
python3 reports/issue-592/collect.py --binary "$evidence_root/before-tests" --output "$evidence_root/before"
git -C "$fixture_root" apply "$repo_root/reports/issue-592/prefix-harness.patch"
build_test before-prefix
SALIMON_STRESS_PREFIX_SAMPLES=32 python3 reports/issue-592/collect.py --binary "$evidence_root/before-prefix-tests" --output "$evidence_root/before-prefix" --cases 1000-mixed-moving
git -C "$fixture_root" apply "$repo_root/reports/issue-592/optimization.patch"
build_test after
python3 reports/issue-592/collect.py --binary "$evidence_root/after-tests" --output "$evidence_root/after"
SALIMON_STRESS_PREFIX_SAMPLES=32 python3 reports/issue-592/collect.py --binary "$evidence_root/after-tests" --output "$evidence_root/after-prefix" --cases 1000-mixed-moving
python3 reports/issue-592/analyze.py "$evidence_root/before" "$evidence_root/after" --output "$evidence_root/comparison.json"

python3 reports/issue-592/analyze.py "$evidence_root/before-prefix" "$evidence_root/after-prefix" --prefix-only --output "$evidence_root/prefix-comparison.json"
