# #751 — navigation browser screenshot regression

Implemented eight frozen full-viewport screenshot comparisons over the production
App and real Chromium/SwiftShader canvas, with pinned fixture fonts, strict pixel
policy, durable diagnostics and a real CSS-regression demonstration. **QA pending:
initial image references are candidates until explicit independent human baseline
approval.** The first CI screenshot run failed on host text rasterization; corrected-head CI
and final independent review remain pending
at report creation. No baseline has been silently approved or regenerated.

Branch `issue-751-navigation-visual-baselines` extends `main` at
`b0318abf9c9e1e71a349ff1948e074efb04092c7`; no active PR stack existed.
Current tested source: `0ce35101daa4c9a9b826db3af61b15aa1eef452d`.
Current candidate digest: `961065dae5acf782dc7b3991d33fd1916763532b48710333bcc26deda7951513`.
The previous `946d...` proposal is superseded; only the new digest can approve current images.
The original local campaign below used `46d6912`; the grayscale revision and
actual first CI failure are recorded in the addendum below.
Final publication head/base, remote CI and independent verdict are bound in the
PR conversation. #750's merged harness is reused. #224/#228 cover the historical
native renderer and are not duplicated by browser layout captures.

## Acceptance matrix

| Criterion / category | Required evidence and actual result |
| --- | --- |
| VIS-1 / AC1 detection | PASS for executable checks: eight frames pass two runs with **0 changed and 0 raw changed pixels** each. Actual velocity CSS changed from green to red in a disposable App: heading/velocity frame fails at **520 pixels**, opposite vectors at **630 pixels**; six unchanged controls pass exactly. Human acceptance of reference correctness is UNVERIFIED/pending. |
| VIS-2 / AC2 current evidence | PASS locally: exact committed-source repeat and mutation summaries, all state/environment observations, originals, diffs, success/failure traces and results retained through [index](evidence-index.json). CI publishes the same at its actual head/merge SHA; remote result remains pending. |
| VIS-3 / AC3 limits and approval | Hardware gaps labeled UNVERIFIED. Changed baseline manifest/image integrity and exact human collaborator approval gate implemented and tested. Actual initial manifest approval BLOCKED until a human independently approves the concrete images. |
| Unit/integration — mandatory | PASS: 30 domain/comparison/approval assertions; existing production runtime composition, malformed PNG/dimension rejection, exact digest, human role and Bot exclusion. |
| Build/typecheck — mandatory | PASS: production build and all source/test/visual/approval scripts typecheck. |
| E2E/visual — mandatory | PASS locally: 24 full browser routes, second eight-screen comparison, real CSS mutation, existing two navigation mutation scenarios. Required current-head CI still pending. |
| Manual/hardware | No physical input/device execution required for browser harness delivery. Discrete GPUs, Metal, native/Windows rendering, hardware FPS, mobile/touch and subjective art qualification remain UNVERIFIED. Human baseline decision is separately required and is not hardware testing. |

## Changes and ownership

- `client/tests/navigation-captures.spec.ts` runs empty scanner, four visible
  contacts, real D-key pan, selected moon, zoom 0.1/1, differing and opposite
  heading/velocity states. It freezes #750's seeded clock/camera and observes
  completed production frames without authoritative browser writes. Full canvas,
  SVG vectors and sidebar are captured; no masks or cropping.
- Vendored licensed DejaVu regular/bold fonts make the fixture's existing Inter
  family deterministic. Production entry and normal production/system-font routes
  remain isolated. Fonts, browser, Playwright, platform/architecture, backend,
  viewport/DPR, locale, seed, epoch and policy are versioned/hashed.
- `scripts/visual-comparison.ts` uses Pixelmatch color threshold 0.1, includes
  antialiasing, and permits zero over-threshold pixels. It retains raw mismatch
  counts, expected/actual/diff PNGs and hashes. Unexpected environment or renderer
  drift fails; candidate generation is explicit, separate and forbidden in CI.
- `negative-visual.mjs` mutates actual CSS in a disposable copy. It requires both
  intended failures plus six unchanged controls; startup errors/timeouts/unrelated
  failures are invalid demonstrations. Originals and traces remain available.
- `baseline-approval.ts` validates image hashes and requires an exact manifest
  digest approval comment from a human repository collaborator for changed
  references; unchanged base references inherit approval. Agents must not post
  that approval. Baseline approval remains separate from implementation QA.
