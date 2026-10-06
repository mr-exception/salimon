# Issue #122 — Current documentation and performance policy

Completed on 2026-10-06 against main `aaf2cd20222f34c9599fbfb4bcf3312a4999d030`.

## Outcome

Active guides describe the current native client and portable domains. Root
smoke instructions and renderer/asset validation guidance now distinguish
behavior checks from revision-specific performance measurements. Rolling FPS
and average/p95 frame intervals cannot establish a per-frame 60 FPS floor;
the dated Phase 0 evaluation retains its original go-with-revisions conclusion.

- Updated runtime/ship package descriptions without changing dependencies.
- Made landed doorway documentation use the active solid body's `SurfaceFrame`,
  matching `controller/doorway.rs` and `controller/surface.rs`.
- Replaced obsolete task-number dependencies in maintained client, runtime,
  ship, world, character, renderer, diagnostics and asset guides with named
  features/contracts. Historical references remain explicitly historical.
- Added `cargo test -p salimon-math --locked` to focused validation.
- Repaired two missing local evidence links in the scout guide: the earlier
  cargo design links to its original issue, and lower-window evidence links to
  the checked-in issue #111 report.

Historical Phase 0 specification/evaluation and prior task reports are unchanged.
No gameplay source, controls, authored/generated assets or lockfile changed.

## Validation

Environment: Linux 6.18.44 x86_64; Python 3.12.14. Rust/Cargo are not installed.

| Check | Result |
| --- | --- |
| `python -m unittest discover -s scripts -p 'test_*.py'` | Passed: 34 tests |
| `git diff --check` | Passed |
| `python /tmp/check_122.py` (one-off documentation audit) | Passed: relative link targets and Markdown heading anchors in changed guides/report; TOML parsing; focused math command; active task-reference audit; historical file byte comparison |
| `rustc --version`, `cargo --version` | Unavailable: commands not found |
| Workspace Rust format/Clippy/test/build and native/model graphical gates | Not run: documentation/package-description-only change; no executable or asset changes, per the documentation validation policy |

The one-off audit resolves relative Markdown links from each changed file,
checks heading fragments against the target, parses manifests with Python
`tomllib`, and compares both historical Phase 0 documents with `git show HEAD`.
No new runtime tests or screenshots are needed for this documentation-only task.
This change makes no new hardware performance claim.
