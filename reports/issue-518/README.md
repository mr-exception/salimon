# #518 — replacement software-Vulkan candidate review

2026-10-22. Candidate evidence delivered for review; **appearance approval and
hosted nightly acceptance remain pending**. This is a baseline review PR, not
approval of its images. Keep it draft until those requirements are resolved.

## Evidence and provenance

Built and captured clean tracked `f2fa220385160f9e46571f9dcf27eb3f2465557e`
(latest main, after merged #519) with the production debug client, locked Cargo
inputs, Xvfb 1280×800 and `LP_NUM_THREADS=2`. Actual context is Linux glibc 2.41,
llvmpipe LLVM 19.1.7 / Mesa 25.0.7-2+deb13u1, Vulkan, physical 1280×800, DPI 1.
This is the local cloud executor, **not a GitHub-hosted runner capture**.

- [Unmodified candidate provenance](candidate-corpus.json): source/binary/fixture
  and per-image SHA-256; all 128 images / 46 checkpoints / nine groups.
- [Every frame and contact sheets](checkpoints.md); originals are submitted in
  [v2](../../scenarios/visual/baselines/software-vulkan-option-a-v2/corpus.json)
  with `status: candidate`. No human review fields are invented. Approved v1
  is untouched. The pending v2 descriptor's renderer scope is now the exact
  observed candidate context; this does not approve it for hosted use.
- [Complete runner diagnostics](runner-logs.tar.gz): scenario/step/result JSON,
  native process logs and expanded fixture actions; raw duplicate PNGs are omitted
  because the exact contracted PNG bytes are in v2. Internal absolute paths are
  original capture provenance, not portable reproduction paths.
- [Build log](build.log), [capture log](capture.log),
  [temporal log](temporal.log), [appearance log](appearance.log), and retained
  [comparison summaries/diffs](comparison/).

## Checks and inspection

`cargo build -p salimon-client --locked` passed (Rust 1.99.1).
`model_visual_tests.py capture --binary target/debug/salimon-client` passed.
`compare --temporal-only` passed all nine groups, **86 comparisons, zero changed
pixels**, with unchanged channel tolerance 8/255, fraction 0.001 and masks `[]`.
`compare --reviewed` returned **3 / candidate pending approval**, as required.
No approved appearance result or approved baseline pixel comparison is claimed.

Author inspected all 128 frames through nine contact sheets: cockpit glazing and
console, landed/approaching instruments, tool/deposit progression, stable contact
pile, static/moving cabin, exterior hull, Core face and door/ramp poses. Repeats
are visually consistent; no mask/tolerance expansion or image substitution was
made. Thumbnail review cannot certify native readability; originals are linked
for explicit owner image review. No gameplay/shader/asset change was made.

## Remaining acceptance before approval/merge

1. Force Periodic verification on current main (`suite=nightly`, `force=true`)
   and retain its exact-revision candidates, provenance, logs and temporal result.
   CLI Actions access returned `Forbidden` here; no dispatch/run is claimed.
2. Compare the actual hosted context with this candidate's context. If different,
   replace these **unapproved** candidates with that complete hosted corpus and
   preserve its hashes/logs. Never reuse approval across driver/OS differences.
3. Human reviewer inspects every original image, all empty masks and unchanged
   limits; record explicit reviewer/date/evidence only after approval as specified
   in the [review policy](../../scenarios/visual/README.md#approval-policy-and-style-replacement).
4. Retain approved `compare --reviewed` appearance.json and all diffs, then verify
   a subsequent full nightly. Metal/hardware remains independent.

#518 retains these requirements while this PR is draft; it must not close on a
candidate-only merge. No acceptance is transferred or waived by this report.

## Documentation impact

Evidence-only baseline proposal. Renderer/runtime ownership, architecture,
invariants, fixtures, gameplay, tolerances and masks are unchanged. The visual
README links this review packet and records its current local-context limitation.
Local Markdown paths/anchors and corpus image/contract hashes were checked.
