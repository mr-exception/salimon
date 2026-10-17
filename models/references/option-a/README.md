# Option A reference slice

Original Salimon geometry/parametric Python under the repository license; no
external images or model services. The canonical [art bible](../../../docs/visual-style.md)
owns policy. This reference proposes palette/composition; **owner approval is
pending PR review**. Do not mark generated candidates as approved visual baselines.

```sh
blender --background --python-exit-code 1 --python models/references/option-a/build_reference.py -- --output artifacts/option-a
```

Pinned Blender 4.5.3 LTS, Cycles CPU, eight samples/seed 273, Standard transfer,
fixed exposure, neutral world and area lights. Camera locations/orientation and
unique geometry/material counts are emitted in `reference-costs.json`. All source
coordinates are meters, +Z up/+X forward. The saved editable `.blend` snapshots
are disposable outputs under `artifacts/`; the tracked Python recipe plus existing
asset sources reconstruct them. Ship links are localized **only in the preview**
so snapshots can be opened without owning production libraries. No runtime swap
or normal-build Blender dependency.

The exterior/far, seated console and Core views compare unchanged geometry under
current/proposed colors at identical camera/light settings. The tool uses the
existing authored source at two scales. Terrain, deposit and sphere closeups are
original editable parametric style studies, not runtime planet generation or a
replacement for the existing resource assets. Their scale/configuration is fixed
in the recipe; physical terrain remains world-owned. Sphere facets deliberately
retain spherical identity. Geological facets are flat; glass and live display
roles remain functional exceptions. The source monitor material is not live
runtime artwork. CPU renders do not establish shader or native screen readability.

Review [images and measurements](../../../reports/issue-274/README.md) for broad
forms at near/far scale, restrained accents, material contrast, geological facets,
console/window sightlines and clear lanes. Deliberately retain more detail on
monitor housings, grip, ramp and landing supports. The proposed ship pass does
not spend triangles on extra bevels; #276 applies the reviewed candidate behind
its own PR. Approve palette/composition and that PR explicitly before merging.
For production renders after an authored edit use `--production`; identical
cameras/lighting allow comparison to these candidates. GPU baseline approval
uses `scenarios/visual/README.md` and remains separate from these Blender previews.

The #274 images record the pre-migration sources in PR #281. To reconstruct that
historical comparison, use that PR's revision; rerunning on a later asset revision
intentionally previews the current sources. #276 production comparisons use the
same camera/light recipe, with before images preserved in its report. Neither
report promotes candidates into approved GPU baselines.
