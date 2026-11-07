# Navigation visual references (#751)

These eight original #751 images are **corrected candidates pending explicit human approval**.
The initial proposal exposed a hidden native-select host-font dependency. The
current proposal pins Fontconfig to the licensed vendored DejaVu files; the only
image changes are the selected-value glyphs. See the [upgrade measurements](../../../reports/issue-751/candidate-upgrade.json)
and [original candidates](../../../reports/issue-751/initial-candidates/manifest.json).
Previous manifest approval does not approve this new digest.
Pixel equality is separate from accepting a reference as correct. The CI approval
gate must pass before QA acceptance. No implementation agent may post approval.

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

## Propose and independently approve a baseline

1. First investigate the unexpected difference. Never update references merely
   because a check fails. Browser/OS upgrades are separate deliberate changes;
   retain old and new originals, environment versions, metrics and upgrade reason.
2. Generate *candidates* outside the tracked reference directory:
   `SALIMON_CAPTURE_CANDIDATES=/tmp/navigation-candidates npm run test:visual`.
   Candidate generation is forbidden in CI. `--update-snapshots` has no effect on
   this custom comparator and cannot update these files.
3. Copy reviewed candidates and their complete `manifest.json` into this directory
   in a dedicated reviewable change. Run normal comparisons twice and the negative
   driver; preserve the previous originals in git and expected/diff artifacts.
4. A human repository OWNER/MEMBER/COLLABORATOR independently reviews every image,
   environment/threshold change, functional tests and diff. Post exactly this on
   the delivering PR, substituting the SHA-256 of the **manifest bytes**:
   `Approve navigation baselines: <sha256>`.
   Agents must never write this comment, impersonate the reviewer or count their
   own image judgment as approval. The human can use the same GitHub account that
   hosts an agent-authored PR; independence is from the implementing agent.
5. Rerun the failed `baseline-approval` job (the successful browser/build job does
   not need to run again). It validates image hashes and accepts only
   that exact digest from a human collaborator (Bot/unaffiliated accounts fail).
   Any changed manifest invalidates the approval. A byte-identical manifest
   inherited from the PR base needs no repeat approval. Baseline approval does
   not replace independent implementation QA or authorize merging.

GitHub branch protection must require the Navigation client check for enforcement
at merge; this issue does not change repository administration settings. A repo
maintainer can always alter workflow policy; ordinary implementation agents cannot
waive it. Review comments are audit evidence, not a cryptographic proof of who
operated an account. Required human decisions stay visible as BLOCKED until made.

Software SwiftShader browser coverage does not qualify physical input, discrete
GPUs, Metal, Windows rendering, native platforms, hardware FPS or subjective art.
Historical native mesh regression #228 is a distinct renderer/asset contract.

## Panoramic candidates (#816)

The shared manifest adds **19 panoramic candidates awaiting human review**; the
eight older PNGs and their environment/pixel policy are unchanged. The same
baseline-approval job checks the complete manifest digest. A passing comparison
does not approve either a new reference or its appearance.

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
Human approval and merge authority remain separate from these local checks.


## Shared panoramic styling proposal (#828)

Five HUD candidates now use the shared projected-instrument palette and controls:
stationary/rear/overlap compass and small warning/terminal. The 14 optical/sensor
panoramic and eight orthographic originals remain byte-identical. No environment
or comparison policy changes. These five replacements still require explicit
human approval of the complete manifest digest; they are not agent-approved.
See [originals, diffs, measurements and exact digest](../../../reports/issue-828/README.md).
