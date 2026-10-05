# Issue #95 — modular scout Blender authoring

## Outcome

Replaced the monolithic scout source with nine independently editable component
libraries and `assembly/salimon-scout.blend`. The final assembly links Collections
directly. `assembly/shared.blend` owns shared runtime hierarchy empties and the
13 material definitions/packed floor texture; component files link those through
portable relative paths. Small details and spatial markers/proxies stay with their
owning component. See the [ownership table and workflow](../../models/assets/ships/salimon-scout/README.md#open-edit-and-verify).

The manifest points to the final assembly. The existing adapter exports one GLB
to the same runtime path. The export report hashes the entire authoring dependency
set. No runtime Rust code or spatial layouts changed. Existing preservation and
ship validation contracts are unchanged. Export ordering and Blender exporter
version changed (original source was saved by a newer Blender; migrated sources
use the documented 4.5 LTS prerequisite), so interchange/GLB bytes differ.

## Validation

Environment: Linux x86_64, Python 3.12.14, Blender 4.5.3 LTS, Rust 1.99.0.
Commands ran from the repository root; Blender executable was
`/tmp/blender-4.5.3-linux-x64/blender`.

| Check | Result |
| --- | --- |
| `python models/assets/ships/salimon-scout/export.py --blender /tmp/blender-4.5.3-linux-x64/blender` | Passed shared budgets and ship-specific checks before publishing |
| `blender --background --python-exit-code 1 --python models/assets/ships/salimon-scout/verify_source.py` | Passed against old interchange before export and regenerated interchange after export: 123 objects, 5,890 triangles, 13 materials; geometry, hierarchy, extras, monitor UVs, anchors and packed images preserved |
| `blender --background --python-exit-code 1 --python models/assets/ships/salimon-scout/verify_modular.py` | Passed in relocated temporary copy: nine linked owners, portable dependencies, independent hull edit reaches assembly without changing other meshes |
| `python models/assets/ships/salimon-scout/validate.py` | Passed; 109 primitives, 481,908-byte GLB |
| `BLENDER=/tmp/blender-4.5.3-linux-x64/blender python -m unittest discover -s models/tests -v` | 41 passed, including real Blender export and modular/source verification |
| `python -m unittest discover -s scripts -p 'test_*.py'` | 34 passed |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | 236 passed; doc tests passed |
| `cargo build --workspace --locked` | Passed |
| `git diff --check` | Passed |

## Limits

No visual redesign was made. Native GPU screenshots/E2E and macOS/Windows checks
were not run; preservation evidence comes from saved source/interchange geometry
and runtime consumer tests, not a new graphical capture. Blender remains optional
for normal client builds. Reopen/reload the assembly after saving component edits;
keep linked ownership rather than appending/making components local there.

## PR update — conflicts and Windows CI (2026-10-05)

Merged main at `8acb48d` into the review branch. Resolved the model README
conflict by retaining both the modular scout workflow and the newly merged
Blender mining-tool documentation.

The original Native builds run `37285408626` passed Linux (including graphical
E2E and packaged smoke) and macOS, but Windows failed the authoring hash check:
`Path` string conversion produced backslash keys instead of the report's slash
keys. Export and provenance validation now use `as_posix()` for portable keys.
A focused regression uses `PureWindowsPath` to exercise Windows path semantics
on any host. Hash values and runtime artifacts are unchanged.

Post-merge local checks passed: 42 model tests with real Blender verification,
34 script tests, rustfmt, Clippy, workspace Rust tests and workspace build. The
updated three-platform workflow will run on the new PR commit; its status is
reported on GitHub. Native graphical suites were not rerun locally for this
path-format and documentation fix.
