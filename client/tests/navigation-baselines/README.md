# Navigation visual references (#751)

These eight original #751 images are retained visual references.
The initial proposal exposed a hidden native-select host-font dependency. The
current proposal pins Fontconfig to the licensed vendored DejaVu files; the only
image changes are the selected-value glyphs. See the [upgrade measurements](../../../reports/issue-751/candidate-upgrade.json)
and [original candidates](../../../reports/issue-751/initial-candidates/manifest.json).
Pixel equality is separate from subjective acceptance. At the owner's request in
PR #834, CI no longer requires a human approval comment or manifest digest approval.
The owner reviews the deployed version and files separate issues for visual
problems. Automated functional, reference, repeat and negative checks remain required.

| Original | Observable case |
| --- | --- |
| [Empty scanner](empty-scanner.png) | Seeded universe before the first scan, no catalog leak |
| [Visible contacts](visible-contacts.png) | Four distinct faceted bodies, fixed front camera |
| [Panning](panned-contacts.png) | Real D-key camera movement, bodies retained |
| [Selected body](selected-body.png) | Moon picker selection and selection geometry |
| [Zoom out](zoom-out.png) | Fixed zoom 0.1, linear world projection |
| [Zoom in](zoom-in.png) | Fixed zoom 1, readable proxies |
| [Heading/velocity](heading-velocity.png) | Yellow heading right, green dashed velocity up |
| [Opposite vectors](opposite-vectors.png) | Heading and velocity reach opposite edges |

Full 1280×800 viewport images include the actual WebGL canvas, DOM vector overlay
and sidebar layout. No masks, cropping, dynamic substitutions or CSS geometry
changes are applied. Vendored, licensed DejaVu fonts supply the fixture's `Inter`
family to avoid host fallback changes. Ordinary production and browser tests still
exercise system fonts; this fixture CSS does not ship in the production entry.

## Compare and retain evidence

```sh
npm ci
npx playwright install --with-deps chromium
npm run build
npm run test:visual
SALIMON_EVIDENCE_DIR=test-results/visual-repeat npm run test:visual
npm run test:visual:negative
```

Use the lockfile Chromium on Linux x64; do not set `CHROMIUM_PATH` for reference
qualification. Environment drift (browser/Playwright/platform/architecture/font
hashes/renderer) fails explicitly. Chromium explicitly disables LCD text
(`--disable-lcd-text`) so opaque sidebar glyphs use the same grayscale
rasterization on Debian and Ubuntu. Actual launch flags are recorded per capture. A minimal, hashed
`tests/assets/fonts.conf` also pins native-control fallback, hinting, antialiasing,
DPI and subpixel policy. It loads only vendored fonts and is scoped to the
screenshot project; ordinary production/browser routes retain host font coverage. Host distro/version is recorded in the issue
report/CI runner rather than asserting Debian and Ubuntu are the same OS.
Seed 750, fixture front camera, fixed zooms, DPR 1, en-US/UTC, dark scheme and
paused RAF/performance clock are recorded in `manifest.json` and observations.
The capture waits for fonts and completed production frames. Screenshot capture
never fast-forwards animations. Traces are retained even on success.

`comparisons/` under each test output contains actual, expected, diff and original
state/metadata JSON for every case. The summary records changed and raw changed
pixels and both hashes. Pixelmatch uses color threshold 0.1, includes antialiasing,
and permits **zero** pixels over threshold. No mask is justified by these frozen
scenes. CI uploads SHA-named originals/diffs/metadata/traces for 14 days; essential
issue evidence is retained beside the delivery report. A page error, environment
change, absent contacts, wrong selection or corrupted/missing reference fails.

The negative driver changes the actual velocity CSS to red in a disposable copy.
Only the two velocity-visible frames must fail, with >100 changed pixels each;
the other six must pass. Timeout, startup error or unrelated visual failures do
not count as detecting the defect. It never edits checkout references.

<a id="propose-and-independently-approve-a-baseline"></a>

## Update visual references

1. Investigate unexpected differences. Do not update references merely because a
   check fails. Retain old/new originals, environment versions, metrics and the
   reason for an intentional appearance or environment change.
