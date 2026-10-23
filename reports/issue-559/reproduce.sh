#!/usr/bin/env bash
# Run from repository root. Use an unused writable checkout/target path.
set -euo pipefail
fixture_root=${1:?fixture checkout path}
test_target=${2:?Cargo target path}
evidence_root=${3:?evidence output path}
mkdir -p "$evidence_root"
git worktree add --detach "$fixture_root" 2ecb271ff4af43ad102367cdea91c8cb08e375ec
mkdir -p "$fixture_root/client/runtime/src/sphere_workloads"
cp client/runtime/src/sphere_workloads/historical-inputs.json "$fixture_root/client/runtime/src/sphere_workloads/"
cp client/runtime/src/sphere_workloads.rs "$fixture_root/client/runtime/src/sphere_workloads.rs"
# Identical per-query timing instrumentation, old grid first.
cp client/physics/src/lib.rs "$fixture_root/client/physics/src/lib.rs"
CARGO_TARGET_DIR="$test_target" cargo test --release --manifest-path "$fixture_root/Cargo.toml" -p salimon-client --locked --no-run --message-format=json > "$evidence_root/build-before.jsonl"
python - "$evidence_root/build-before.jsonl" "$evidence_root/before-tests" <<'PY'
import json,shutil,sys
for line in open(sys.argv[1]):
 r=json.loads(line)
 if r.get('reason')=='compiler-artifact' and r.get('executable') and r.get('profile',{}).get('test'):shutil.copy2(r['executable'],sys.argv[2])
PY
python reports/issue-559/collect.py --binary "$evidence_root/before-tests" --output "$evidence_root/before"
cp client/physics/src/broad_phase.rs "$fixture_root/client/physics/src/broad_phase.rs"
CARGO_TARGET_DIR="$test_target" cargo test --release --manifest-path "$fixture_root/Cargo.toml" -p salimon-client --locked --no-run --message-format=json > "$evidence_root/build-after.jsonl"
python - "$evidence_root/build-after.jsonl" "$evidence_root/after-tests" <<'PY'
import json,shutil,sys
for line in open(sys.argv[1]):
 r=json.loads(line)
 if r.get('reason')=='compiler-artifact' and r.get('executable') and r.get('profile',{}).get('test'):shutil.copy2(r['executable'],sys.argv[2])
PY
python reports/issue-559/collect.py --binary "$evidence_root/after-tests" --output "$evidence_root/after"
