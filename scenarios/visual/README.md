# Production model image regression

Run from repository root, on a dedicated desktop with a fixed 1280×800 viewport,
same driver/backend, build, seed and pixel density. Fixtures reuse maintained
production scenarios, seed and fixed-step actions. Screenshots use the explicit
wgpu surface copy (not a desktop screenshot or Blender rendering). Repeats freeze
simulation while continuing presentation. The moving fixture advances camera
pose between checkpoints; comparisons use matching poses, never consecutive
moving images. Other views cover lower glazing/glass edges, exterior, monitor,
Core, tool/deposit/fragment, contact piles and floor/door/ramp.

```
python -m pip install -r scripts/visual-requirements.txt
python scripts/model_visual_tests.py capture --binary target/debug/salimon-client
python scripts/model_visual_tests.py compare --temporal-only
python scripts/model_visual_tests.py compare --reviewed --baselines scenarios/visual/baselines/software-vulkan-option-a-v2
```

Linux software Vulkan: prefix capture with
`xvfb-run -a -s '-screen 0 1280x800x24'` and set `WGPU_BACKEND=vulkan`.
Nightly verification runs software-Vulkan captures, repeat stability and reviewed appearance comparison.
These commands also remain available locally; PR CI only builds source. Missing approved baselines
fail ordinary comparison, never silently initialize or overwrite a baseline.
Both comparison modes derive the exact expected PNG names from each fixture's
source scenario and repeat count. Missing poses/repeats and unexpected stale PNGs
fail before pixel comparison, including with `--temporal-only`. Screenshot names
must be unique file names; moving fixtures require at least two repeats per pose,
and static fixtures require at least two frames in total. Capture and comparison
share the same expansion rule, so deleting evidence cannot weaken the gate.
Artifacts contain PNG candidates, per-step production inspection/protocol logs,
highlight/difference PNGs, changed/compared pixel counts and peak channel delta.

Default tolerance: RGB channel delta >8 counts as changed, at most 0.1% of
unmasked pixels. This allows minor raster noise, not major color/silhouette
changes. Absolute pixel masks `[left,top,right,bottom]` exclude intentional
animation (or a changing HUD) only after review. Empty comparison regions fail.
Flicker checks compare each frozen repeat with its first frame; deterministic
intentional motion belongs between poses. Tolerances are backend-specific
starting points and must be calibrated with real captures rather than inflated
to conceal failures. Size changes fail explicitly.

## Approval policy and style replacement

`baselines/software-vulkan-option-a-v1` and `baselines/metal-option-a-v1`
are **independent** versioned corpora. The software-Vulkan corpus was approved
by the repository owner on 2026-10-19 after PR #335; Metal approval remains pending. See the [software capture report](../../reports/issue-321/README.md)
and [checkpoint index](../../reports/issue-321/checkpoints.md). Software approval is tracked in
[#321](https://github.com/mr-exception/salimon/issues/321); native M1/Metal review
remains [#314](https://github.com/mr-exception/salimon/issues/314).

Nightly now selects `baselines/software-vulkan-option-a-v2`, explicitly pending
[review #518](https://github.com/mr-exception/salimon/issues/518). Engine #493
changed the cockpit fixture's flight timing, and hosted Mesa changed since v1.
The old approved v1 corpus remains untouched; its approval does not transfer to
new actions or renderer contexts. Capture complete candidates after the repair
merges and use the separate review process below before claiming appearance
coverage. See [nightly transition](../../docs/periodic-ci.md#current-corpus-transition-2026-10-22).

Capture only from a clean tracked source revision. Each capture writes a
`corpus.json` candidate record with actual runtime backend/adapter/driver,
physical viewport pixels and DPI scale, OS, source revision and binary SHA-256.
The fixture-contract hash covers source actions/assertions, seed, repeat counts,
tolerances and masks. Per-image SHA-256 records detect replaced/missing images.
All fixture screenshots must agree on renderer context and dimensions. The
revision and binary hashes provide provenance; comparing a newer revision is
allowed, while context and fixture-contract changes require independent review.
Runner diagnostics are retained beside the corpus: the default candidate images
are in `artifacts/model-visual/candidates/<fixture>/`, and full runner logs and
duplicate raw screenshots are in `artifacts/model-visual/candidates-runs/<fixture>/`.
Only contracted checkpoint PNGs belong inside a corpus; unexpected images still fail.

To review a new corpus:

1. Keep the candidate directory and run logs as evidence; inspect every PNG for
   coherent low-poly silhouettes, material/color, transparency, scale and layout.
2. Review the exact fixture tolerances and pixel masks. Masks must exclude only
   reviewed intentional animation/UI; never mask defects or promote a repeatable
   but wrong appearance. Fixture/mask/tolerance changes invalidate old approval.
3. In a **separate baseline review PR**, manually copy the selected fixture PNGs
   and candidate `corpus.json` into the intended versioned corpus. Preserve all
   provenance and image hashes; add `corpus_id`, set `status` to `approved` and
   add a `review` object containing nonempty `reviewer`, `reviewed_at`, `evidence`
   (review link/notes) and `masks_and_tolerances_reviewed: true`. Human approval is
   required for the images themselves and these tolerance/mask decisions.
4. Run `compare --reviewed --baselines <that-corpus>`, retain diffs and
   `appearance.json`, and have the reviewer approve that PR. No command promotes
   or overwrites approved images. New backend/adapter/driver/OS/viewport/DPI or
   art direction uses an independently reviewed corpus, never an automatic reset.

`--reviewed` exits **0** for passed approved appearance, **1** for broken capture,
metadata/incompatible context or a pixel regression, and **3** for missing or
pending/unapproved baseline coverage. Pending does not pass appearance. Temporal
completeness/stability still must pass before a pending outcome can be emitted.
Nightly reports show a `pending` lane when every executed check passed but
appearance approval is pending; a real failure still takes priority. Download
`artifacts/model-visual/diffs/appearance.json` and the candidates for details.
Source-only PR CI is unchanged. A software-Vulkan approval never certifies Metal.

An Option B trial uses a different corpus via `--baselines`, preserving the
fixture gameplay assertions. Intentional replacements require a new reviewed
commit. The tooling tests inject persistent silhouette, material and layout
changes: temporal stability passes but appearance comparison fails. Synthetic
fixtures test the comparator, not production aesthetic approval.

### v2 candidate evidence (#518)

[Review packet and every-frame index](../../reports/issue-518/README.md) retain
128 current-action local software-Vulkan candidates and zero-change temporal
results. v2 is `candidate`, with exact observed Linux/Mesa context, awaiting
human review. These captures are from the cloud executor; hosted nightly
context/captures and subsequent approved comparison remain required. Never
approve this local context as a substitute for a different hosted driver/OS.