2. Generate candidates outside the tracked reference directory:
   `SALIMON_CAPTURE_CANDIDATES=/tmp/navigation-candidates npm run test:visual`.
   Candidate generation is forbidden in CI; `--update-snapshots` cannot update
   this custom comparator's references.
3. Inspect candidates and copy their complete `manifest.json` and images here in
   a reviewable change. Run normal comparisons twice and the negative driver;
   retain previous originals in git and expected/diff artifacts.
4. Publish the evidence with the implementation. No human approval comment,
   manifest digest approval or separate `baseline-approval` CI job is required.
   The owner reviews the deployed version and creates separate issues if needed.
   This changes subjective review timing, not pixel thresholds or functional gates.

The former `npm run test:visual:approval` script remains a legacy optional tool;
it is not invoked by CI or required for delivery. This PR does not change repository
branch protection or merge/deployment authority.

Software SwiftShader browser coverage does not qualify physical input, discrete
GPUs, Metal, Windows rendering, native platforms, hardware FPS or subjective art.
Historical native mesh regression #228 is a distinct renderer/asset contract.

## Panoramic candidates (#816)

The shared manifest includes 19 panoramic references; the eight older PNGs and
their environment/pixel policy are unchanged. A passing comparison demonstrates
repeatability, not subjective appearance approval. No human approval job is required.

| Originals | Case |
| --- | --- |
| [Wide](panoramic-wide.png), [translated wide](panoramic-translated.png) | 75° optical scene, no scan |
| [Telescope](panoramic-telescope.png), [translated telescope](panoramic-telescope-translated.png) | 0.1° and huge split origin |
| [Surface](panoramic-surface.png), [below horizon](panoramic-below.png) | Shared grounded horizon/opaque globe |
| [Empty](panoramic-empty-scanner.png), [detected](panoramic-detected-occluded-rear.png) | Scanner authority; hidden/rear contacts |
| [Overlap](panoramic-overlap.png), [pagination](panoramic-pagination.png) | Dense contact browser |
| [Sensor telescope](panoramic-sensor-telescope.png), [loss](panoramic-lost-contact.png) | Narrow markers and invalid action removal |
| [Look at](panoramic-look-at-occluded.png), [Set direction](panoramic-set-direction.png) | Independent camera/ship actions |
| [Stationary](panoramic-compass-stationary.png), [rear](panoramic-compass-rear.png), [overlap](panoramic-compass-overlap.png) | Compass states |
| [Warning](panoramic-small-warning.png), [terminal](panoramic-small-destroyed.png) | 960×600 priority |

See the [acceptance commands](../../README.md#panoramic-acceptance-matrix-816).
Generate a complete proposal with
`SALIMON_PANORAMIC_CANDIDATES=/tmp/panoramic-candidates npm run test:visual -- --grep PANO-VIS`,
inspect it, then copy the proposed PNGs/manifest here in a reviewable change.
Normal comparisons run the shared pixel policy, and the negative driver retains
a deliberately mutated expected telescope image, its isolated failure and trace.
Deployed-version review and merge authority remain with the owner.


## Shared panoramic styling proposal (#828)

Five HUD candidates now use the shared projected-instrument palette and controls:
stationary/rear/overlap compass and small warning/terminal. The 14 optical/sensor
panoramic and eight orthographic originals remain byte-identical. No environment
or comparison policy changes. These five replacements are covered by automated
comparisons; the owner reviews their appearance in the deployed version.
See [originals, diffs, measurements and exact digest](../../../reports/issue-828/README.md).

## Cross-view feedback reference proposal (#830)

Six proposed replacements cover explicit scanner contact loss, quiet scanner status
with a separately readable pulse counter, shared status styling and persistent
Warning text (stationary/rear/overlap HUD and small warning/terminal views).
The other 13 panoramic and eight orthographic references stay byte-identical.
The replacements originate from the matching Chromium 156.0.8078.4 CI capture
for PR #836, run 42437273206, merge revision
`3a758470fc84e34b95bb9cbb6a634967d977b944` (implementation head `ebfbc472`).
The PR records original/diff metrics and subsequent normal/repeat/negative checks.
Pixel policy, environment and `AWAITING_HUMAN_REVIEW` status are unchanged;
this proposal does not grant appearance approval or merge authority.
