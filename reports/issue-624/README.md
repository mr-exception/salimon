# #624 — rare biological and mixed natural sites

Parent: [PR #703](https://github.com/mr-exception/salimon/pull/703),
`issue-623-geological-landmarks`, `27d6d0db5166f980b8005593f2c404a514d57f0d`.
#611/#620 are merged; #623's actual terrain inventory, identity and checks are
included in this ancestry. #702 and #703 current-head CI passed before this delivery.

[Maintained biological signature contract](../../docs/biological-signature-sites.md)
records rare giant specimens, compact dense colonies and real geological/active
source intersections. Every site names actual existing physical source objects,
retaining exact roots, part geometry, species palette, ecological realm and
collision/exclusion ownership. Nothing spawns extra trees, adds collision,
relocates deposits, supplies rewards or grants player chart/scanner knowledge.
A dense colony needs at least two physical Colony specimens within **8 m**.
Mixed/geothermal provenance requires the same actual geological/climate sources
and spatial support; ancient volcanism alone cannot supply activity. No new
activity, heat simulation, growth, hydrology or owner-authored signature is invented.

Strict publication/reference binding reconstructs identically from original
biosphere/population/geology, rejecting changed or excluded sources, duplicates
and corrupt/trailing binding bytes. Existing population IDs/binding bytes are
preserved; a read-only upstream-biosphere getter supplies compatibility checks.
Canonical Vesper reconstructs the same empty source inventory without a mutable
save schema change. The live automation observation exposes only publication/count,
without automatically discovering sites. #621 retains future mutable harvesting.

## Actual verification

Debian 13 x86_64, Rust/Cargo 1.99.1, Python 3.12.14, Mesa 25.0.7 llvmpipe Vulkan.
Debug assertions enabled; workspace-local tooling. Positive biological sources
are declared fictional developed biospheres, not claims of life on the Moon.

- `cargo test -p salimon-world --locked natural_ -- --nocapture`: **4 passed**.
  [Audit](rarity-audit.log): **27 sites across 128 developed fictional worlds**,
  26 giants and one compact dense colony. These are conditional on already
  published developed life, not the prevalence across all planets. No quota.
  Exact physical bounds/reconstruction, authored mixed geological provenance,
  excluded/missing/duplicate sources, activity locality/dormancy and sterile
  Vesper pass. No positive naturally co-located geothermal world was selected
  for GPU capture; that branch is governed by explicit same-source/locality gates.
- `cargo test --workspace --locked`: **1205 passed**, **49 opt-in ignored**. Includes runtime
  comparison of every rendered source endpoint with its real physical part and
  reconstruction/quality invariants. No tolerance widening or source relocation.
- `cargo build --workspace --locked`, all-target workspace Clippy `-D warnings`,
  formatting, whitespace and `python scripts/check_docs.py --base
  issue-623-geological-landmarks`: passed.
  Script code is unchanged; 136 tooling tests passed earlier in this batch.
- With absolute `SALIMON_NATURAL_SITES_FILE`, `cargo test -p salimon-client
  --locked natural_site_production_export -- --ignored --nocapture`: passed.
  [Source JSON (gzip)](natural-sources.json.gz) retains actual world bytes,
  site/population bindings, physical IDs/bounds and runtime renderer segments.
- Feed that exact absolute file as `SALIMON_ORGANISMS_FILE`, set
  `SALIMON_ORGANISM_DIR`, `WGPU_BACKEND=vulkan` and workspace `lvp_icd.json`:
  `cargo test -p salimon-renderer --locked organism_production_reference --
  --ignored --nocapture`: passed. Two distinct [giant](natural-GiantOrganism.png)
  and [compact colony](natural-DenseColony.png) originals retain all **11/28**
  physical segments at both LODs. Cameras fit each actual source at physical
  scale; density/LOD never regenerates identity. [GPU log](gpu.log) records real
  software submit/readback cost, not hardware FPS or art approval.
- Production Xvfb/Mesa `natural-sites-canonical.json`: **10/10 passed** ([summary](native-summary.json)); current
  Vesper remains empty, repeated simulation retains source count and chart has
  no biological site/lead observations; the ordinary five-second scanner may legitimately discover Vesper. [Native original](sterile-natural-sites.png) and [hashes](media-sha256.txt). Parent #623 supplies the 20-step terrain streaming/revisit route and
  real geometric source/LOD evidence for mixed geological support.

Initial Clippy found an unnecessary clone in test setup, fixed before readiness.
Adding the observation inside the existing large JSON macro hit its recursion
limit; moving it to the normal independent observation assignment fixed build
without raising limits. The first native attempt ran the older binary after that
failed build and failed on the missing observation path; rebuilt runs pass.
First visual inspection showed that two organisms up to 100 m apart were too sparse
for a dense-colony label. The criterion was tightened to 8 m, then audit, source
export, GPU captures and affected/workspace gates rerun. The old 50-site audit
and sparse capture are superseded, not final evidence.

A later repeat crossed the ordinary five-second scanner boundary and correctly
observed Vesper. The test's perpetual-empty-chart assertion failed for legitimate
scanner behavior. The final scenario deliberately advances 400 frames and checks
exactly the real Vesper scanner body with no leads, while site count stays zero;
it does not suppress normal discovery or broaden site admission.

Originals are review candidates only. #561 retains subjective ecology/art and
combined positive-world presentation review, #557 physical input/platform, #563
actual hardware/M1/Metal budgets. These remain explicitly unverified. No software
render timing proves hardware FPS or full ecological simulation. No PR merged.