- Navigation CI repeats comparison, proves the CSS defect, gates approval and
  uploads revision-named raw artifacts (14 days). Checkout fetches merge parents,
  rather than all historical objects. Token permissions are read-only.

The [reference guide](../../client/tests/navigation-baselines/README.md), client
guide, validation and maintenance map own capture, drift, proposal/approval,
threshold and retention procedures. There is no production gameplay change.

## Commands, revision binding and failures

Environment: managed Linux cloud, Debian 13 x64; Node 24.19.0 / npm 11.9.0;
Playwright 1.64.1 / lockfile Chromium 156.0.8078.4. Actual adapter is ANGLE Vulkan
SwiftShader (Subzero). CI uses Ubuntu 24.04 / Node22 and the same locked browser
and vendored fonts; cross-distro equality is validated by CI, not presumed.
Viewport 1280×800 / DPR1 / en-US / UTC / dark; seed750; fixed epoch
2026-11-04T00:00:00Z, controlled RAF/performance clock; synthetic browser input.

Commands from `client/`, browser installed with
`PLAYWRIGHT_BROWSERS_PATH=/tmp/salimon-playwright`:

| Command | Result / evidence |
| --- | --- |
| `npm ci --no-audit --no-fund --cache /tmp/salimon-npm-cache`; `npx playwright install chromium` | PASS after using writable `/tmp` browser cache; home cache is read-only. |
| `npm test` | PASS, eight test files; detailed count confirmed by next command. |
| `node --experimental-strip-types --test --experimental-test-isolation=none src/*.test.ts scripts/*.test.ts` | PASS, 30 assertions, no skips/failures ([log](final-unit.log)). Used to expose individual results where this host's isolated runner aggregates by file; an intentional failing runner probe was correctly rejected. |
| `npm run build` | PASS, including all scripts ([log](final-build.log)); existing chunk-size warning retained. |
| `SALIMON_EVIDENCE_DIR=test-results/full npm run test:browser` | PASS, 24 routes ([log](browser.log)). |
| `SALIMON_EVIDENCE_DIR=test-results/visual-repeat npm run test:visual` | PASS, eight images identical, exact committed source `ef4f49e` ([log](repeat.log)). |
| `npm run test:visual:negative` | PASS, two intended visual failures and six controls at `46d6912` ([log](negative.log); [raw mutation](evidence/visual-negative/mutation-summary.json)). |
| `npm run test:browser:negative` | PASS, both prior navigation mutations rejected ([summary](navigation-negative-summary.json), [log](navigation-negative.log)). |
| `SALIMON_BASE_SHA=b0318abf9c9e1e71a349ff1948e074efb04092c7 npm run test:visual:approval` | Expected BLOCKED: new images have no human approval ([log](approval.log)). This is not a passing required acceptance gate. |
| `SALIMON_BASE_SHA=46d691231548a6fde59ba056a8f9c125dc693a4e npm run test:visual:approval` | PASS supporting inherited-manifest branch ([log](inherited-approval.log)); this does not approve the initial images against real PR base. |

The initial full suite started with the complete working inputs just before their
source commit; its first screenshot metadata therefore reports base `b0318ab`.
This is disclosed, not relabeled as a new-head run. It used the same App/domain,
fonts, baselines, normal screenshot assertions and existing browser tests as the
committed source. The exact committed `ef4f49e` repeat independently proves all
pixels. [Input delta](tested-input-delta.patch) documents subsequent standalone
negative-driver fixes, workflow depth, scripts typecheck inclusion and empty
candidate-override normalization (normal run has undefined override). Final build,
30 unit assertions and actual CSS negative cover `46d6912`. The independent
reviewer assessed reuse as reasonable. [Source hashes](source.sha256) bind final
client/test/config/workflow bytes for report-only publication. Final-head CI must
still pass and is never inferred from these local results.

Text log/patch exports trim trailing whitespace; images, numerical JSON and retained
traces are unchanged. One redundant first-run success trace is replaced by the
exact committed-source repeat trace, as documented in the evidence index.

