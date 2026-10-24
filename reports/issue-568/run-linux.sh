#!/usr/bin/env bash
# Full nightly-equivalent local Linux reproduction; preserve independent failures.
set -u
out=/source/artifacts/verification-568
mkdir -p "$out"
git config --global --add safe.directory /source
if [ "$(git rev-parse HEAD)" != "${EXPECTED_REVISION:?}" ] || [ -n "$(git status --porcelain --untracked-files=no)" ]; then
  echo 'Expected clean tracked source revision' > "$out/admission-error.log"
  exit 2
fi
export XDG_RUNTIME_DIR=/tmp/salimon-xdg
mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"
icds=(/usr/share/vulkan/icd.d/lvp_icd*.json)
export VK_DRIVER_FILES="${icds[0]}"
export SALIMON_GLB_CANDIDATES=/source/artifacts/glb
python - "$out/context.json" <<'PY'
import json,subprocess,platform,os,sys
from pathlib import Path
commands=[['git','rev-parse','HEAD'],['git','status','--porcelain','--untracked-files=no'],['rustc','-vV'],['cargo','--version'],['node','--version'],['python','--version'],['uname','-a']]
context={'lane':'local Linux container, not hosted GitHub CI or native Metal','platform':platform.platform(),'commands':{' '.join(c):subprocess.check_output(c,text=True).strip() for c in commands},'vulkan_icd':os.environ['VK_DRIVER_FILES'],'icd_metadata':Path(os.environ['VK_DRIVER_FILES']).read_text()}
Path(sys.argv[1]).write_text(json.dumps(context,indent=2)+'\n')
PY
run_step() {
  local name=$1
  shift
  "$@" > "$out/$name.log" 2>&1
  local code=$?
  python - "$out/status.jsonl" "$name" "$code" "$@" <<'PY'
import json,sys,datetime
with open(sys.argv[1],'a') as f:f.write(json.dumps({'step':sys.argv[2],'exit_code':int(sys.argv[3]),'command':sys.argv[4:],'finished_utc':datetime.datetime.now(datetime.timezone.utc).isoformat()})+'\n')
print(sys.argv[2]+': '+sys.argv[3],flush=True)
PY
}
run_step rust-components rustup component add rustfmt clippy
run_step npm npm ci --prefix models/tools/gltf --ignore-scripts --no-audit --no-fund
run_step models python -m unittest discover -s models/tests -v
run_step scripts python -m unittest discover -s scripts -p 'test_*.py' -v
run_step format cargo fmt --all -- --check
run_step clippy cargo clippy --workspace --all-targets --locked -- -D warnings
run_step workspace cargo test --workspace --locked
run_step persistence cargo test -p salimon-client --locked persistence_extended -- --ignored
run_step glb python models/tools/process_glb.py all --profile exact-v1 --output artifacts/glb
run_step importer cargo test -p salimon-renderer --locked offline_glb_candidate -- --ignored
run_step headless-resource cargo test -p salimon-renderer --locked headless_resource -- --ignored
run_step build cargo build --release --workspace --locked
run_step baseline timeout 600 xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --summary --binary /target/release/salimon-client
run_step evidence timeout 1200 xvfb-run -a -s '-screen 0 1280x800x24' python scripts/salimon-test suite --evidence --summary --binary /target/release/salimon-client --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
run_step corpus timeout 900 xvfb-run -a -s '-screen 0 1280x800x24' python scripts/model_visual_tests.py capture --binary /target/release/salimon-client
run_step temporal python scripts/model_visual_tests.py compare --temporal-only
run_step appearance python scripts/model_visual_tests.py compare --reviewed --baselines scenarios/visual/baselines/software-vulkan-option-a-v2
run_step package python scripts/build_game.py --profile release --output artifacts/build/release
run_step packaged-smoke timeout 300 xvfb-run -a -s '-screen 0 1920x1080x24' python scripts/packaged_smoke.py --binary artifacts/build/release/salimon-client --timeout 90
run_step experience python scripts/experience_suite.py --output artifacts/experience-568
export SALIMON_BALANCE_REPORT=/source/artifacts/verification-568/balance.json
export SALIMON_AVAILABILITY_REPORT=/source/artifacts/verification-568/availability.json
run_step balance cargo test -p salimon-client survival_balance --locked
run_step tracked-clean bash -c 'git status --porcelain --untracked-files=no; test -z "$(git status --porcelain --untracked-files=no)"'
python - "$out/status.jsonl" <<'PY'
import json,sys
rows=[json.loads(l) for l in open(sys.argv[1])]
failed=[r['step'] for r in rows if r['exit_code'] and not (r['step'] in ('appearance','experience') and r['exit_code']==3)]
print(json.dumps({'failed':failed,'pending':[r['step'] for r in rows if r['exit_code']==3]}))
raise SystemExit(bool(failed))
PY
