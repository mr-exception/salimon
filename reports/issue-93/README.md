# Issue #93 — Camera-relative mining viewmodel

2026-10-05; based on main `9c68fb0213a02398a686c7c5107f5ed27e32351c`.

## Outcome

The Blender migration (#92) already replaced the world-space placeholder with
camera-local geometry. This change consolidates its placement into the single
renderer-owned `GRIP_TO_VIEW` matrix and composes it with the active camera
projection on the CPU. WGSL consumes the resulting model-to-clip matrix; it no
longer defines a second placement formula. The grip remains 0.55 m forward,
0.20 m right and 0.20 m down, following both camera yaw and pitch.

Regression tests compare every authored vertex against independent placement
along the actual scene camera's forward/right/orthonormal-up basis at horizontal
look angles, both controller pitch limits, tilted gravity and 1e12 m origins.
They check viewport framing, near-plane clearance and reverse-Z depth. Another
test checks stable screen placement and projection of the forward aim ray to
screen center. Runtime mining targeting, equip/stow and active feedback are
unchanged; there is one authored-tool presentation path.

The existing native mining evidence scenario now includes maximum upward pitch,
yaw at maximum pitch, maximum downward pitch and stowing at that angle. Its
prior extraction/mass/target/equip checks remain in place.

## Validation

Linux x86_64; rustc 1.99.0; Python 3.12. Compilation uses checked-in GLB exports,
without Blender regeneration or asset modifications.

Passed:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo test -p salimon-renderer --locked` (41 tests)
- `cargo test --workspace --locked`
- `cargo build --workspace --locked`
- `python -m unittest discover -s scripts -p 'test_*.py'` (34 tests)
- `git diff --check`

Native attempt (failed before renderer readiness):

```sh
WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
xvfb-run -a -s '-screen 0 1280x800x24' \
python scripts/salimon-test run scenarios/evidence/mining-tool.json \
  --binary target/debug/salimon-client --artifacts artifacts/issue-93 \
  --screenshot-command '["python", "scripts/capture_settled.py", "{path}"]'
```

Xvfb cannot create local/unix listening sockets in this environment. A direct
AF_UNIX socket probe returns `PermissionError: Operation not permitted`; the
client reports `Failed to open connection to X server`. No native screenshot
or gameplay result is claimed. Native evidence must be run on a display-capable
host (or the workflow's optional visual evidence route) before visual acceptance.

## Limitations

Ordinary world reverse-Z occlusion remains active, preserving the practical
avoidance of drawing the tool through nearby geometry. No new wall collision,
viewmodel retraction or independent always-on-top pass is introduced. CPU tests
prove placement/framing, not live GPU appearance or hardware performance.
macOS/Windows graphical validation and the reference M1 performance smoke were
not run. The issue stays open for PR review/merge.
