# Issue #51 validation

The resource collection suite is documented in [scripts/README.md](../../scripts/README.md#resource-collection-suite-51).

[Native CI run 36904311961](https://github.com/mr-exception/salimon/actions/runs/36904311961)
on `77d7c3433336eba488b54f3f796f9555ec1b7b7c` passed Rust formatting, Clippy,
workspace tests, Python contracts and both staged builds on Linux, macOS and
Windows. Linux also passed all 24 native baseline/evidence routes, including
all six required resource evidence scenarios. The uploaded Linux artifact
contains their structured results, state snapshots, logs and screenshots.
[Extracted results](prior-ci-results.json) record the exact route outcomes.

That run failed the subsequent packaged black-box smoke: two OS F2 commands
produced three view transitions (PrecisionTour, Gameplay, PrecisionTour), then
the runner timed out waiting for Gameplay. The log establishes the extra toggle;
focus replay is the suspected cause. Winit synthesizes held-key presses on focus
gain, which previously passed the F2 action guard.

The follow-up rejects synthetic F2 presses, preserves genuine initial presses,
and avoids redundant X11 refocusing in the smoke helper. Native key injection
uses an explicit 80 ms delay. Regression tests cover focus replay and both
already-focused/new-focus helper paths. The smoke still requires the real
PrecisionTour/Gameplay acknowledgements and screenshots; no gate is skipped.

Local validation: all 34 Python contract tests and `git diff --check` passed.
Rust/native validation of the follow-up is delegated to the required CI job;
this workspace has no Rust toolchain or X11 display. Issue #51 must remain open
until the follow-up Rust, resource E2E and packaged smoke gates pass.

## Follow-up native result

Run [36960052701](https://github.com/mr-exception/salimon/actions/runs/36960052701)
on `be6cba56944685dd5934a0de91a01078f037a79a` passed all Rust/contract/build gates
on all three platforms and the full Linux native E2E step. Packaged smoke failed
before sending its first key: `xdotool getwindowfocus` tried to resolve WM_CLASS
on X11 PointerRoot (window 1), causing BadWindow in the WM-free Xvfb session.

The next correction uses documented `getwindowfocus -f` to read the actual X11
focus without top-level-window lookup. PointerRoot now triggers normal focusing
of the PID-matched game window. The helper contract includes PointerRoot, an
unrelated focused window and the already-focused game. Unexpected helper errors
still fail the smoke; this does not suppress input failures.
