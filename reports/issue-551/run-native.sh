#!/bin/bash
set -euo pipefail
source /workspace/.salimon-tools/env.sh
export DISPLAY=:101 WGPU_BACKEND=vulkan LP_NUM_THREADS=2 RUST_LOG=info
export VK_DRIVER_FILES=/workspace/.salimon-tools/sysroot/usr/share/vulkan/icd.d/lvp_icd.json
Xvfb :101 -screen 0 1280x800x24 -nolisten tcp -fp /workspace/.salimon-tools/sysroot/usr/share/fonts/X11/misc > artifacts/issue-551/checks/xvfb-101.log 2>&1 &
issue551_display_pid=$!
trap 'kill "$issue551_display_pid" 2>/dev/null || true' EXIT
for attempt in $(seq 1 50); do
  test -S /tmp/.X11-unix/X101 && break
  sleep 0.1
done
python3 scripts/salimon-test suite --group scout-construction --evidence --binary target/debug/salimon-client --artifacts artifacts/issue-551/verified-integrated --screenshot-command '["native"]' --summary > artifacts/issue-551/checks/verified-integrated.json 2> artifacts/issue-551/checks/verified-integrated.stderr
python3 reports/issue-489/walkthrough.py --binary target/debug/salimon-client --output artifacts/issue-551/machinery > artifacts/issue-551/checks/machinery.log 2>&1
python3 reports/issue-505/verify.py --binary target/debug/salimon-client --output artifacts/issue-551/archive --scale 1 > artifacts/issue-551/checks/archive.log 2>&1
python3 reports/issue-505/verify.py --binary target/debug/salimon-client --output artifacts/issue-551/archive-power --power > artifacts/issue-551/checks/archive-power.log 2>&1
