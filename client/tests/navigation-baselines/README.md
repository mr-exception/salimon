# Navigation visual references (#751)

These eight initial images are **candidates pending explicit human approval**.
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
hashes/renderer) fails explicitly. Host distro/version is recorded in the issue
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
5. Rerun the failed approval step/job. It validates image hashes and accepts only
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