Iteration failures: first browser attempt preceded completed browser installation;
rerunning after installation passed. Initial independent review caught the planned
heading mutation affected all eight frames; it was corrected to velocity, which
appears in exactly two. The first negative execution then failed before capture
because an empty override selected mkdir(''); normalization fixed it, and the
actual CSS failure proof passed. [Failure log and summary](iteration-failure/command.log)
preserve that invalid attempt. No threshold/mask/reference was changed to hide a
failure. First sandboxed git fetch could not connect to proxy; the environment's
authorized network capability resolved it. No credential values were printed.

Repository rulesets returned empty; branch-protection read was denied 403 by the
integration, so administrative enforcement is UNVERIFIED. This work implements
checks, not new branch-protection settings. Required observed CI is still mandatory.

## Independent review and concrete baseline decision

Distinct reviewer agent `/root/review_751`, cycle1, independently inspected all
originals, diff, first/repeat summaries and real mutation results. Two actionable
findings (mutation target and empty candidate override) were fixed. No additional
code defects were identified before publication. Advisory image review found no
obvious new layout fault; it is not human baseline approval. Final exact-head
structured verdict follows in the PR after CI. Initial result remains QA pending,
with human approval and current-head CI vetoing ACCEPT.

Review all [eight originals](../../client/tests/navigation-baselines/README.md).
Manifest SHA-256:
`946d93ddef08b91e81eb29cf790389251058f4326148a85245f36e89069bdd60`.
A human OWNER/MEMBER/COLLABORATOR must post exactly on the PR:

```text
Approve navigation baselines: 946d93ddef08b91e81eb29cf790389251058f4326148a85245f36e89069bdd60
```

Then rerun the failed gate/job and obtain final independent acceptance. The
implementing agent will not post this approval. Keep #751 open before the
accepted delivering PR merges. No merge, deployment or release is authorized.


## CI rasterization diagnosis and revision (538a77b)

