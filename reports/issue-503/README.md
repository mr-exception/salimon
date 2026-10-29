# Issue #503 — Scout scanner readability

The physical scanner now uses the shared Scout face and functional palette.
The previous long single-line hint becomes three bounded rows, with explicit
FRESH / NO FRESH CONTACTS / OFFLINE states, sampled center distance in km and
age in s, bracketed M/P affordances and a logarithmic range notice. Invalid
readings show `--`, large readings use scientific notation, and contact labels
are sanitized/bounded. Selected amber geometry is also larger; state is never
communicated by color alone. The prompt raster now preserves the shared face's
lowercase SI glyphs instead of silently uppercasing units.

Renderer owns `ScannerReadout`, functional sRGB-to-linear projection inks and
shared bitmap glyphs. Runtime maps only current authoritative same-frame
scanner observations and bus state to the existing action bar. No second
HUD, detection/expiry/cadence, source state, input admission, save schema,
geometry placement or collision rule is introduced. Chart history retains its
existing explicit provenance. See [Scout presentation](../../docs/scout-presentation.md),
[foundation #466 report](../issue-466/README.md) and
[foundation PR #504](https://github.com/mr-exception/salimon/pull/504).

## Revision and environment

Parent: `main` at `9ec1aadc494066c874ab3735703131136494ffa9`; no active PRs at
selection. #430/#466/#429/#431 are closed and their implementations are on this
ancestry. This is the first branch in the two-issue sequence; #438 follows it.
Cloud Debian 13 x86_64, Python 3.12.14, Rust 1.99.1, Mesa 25.0.7 llvmpipe Vulkan,
Xvfb :77, 1280x800 window at scale 1. The 1920x1080 virtual desktop is not the
native GPU capture size. Protocol `canonical-flight` explicitly starts in the
canonical cockpit; this is not a measured ordinary boarding route.

Rust and Debian graphics/development packages were installed under `/workspace`
without changing system files. The initial apt install lacked root permissions;
workspace-local apt indexes/downloads and extracted packages resolved it. The
first link failed because the extracted ALSA development symlink had no local
runtime target; binding it to the existing system library fixed linking. Git
requires the execution tool's network permission to use the configured proxy.
No credentials were read, printed or replaced.

## Verification

- `cargo build --workspace --locked`: passed.
- `cargo test --workspace --locked`: 1192 passed, 0 failed, 46 existing opt-in ignored.
- Final `cargo test -p salimon-client --locked scanner`: 8 passed, including source/power, malformed persistence, input gates and bounded readout.
- `cargo test -p salimon-client --locked action_bar`: 12 passed, one existing evidence writer ignored; SI glyph regression passed.
- `cargo test -p salimon-renderer --locked`: 86 passed, 20 existing opt-in GPU/evidence checks ignored. Production captures below provide actual GPU verification.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed after correcting the new pixel assertion to the required slice `contains` form. Initial chunks/iterator compile and lint failures were corrected; no lint was disabled.
- `cargo fmt --all -- --check`, `git diff --check`: passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 134 passed; negative-path fixture error messages are expected assertions.
- `python3 scripts/check_docs.py --base origin/main`: zero failures.
- `PYTHONPATH=scripts python3` with `salimon_test.run('scenarios/evidence/scanner-hologram.json', ['target/debug/salimon-client'], artifact_root='artifacts/issue-503-final', screenshot_command=['native'])`: 28/28 passed; four original GPU captures at 1280x800, scale 1. `DISPLAY=:77`, `LP_NUM_THREADS=2`, workspace-local Vulkan ICD/library paths.

Original native GPU captures:

- [No discovery](undiscovered.png)
- [Fresh selected contact and unit labels](identified.png)
- [Existing cockpit monitor coexistence](cockpit.png)
- [Real Core blackout](blackout.png)

[Capture context and hashes](capture-context.json), compressed result/protocol/
stderr files and check logs are retained beside this report. Captures use the
production wgpu color surface (`screenshot_command=['native']`), not a fixture
raster or OS desktop. No image baseline, mask or tolerance was promoted.

## Corrected checks and limitations

The initial 28-step route passed, but used the runner's default OS screenshot
capture and exposed uppercased unit glyphs. Those images were preliminary;
`preliminary-result.json.gz` retains its actual outcomes. The glyph regression
and final native GPU recapture replace that evidence. Vulkan adapter operation
is verified; fallback EGL initialization errors are not a passing GL claim.

#438 owns the broader cloud integration/viewport matrix. The former #502
readability follow-up is preserved through #561 subjective appearance/readability,
#557 physical DPI/input/OS behavior, and #563 actual hardware/M1/Metal budgets.
These qualifications remain **unverified**. Software timings and pixel captures
are not hardware FPS, physical DPI, listening or human accessibility approval.
No merge was performed; source issue closure belongs to reviewed PR merge.
