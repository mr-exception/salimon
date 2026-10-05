# Issue #98 — Documentation and maintenance conventions

Implemented on 2026-10-05 from `b926083cdaf44047cd669e7767c755d7ebb42616`.

## Outcome

Refreshed active root/client/world/runtime guides against current native gameplay,
in-memory resource retention and Blender asset ownership. The architecture guide
now separates implemented behavior, accepted constraints and future candidates;
obsolete Notion worker/task-order instructions are removed from active guidance.

Added a feature/source/test/scenario map, shared Rust/Python/WGSL/asset conventions
and an exact validation matrix. All six crate maintenance guides link these
canonical rules. Completion reports with applicable images are mandatory under
root `reports/`; `docs/` remains maintained specifications/guides.

Moved 122 historical report/evidence files from 18 `docs/issue-*` / `docs/task-*`
folders. Retained 120 files after removing two byte-identical PNG copies with
[explicit aliases and hashes](../evidence-aliases.json). Updated repository links;
all retained non-Markdown evidence remains byte-for-byte identical to the base.
Past assertions and logs are preserved rather than rewritten. The #29 report
now discloses that its two original image files were identical. Historical
`task-*/validation.md` names remain supported; new reports use `README.md`.

No runtime code, generated model/layout, dependencies or workflow changed.
No new product decisions or broad refactors were introduced. The confirmed
feedback in #98 (root reports plus applicable images) is incorporated.

## Validation

Environment: Linux, Python 3.12.14. Installed model requirements
(`jsonschema==4.26.0`).

| Check | Result |
| --- | --- |
| `python -m pip install -r models/tools/requirements.txt` | Passed |
| `python -m unittest discover -s scripts -p 'test_*.py'` | 34 passed |
| `python -m unittest discover -s models/tests -v` | 39 passed; 1 real Blender integration test explicitly skipped (BLENDER unavailable) |
| `python models/assets/ships/salimon-scout/validate.py` | Passed; 5,890 triangles, 109 primitives, 13 materials, 481,904-byte GLB |
| `python models/tools/validate_asset.py resource.iron-fragment` | Passed |
| Repository Markdown relative paths and heading anchors | Checked against local destinations |
| Migrated evidence integrity and removed-path scan | Checked against base Git blobs and alias hashes |
| `git diff --check` | Passed |

[validation.json](validation.json) records the documentation/evidence audit.
Source/test/scenario routes were checked against the repository and validation
commands against `.github/workflows/native-build.yml`, Cargo package metadata,
model commands and runner help.

## Limitations

Rust build/fmt/Clippy/tests and native staging/E2E were not run locally: this
container has no Cargo/rustc, and this change only edits documentation and moves
historical evidence. CI will evaluate those gates on the PR. Blender regeneration
and graphical/performance checks were not run; no new runtime screenshots apply
to a documentation-only task. Historical logs can still contain original output
paths; these are recorded past executions, not current command instructions.

The PR links #98 for closure on merge; the issue remains open during review.
