# Issue 191 — Power-dependent interior zero-G

Added view-relative three-axis momentum/contacts indoors, outage/walking/EVA
transition velocity preservation, no-teleport power restoration and cargo ceiling
containment without ambient zero-G damping. Existing carry/Core intake and stable
IDs remain authoritative. Ceiling support is conservatively radius-based;
fixture footprints remain existing conservative proxies. Cabin acceleration
forces and full inertial crew dynamics are outside scope.

Validation: `git diff --check` passed; added character coast/contact/restoration
and cargo drift/ceiling/power-return regressions. Workspace format/lint/tests/build
run in Actions; local Rust toolchain unavailable. Native Apple M1 excluded.

## Completed CI validation

[Native builds run 38579723684](https://github.com/mr-exception/salimon/actions/runs/38579723684)
passed on Ubuntu 24.04, macOS 14 and Windows 2022 at commit
`35ba59807d59aab11179e657d2efef5a4dda9f1f`.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: passed.
- `python -m unittest discover -s models/tests -v`: passed; Blender-only
  tests skipped on the hosted runners as declared by the test suite.
- `python scripts/build_game.py --profile debug --output artifacts/build/debug`
  and the equivalent release staging command: passed on all three OSes.
- Linux software-Vulkan baseline, cockpit-window capture, resource-collection
  and ship-EVA evidence suites, headless rendering contracts, fragment CPU
  checks and packaged X11 input smoke: passed through the existing workflow.

These hosted checks do not provide physical Apple M1 gameplay/performance signoff.