[First CI run41798103146](https://github.com/mr-exception/salimon/actions/runs/41798103146)
at published e038da8 / merge911afd1 passed units, build and23 existing browser
routes but failed all8 screenshot comparisons. Counts were6512/6449/6449/6098/
6449/6449/6425/6429, respectively. Its14-day
[artifact13306277199](https://github.com/mr-exception/salimon/actions/runs/41798103146/artifacts/13306277199)
was downloaded and essential originals, diffs, metadata, raw summary/results and
failure trace are retained via the updated evidence index. This was an actual
mandatory failure, not a passing run or a reason to regenerate references.

Independent byte/region analysis of heading/velocity found0 changed pixels in
the entire970-pixel-wide radar (canvas, bodies, vectors and composited overlay
labels), versus19796 raw changed pixels in the opaque sidebar. The original
[CI frame](ci-failure/heading-velocity.png) and
[diff](ci-failure/heading-velocity-diff.png) show glyph-edge color changes there.
The cloud host explicitly disables subpixel rendering in fontconfig; Ubuntu's
opaque text path selected LCD rasterization. Font bytes and layout were already
identical. Chromium now explicitly uses `--disable-lcd-text`, locking grayscale
rasterization across both hosts; the screenshot test requires that flag and
records actual launchArgs in each observation and summary.

**No image, manifest byte, threshold or mask changed.** The approval digest is
still946d93ddef08b91e81eb29cf790389251058f4326148a85245f36e89069bdd60.
This preserves the original reference proposal and makes the runtime reproduce
its rasterization, rather than approving or replacing a changed visual design.
The separate `baseline-approval` CI job requires the successful build/browser job;
a later human comment can rerun only that gate instead of repeating gameplay.

At exact committed source538a77b:

- `npm run build`: PASS ([log](grayscale-build.log)).
- `SALIMON_EVIDENCE_DIR=test-results/grayscale npm run test:visual`: PASS,8/8,
  zero changed and raw changed pixels ([log](grayscale.log)).
- `SALIMON_EVIDENCE_DIR=test-results/grayscale-repeat npm run test:visual`: PASS,
  another8/8 exact match ([log](grayscale-repeat.log)).
- `npm run test:visual:negative`: PASS, red velocity still fails the same two
  frames (520/630 pixels), six unchanged controls pass ([log](grayscale-negative.log)).
- Domain source/unit assertions are unchanged; their30-test result is reused.
  Current-head CI must still run all required gates, including full browser and
  prior navigation mutations, with the new launch setting. No CI pass is predicted.

The distinct reviewer rechecked the four-file source diff, CI originals, exact
regional counts, local repeat and negative evidence. No additional blocking code
finding; actual current-head CI and explicit human baseline approval remain
mandatory. Source hashes now bind538a77b inputs; the source delta from46d6912 is
[recorded separately](grayscale-input-delta.patch). Final head/base/CI/verdict are
bound after publication in the PR conversation. The user intends to review and
approve on GitHub; intent is not recorded approval and the gate remains required.


## Final font contract and corrected candidate proposal (0ce3510)

The second [CI run41821988777](https://github.com/mr-exception/salimon/actions/runs/41821988777)
at040a760 passed units/build and23 existing browser routes. Grayscale text removed
most mismatches, but seven captures still failed by467 pixels and selected moon
by129 pixels. The remaining differences were exclusively inside the native
select's displayed value; its hidden host-font fallback was not governed by the
page's custom webfont. CDP reported the custom page font but did not expose that
internal native-control dependency.

Local `fc-match Inter` resolved to a host-specific OpenAI Sans file; a minimal
Fontconfig policy resolving only vendored DejaVu produced **byte-identical hashes
to all eight Ubuntu outputs**. This proves the remaining drift was an unpinned
native-control font, rather than a canvas/layout defect. No host-specific font is
redistributed. The licensed vendored DejaVu files now serve that fallback through
screenshot-only `FONTCONFIG_FILE`; other browser/production projects retain host
font coverage. The policy pins relative font directory, cache location,
antialiasing, hinting/hintslight/autohint, rgba:none, LCDfilter and DPI75. Its bytes
are hashed in the environment manifest, which explicitly labels the native
fallback. The relative path works in disposable negative copies and CI.

This is an **explicit revision of unapproved candidates**, not automatic approval
or an attempt to hide a product defect. [Exact upgrade measurements](candidate-upgrade.json)
show only native-select glyph changes: seven frames467 perceptual/586 raw pixels
within x1008–1119,y542–553; selected moon129/169 within x1009–1044,y546–553.
There are ZERO changed pixels elsewhere, including every body, vector, camera,
selection marker, label and layout. The initial originals/manifest are permanently
retained under [initial-candidates/](initial-candidates/manifest.json); prior
reference-index links point to those exact preserved bytes. Old source/evidence
and actual failures remain disclosed. Threshold0.1, zero accepted pixels and no
masks remain unchanged.

At exact source0ce3510, build/typecheck passes, both eight-screen runs match all
reference bytes, and the real velocity-color mutation still fails at520/630
pixels with six unchanged controls. Commands and logs:

- `npm run build`: PASS ([log](pinned-build.log)).
- `SALIMON_EVIDENCE_DIR=test-results/pinned npm run test:visual`: PASS ([log](pinned.log)).
- `SALIMON_EVIDENCE_DIR=test-results/pinned-repeat npm run test:visual`: PASS ([log](pinned-repeat.log)).
- `npm run test:visual:negative`: PASS ([log](pinned-negative.log)).

Updated raw summaries/state, original captures/diffs and current-source
success/negative traces are indexed. The repeated success trace is redundant; the
intermediate second-CI trace remains in14-day
[artifact13318325055](https://github.com/mr-exception/salimon/actions/runs/41821988777/artifacts/13318325055),
while its essential originals, state and raw measurements are retained here.
Unchanged30 unit assertions are reused; final-head CI must rerun all affected
cloud gates, including full browser and prior navigation mutations. No CI pass
is predicted from local evidence or the earlier failures.

The distinct reviewer independently verified every new image hash, all eight
pixel-change bounds, zero changes outside native select, the policy/source diff,
and representative corrected originals. Advisory review preserves text semantics
and readability; it **does not approve the baseline**. Required current-head CI
and explicit human approval still veto acceptance.

The new manifest digest is
`961065dae5acf782dc7b3991d33fd1916763532b48710333bcc26deda7951513`.
The previous digest946d... is INVALID for the corrected candidate set, even if an
old approval comment appears. A human collaborator must review the corrected
originals and post exactly onPR755:

```text
Approve navigation baselines: 961065dae5acf782dc7b3991d33fd1916763532b48710333bcc26deda7951513
```

Final publication head/base, actual CI and structured independent verdict are
bound in the PR conversation. The issue remains open and PR draft until mandatory
approval/checks pass. No merge/deployment/release is authorized.
