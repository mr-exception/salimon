# Issue #828 — shared panoramic projected instruments

Implemented on `issue-828-projected-cockpit` from `main` at
`7254ba929d86a9af5b427cd325de98dfec15dae5`. Both prerequisites (#825 and #826)
are merged into this base; no active PR stack existed at discovery or publication
preflight. Issue #828 remains open until merge.

The cockpit now uses the shared Expedition Interface tokens and button, utility
panel and dialog skins. Native controls retain their existing disabled/focus and
command handlers. Readouts are opaque over bright bodies and dark sky; compass
axis shapes/meanings remain cyan view, yellow heading and green velocity. Expanded
telemetry sits beside the seated survival column, above the footer. Help retains
navigation and survival warning summaries instead of dimming every warning behind
its modal backdrop. There is no navigation, simulation, save or optical shader change.

## Local criterion review

This is the implementing agent's **same-session source/evidence review**, not
independent execution or approval of visual references. Local criteria pass;
**QA pending — CI awaiting user report; human baseline approval UNVERIFIED**.

| Criterion | Result and actual evidence |
| --- | --- |
| 1: shared language, viewport and readability | PASS locally: HUD-STYLE at 960×600, 1280×720 and 1920×1080; full viewport/no page overflow, focused search/help, opaque readouts. Text/status contrast on instrument void is 6.81:1–14.29:1. HUD-BRIGHT renders an actual bright star and checks a bright scene pixel plus visible warning summaries at all three sizes. |
| 2: compass and physical scene | PASS locally: HUD-2 front/rear/overlap/stationary, repeat and negative compass comparison. All 14 optical/sensor/action reference originals are byte-identical to the base; only five HUD images change. No geometry, physical scene shader or compass arithmetic changed. |
| 3: contacts, commands and input | PASS locally: HUD-1/2/3, LOOK-1/2 and SENSOR-1/2/3/4 cover search/paging, offscreen/occluded contacts, stale/adversarial markers, selection, independent Look at/Set direction, grounded gates, telescope input/0.1° FOV, focus and Escape. HUD-BRIGHT verifies warning visibility inside focused help and focus return. |
| 4: lifecycle and authority | PASS locally: COCKPIT-1 completes two fresh production contexts, 2D→cockpit→2D, persisted observation, reserved seat, blackout and restoration. Its five repeat comparisons pass the unchanged pixel policy; navigation advanced 61/60 ticks in the two one-second samples. |
| 5: visual/keyboard evidence | PASS locally: originals cover dark dense contacts, grounded horizon, a deliberately bright star, focused dialog, dangerous approach, terminal warning/restart and the small desktop window. Human appearance approval remains UNVERIFIED. |

## Commands, results and source binding

From `client/`, with Linux x64, Node 24.19.0, npm 11.9.0, locked Playwright 1.64.1
and Chromium 156.0.8078.4 / ANGLE SwiftShader:

- `npm ci --no-audit --no-fund`: PASS, 98 locked project packages; lockfile unchanged ([log](logs/npm-ci.log)).
- `npm test`: PASS, **158/158** domain/script tests ([log](logs/unit.log)). Every Node-suite owner, test and dependency input remains unchanged after this run.
- `npm run build`: PASS on the initial UI source ([log](logs/build.log)). Final source `d716d84` also passes `npm run build -- --outDir /tmp/issue-828-final-dist` ([log](logs/build-final.log)); output was copied to `dist/` after the production journey finished. The separate output avoided changing the built app during the two-context journey. Existing large-bundle advisory only.
- `SALIMON_EVIDENCE_DIR=../reports/issue-828/browser npm run test:browser -- --project=panoramic --project=perspective --project=look-at --project=cockpit --grep 'HUD-|PERSPECTIVE|LOOK|COCKPIT-1'`: PASS, **13/13**, no skips, 11.3 minutes ([log](logs/browser.log), [raw results/embedded state and repeat attachments](browser/results.json)).
- `SALIMON_EVIDENCE_DIR=../reports/issue-828/bright npm run test:browser -- --project=panoramic --grep HUD-BRIGHT`: PASS, **3/3**, no skips, 27.1 seconds at final source `d716d84` ([log](logs/bright.log), [raw results](bright/results.json)).
- `SALIMON_PANORAMIC_CANDIDATES=/tmp/issue-828-panoramic-candidates SALIMON_EVIDENCE_DIR=../reports/issue-828/candidate-run npm run test:visual -- --grep PANO-VIS`: PASS, complete **19-state candidate matrix**, 3.8 minutes at `d716d84` ([log](logs/candidates.log), [raw results](candidate-run/results.json), [canonical scenario measurements](visual/summary.json)). Candidate generation does not assert approval or a passing comparison against the five superseded HUD images.

The 13-check run started at UI source `156df2da055659dbf8d6952c9aee1e024554ca9f`;
its production build stayed fixed throughout. Final source
`d716d84` adds only conditional warning summaries, their presentation prop,
the bright-only fixture branch/tests and a capture-name correction. In the earlier
scenarios that open help, these warning conditions are absent, producing the same
DOM; other scenes/handlers/owners and their tests/assets/lockfile are unchanged.
The new nonempty-warning boundary is verified by all three final-source HUD-BRIGHT
checks. Final candidate captures exercise all 19 regular scenarios on that source.
[Initial](initial-source-inputs.json) and [final](source-inputs.json) SHA-256 input
inventories bind this reuse. Later commits contain references, docs and evidence only.

The [initial development run](initial/results.json) was interrupted after HUD-1
while freezing the final CSS/tests. Its interrupted HUD-2 screenshot and unrun
HUD-3 are preserved, not counted as passes ([log](logs/initial-interrupted-browser.log)).
The subsequent 13-check run passes both on the stable source. The raw standing-return
repeat records 6,601 byte-level pixel differences, **zero** changed pixels under the
existing threshold; the policy is unchanged. The other four cockpit repeats have
zero raw changed pixels as well.

## Originals and reference proposal

- [Production grounded cockpit](browser/cockpit-COCKPIT-1-panorami-aeb94-t-and-persistent-simulation-cockpit/panoramic-horizon-0.png), [external observation](browser/cockpit-COCKPIT-1-panorami-aeb94-t-and-persistent-simulation-cockpit/external-body-observation-0.png).
- [960×600 dense contacts/warnings](browser/panoramic-HUD-2-compass-re-8cfc6-responsive-warning-priority-panoramic/warning-960x600.png), [focused help](browser/panoramic-HUD-1-full-viewp-3939b-browser-and-priority-Escape-panoramic/help.png).
- [960×600 bright body](bright/panoramic-HUD-BRIGHT-warni-121f3-p-over-a-bright-body-at-960-panoramic/bright-body-warning.png), [focused critical warnings](bright/panoramic-HUD-BRIGHT-warni-121f3-p-over-a-bright-body-at-960-panoramic/bright-body-focused-warning.png).
- [1920×1080 bright body](bright/panoramic-HUD-BRIGHT-warni-fb431--over-a-bright-body-at-1920-panoramic/bright-body-warning.png).

The broad run's filenames `bright-horizon` describe the normal grounded horizon,
which is mostly dark; final tests call it `grounded-horizon`. The additional
bright-body fixture deliberately supplies a luminous star behind the foreground
contacts through the production OpticalSky path; it does not add a production body.

Only the three compass HUD images and two small-window warning/terminal images
change. The eight orthographic and 14 other panoramic originals remain byte-identical.
[Change metrics and manifest digest](visual/changes.json) retain old/new hashes and
pixel counts; five original diff PNGs and all 19 state/environment JSONs are in
`visual/`. New originals live in the [tracked reference corpus](../../client/tests/navigation-baselines/README.md);
old originals remain available in base commit `7254ba9`.

Manifest SHA-256: `ec1d2f54f31a64f7f3c360fd45f6d334e6065c123d741ac768fea376ac779fe3`.
This is an **unapproved proposal**. No environment, comparison tolerance, negative
check, baseline policy or CI workflow was changed. A human collaborator must
review the images and approve this exact digest using the existing
[baseline procedure](../../client/tests/navigation-baselines/README.md#propose-and-independently-approve-a-baseline).
Normal full-browser comparisons, repeated reference runs, navigation/visual
negative drivers and baseline approval remain **UNVERIFIED**, awaiting the user's
current-head CI report. CI was not queried or monitored. No merge/deployment is authorized.

## Environment additions

Reusable template prerequisites actually added on Debian 13 / Linux x86_64:
Playwright Chromium **156.0.8078.4** (revision 1248), matching Headless Shell 1248,
and its FFmpeg **n7.0.1-playwright-build-1013** dependency. Reproduce after project
installation with `cd client && npx playwright install chromium` ([install log](logs/browser-install.log));
cache is `/home/agent/.cache/ms-playwright`. No system libraries were installed.

Project setup only: `cd client && npm ci --no-audit --no-fund` (98 lockfile-pinned
packages). Node/npm/system Chromium 154 were preinstalled; validation used the
new locked Playwright browser. A one-off root-level `npx playwright --version`
probe cached Playwright 1.65.0 outside the project; it was not used for validation
or recommended for the template. No saved environment configuration was changed.
Physical input, hardware GPU/Metal performance and screen-reader listening were
not exercised; browser input is synthetic and rendering is software WebGL.
