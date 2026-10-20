# #379 — deterministic new-run starting supplies

Implementation date: 2026-10-20. Based on main `1260492805bd7497ce068727394a22c458910e53`.

## Delivered behavior

Normal-play new runs and verified post-death restart candidates receive two basic
rations, one medkit and exact recipe ingredients for one additional ration in the
existing Fabricator reserves. Recipe definitions supply masses, categories and
integer ingredient quantities. Starter output identities reserve jobs 1/2;
ordinary crafting starts at job 3. No work, time or Core energy is charged for
the initial endowment. Only the raw reserves count toward ship mass (currently
0.5 kg); existing committed-input/tray/collected-inventory mass rules are retained.

Restore keeps complete saved owners, including spent slots, depleted reserves,
old empty saves and allocator state. Focus/cursor release, cockpit reentry, view
changes and lifecycle reconstruction do not replenish anything. Restart uses the
existing verified archive/two-generation transaction before exposing a fresh run.
No save schema, unlock acquisition, input mapping or second inventory is added.
Isolated E2E/benchmark fixtures retain their explicit empty initialization; new-run
portable E2E tests use the same constructor as production launch/restart.

Canonical behavior: [supply/save contract](../../docs/save-contract.md#starting-supplies-379).

## Regression evidence

- Fabrication test: deterministic item/category/slot/recipe values, exact reserves,
  no initial work/draw, allocator reservation, rejected batch atomicity, exactly
  one additional ration, full inventory save validation and no duplicate tray
  collection.
- Session E2E: construct/save/restore, craft/collect/consume/resave/reload, exact
  reserve/queue/tray/inventory mass transitions, repeated stale consumption,
  vacant slots and exhausted reserves preserved across repeated reloads.
- Existing-save migration: schema 1/2/3 empty saves remain empty through launch
  recovery into a pre-populated destination.
- Native storage orchestration without a window: representative Earth launch,
  first checkpoint, spent food/medical restore, terminal restart, repeat restart
  rejection, and both fresh fallback generations independently decoded with one
  grant. Existing archive/publication failure tests remain part of the workspace.
- Portable cockpit entry/exit, input-latch clearing and view changes preserve
  inventory and reserve state. OS input delivery remains a separate native check.

## Validation

Environment: Linux x86_64, stable Rust 1.99.1; a clean replacement workspace
required reinstalling Rust and downloading the locked dependency cache.

| Command/check | Result |
| --- | --- |
| `cargo test --workspace --locked` | PASS: 667 tests, 0 failures; 22 existing opt-in tests ignored |
| `cargo build --workspace --locked` | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| `cargo fmt --all -- --check` | PASS |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | PASS: 93 tests (Python 3.12.14) |
| Changed local documentation links; `git diff --check` | PASS |
| Native launch with isolated `SALIMON_SAVE_DIR` | FAILED before window creation: neither `WAYLAND_DISPLAY`, `WAYLAND_SOCKET` nor `DISPLAY` is set |
| Native GUI/input/DPI, Metal/M1 and frame-tail performance | NOT RUN; existing #376 |

The first Rust run found an error in the new fallback test: it supplied a raw
checksum-wrapped `.sav` file to the JSON restore API (`expected value at line 1
column 1`). The test now places each checkpoint alone in an isolated directory
and recovers through production `Checkpoints::open_at`. The focused three session
tests then passed, followed by the complete workspace and quality gates above.
No production persistence rule or test tolerance was relaxed.

## Native limits

No new UI/asset implementation is part of this issue. Reference native visual,
OS input/focus, DPI and performance checks remain in existing
[#376](https://github.com/mr-exception/salimon/issues/376), whose approved expansion
scope explicitly includes starter supplies #379. Portable storage tests are not
native screenshots, Metal coverage or reference hardware signoff. Expanded
survival verification remains #375 with its other feature dependencies.
