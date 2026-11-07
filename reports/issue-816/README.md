# #816 — Panoramic browser acceptance

The integrated panoramic cockpit now has a 19-screen reference matrix in the
existing Playwright `navigation-captures` project, alongside the unchanged eight
legacy references. New adversarial checks reject unscanned optical markers and
stale marker clicks. Telescope stress retains actual rendered zoom states and
raw GPU pixels; the negative drivers prove both a real optical marker leak and
an expected-image mutation fail their named assertions.

**QA pending:** local functional coverage is complete after the recorded optical
timeout correction. Required CI awaits the owner's report in the active chat;
new references await explicit human approval. No baseline was approved, issue
closed, PR merged or release qualified by this work. The delivering PR and final
head/base binding are linked from [issue #816](https://github.com/mr-exception/salimon/issues/816).

## Revisions and evidence reuse

- Parent: `main`, `15a6e73c36f6ac151434e583eb4d65d5d1c35871`. No active PRs at
  preflight. #809 and #810–#815 are closed/merged ancestors with required code
  present; formal blocked-by metadata is empty.
- Full 64-case browser run: `8e6666018bf8962b0256d7bc4e40589ee7f1e82a`, client tree
  `3ef20e026c860fd7c4f6c2d5ad27addfe47146c4`: **63 PASS, one OPT-1 timeout,
  zero skips**. This is not reported as a green full invocation.
- Corrected OPT-1 and rendered stress: `b7224b7fb8375cf532537f55efa14bb3c4a4f4f4`, both PASS. The sole OPT-1
  change gives four full-resolution software-WebGL boots 240 s instead of 120 s;
  every older optical/scanner/ship assertion remains intact.
- Final stress assertion and visual/deterministic repeat:
  `cb5b87a5c859f50802bb45bc47abf00dbfb672b9`, **7 PASS, zero skips**.
- Final source: `a63c90e06cdaf8bb03f178b9cbcda13a3e884dff`, client tree
  `572e01f01e2f8c89935e86633cd7330e628faa8b`. Its changes after the successful
  repeat only record actual per-capture viewport/DPR and strengthen the negative
  oracle to require the actual telescope image to match the original hash.
  Negative commands execute this revision. No rendering, physics, interaction,
  fixture geometry, dependency, font, tolerance or positive assertion changed.
- Production `client/src` tree is `5cdf0059b3d4eb8586b825ca7d009e71b780c799`,
  unchanged from the parent. Domain and production-bundle inputs are unchanged;
  the 157-test domain run and Vite build were executed before the first source
  commit and reused on that verified basis. Final strict `tsc -b` passes.
  Earlier browser cases remain applicable: later fixture changes add read-only
  origin metadata and a stress-only physical one-metre target/readback, leaving
  normal fixture geometry and production behavior unchanged. All affected stress,
  optical and capture paths were rechecked. The final report commit changes no
  client/test/configuration inputs.

## Criterion matrix

Exact commands and revision stages: [command record](evidence/commands.txt).
Tool-native results, originals, raw measurements, diffs and traces remain in the
archives; `results.json` indexes attachments, including inline original PNG/JSON
bodies. Archive paths retain the Playwright test names.

| ID / category | Local result / evidence |
| --- | --- |
| C1 — unit, integration, build and older regressions | PASS: `npm ci`, 157 domain tests, build and final strict typecheck. Full browser command executed all 64 cases; OPT-1's timeout was corrected and its complete case passed the focused recheck. No required case is skipped or left failing. [Domain](evidence/domain.log), [build](evidence/build.log), [full browser](evidence/browser-full.tar.gz), [recheck](evidence/focused-recheck.tar.gz). |
| C2 — optical/scanner/HUD visual matrix | PASS: all 27 comparisons twice under the existing policy: threshold 0.1, max differing pixels 0, AA included, no masks. New [19 originals and case index](../../client/tests/navigation-baselines/README.md#panoramic-candidates-816) cover wide/telescope, huge origin, empty/acquired/lost scanner, occluded/rear contacts, overlap/pagination, horizon, actions, compass and small warning/terminal states. [Repeat](evidence/repeat.tar.gz). |
| C3 — selection leakage and invalid actions | PASS: SENSOR-3 checks unscanned optical clicks, foreground selection, missing hidden/rear/outside markers and a stale captured target after range loss. LOOK/HUD/default cockpit cases preserve grounded and stale-contact command guards. Controlled source mutation leaks catalog markers before scanning and fails `UNSCANNED_OPTICAL_MARKER_LEAK`; older reversed-WASD and heading-color mutations also fail their exact assertions. [Negative navigation](evidence/negative-navigation.tar.gz). |
| C4 — precision, depth, motion and authority | PASS: OPT-3 renders identical angular scenes at physical depths from 0.01 m to 1e24 m and byte-identical 0.1° optics after enormous split-coordinate translation. SENSOR-4 records 90 distinct completed-buffer frame IDs with raw 24×24 RGBA regions, GL errors 0, minimum peak RGB sum 255 and minimum 550 lit pixels; rendered FOV spans 0.1–0.1010050167084168°. Keyboard motion is non-vacuous; endpoint canvas PNGs are byte-identical, camera/markers finite, scanner tick/contacts/origin/actions unchanged. OBS-1–5 cover spherical turns, cancellation, independent ship controls and moving shared origin. [Repeat raw evidence](evidence/repeat.tar.gz). |
| C5 — keyboard/accessibility and layout | PASS: HUD-1–3, OBS, desktop Help and viewport checks cover Tab/Enter/search, Escape priority/focus return, typing/blur/resize cancellation, warning/terminal priority, 960×600 and 1280 desktop layouts, DPR 1/2 and no body overflow. Essential HUD controls are in the viewport and hit-tested against overlay obstruction. [Full browser](evidence/browser-full.tar.gz). |
| C6 — production journey and power | PASS: COCKPIT-1 and SLICE-1 each replay in two fresh contexts, including built default outside→inside→cockpit→outside, correct Vesper horizon, shared anchor, camera/FOV persistence, standing voluntary exit, seated blackout, explicit restoration and no relocation/extra clock. Existing collision, gathering, inventory, storage, survival and save/error routes pass. [Full originals/state](evidence/browser-full.tar.gz). |
| C7 — repeat and negative visual sensitivity | PASS: both completed visual-negative components, with the interrupted wrapper retained. Exactly 10,000 telescope pixels fail after the expected-image mutation. Four NAV scenarios and all 27 visual comparisons ran twice. The expected telescope image is mutated only in a disposable copy, with its integrity hash updated, so the named pixel assertion must fail while the actual screenshot retains the original hash; older red-velocity mutation still fails only its two intended comparisons. [Negative visual originals/diffs/traces](evidence/negative-visual.tar.gz). |
| C8 — manual / hardware qualification | UNVERIFIED: human approval of the new manifest, physical input/platform behavior (#720), subjective appearance (#721) and hardware/Metal budgets (#722). Software WebGL is not target-device GPU/Metal 60 FPS evidence. No hardware FPS or GPU-time measurements were collected. |

Raw pixel equality is **not** claimed for every panoramic case. Against the
candidate reference, the two small HUD cases have 11 raw changed pixels each in
the full run and 60 each in the repeat; all are below the unchanged perceptual
threshold (zero policy-counted differences). Their raw counts/diffs remain in the
summaries. The other 17 panoramic cases and all eight older references are
byte-identical in the repeat. No tolerance or mask was changed to obtain passes.

## Approval and review

The [shared manifest](../../client/tests/navigation-baselines/manifest.json) SHA-256 is
`83a51c49e9d88248dec0be8bf65af83b8eb9d20b0c6b44f791d4b7460c7ca369`.
The existing approval command correctly rejects missing human approval; CI and
in-place candidate generation also fail at collection with their expected guards.
Future proposals must use an absolute external output directory, preserve the
other reference family and pass the same human approval workflow. Only a human
may post `Approve navigation baselines: <manifest-sha256>` after reviewing the
originals. No approval comment was posted by the implementing agent.

Same-session review by the implementing Codex agent, without separate-agent or
independent-execution claims: reviewed the issue/dependency closure, final source
diff, every comparison summary and test outcome, original telescope/horizon,
contact-group/compass/warning captures, the production horizon/blackout originals,
and the GPU stress original/raw readbacks. Local findings were corrected without
weakening production assertions or baseline policy. Verdict: **QA pending**;
local functional criteria pass, CI and human baseline approval remain UNVERIFIED.
The final published head/base and evidence reuse are bound in the PR comment.

The aggregate visual-negative command was interrupted by a workspace reconnect
after its older component passed. Its unfinished panoramic child has no verdict
and is retained as interrupted; only that child is resumed against unchanged
source. The first resumption was rejected as INVALID_DEMONSTRATION because owned Vite
servers from the interrupted copy still held its ports. Their exact test paths
were verified, those servers were terminated, and the unchanged panoramic child
reran successfully. Both completed component results are required, never
inferred from either startup failure or interruption.

Iteration evidence preserves the startup connection refusal caused by overlapping
server lifetimes, the initial warning capture without a pulse, keyboard-focus
pixel decoration, and optical timeout failures. The warning setup now acquires
contacts and asserts an actual warning; stress originals use consistent keyboard
focus and settle before comparison. Strict typing also caught an untyped sample
array during the readback extension; it was annotated before the final typecheck.
[Iteration originals/results/traces](evidence/iterations.tar.gz).

## Environment additions

Managed Linux x64 / Debian 13; AMD EPYC 9V74 virtual CPU, 3 visible CPUs with a
2-CPU quota. Chromium 156.0.8078.4 / Playwright 1.64.1. Actual adapter:
`ANGLE (Google, Vulkan 1.3.0 (SwiftShader Device (Subzero) (0x0000C0DE)), SwiftShader driver)`.
Captures use DPR 1 and vendored DejaVu/Fontconfig; observation/viewport checks also
exercise simulated DPR 2. Fixed clocks and synthetic Playwright keyboard/pointer
and DOM wheel input are recorded in the fixtures. Power interruption uses the
existing test control; the built normal journey uses the ordinary production entry.

Actual reusable additions: Playwright Chromium and headless shell **v1248**
(156.0.8078.4), plus bundled FFmpeg **v1013**, installed successfully with
`npx playwright install chromium` from `client` after locked project setup.
Cache: `/home/agent/.cache/ms-playwright`. No system libraries, Node/npm or saved
environment template were installed/upgraded/changed. Preinstalled/reused:
Node 24.19.0, npm 11.9.0 and existing system libraries.
Project setup is separate: `cd client && npm ci --no-audit --no-fund` (98 locked
packages). CI runner installation remains owned by the existing workflow.
