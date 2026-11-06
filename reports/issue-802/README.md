# #802 — One grounded Scout across world and cockpit

The production `/` world and cockpit radar now consume `scoutLocation.ts`'s shared
site/body/frame/physical anchor: Scout grounded on sample Vesper, zero speed and
throttle, heading matching the authored 2D pose. Scanner/time and payload mass
continue independently of presentation. Propulsion, steering, alignment,
relocation and restart-to-flight are gated until the 2D world supports those
transitions. The isolated `/?scene=radar` keeps its free-flight fixture.

The site maps local X/Y to sample X/Z at Vesper's +Y pole, with ship center on
planet radius + the existing conservative 5 m ship sphere. Split SI coordinates
remain intact. This is an illustrative binding, not canonical generation.
Existing saves already bind to `site:illustrated-landing:v1`: restoring player,
inventory and sources reconstructs this same fixed grounded anchor. No competing
ship save or schema change is introduced. Radar ticks/contacts are session-only.
The domain owner and lifecycle restriction are pure TypeScript, separate from React.

Base: `main`, `7d959ae8848efb19d4c377823f64f61a68120e37`; no active PR stack or
formal blockers at discovery/publication. Tested source:
`cbefee248129b4a242015018e55980618935c6d1`, branch
`issue-802-shared-grounded-scout`. [Source hashes](source-hashes.json) bind all
tracked client runtime/tests/assets/config/lockfile and workflows (excluding Markdown).
The only source difference from browser-tested `e8c8ac8` is a removed blank EOF
line. Unit/build were rerun at this source revision; the compiled JS/CSS names
and content are unchanged. Browser evidence is reused for identical executable
inputs. Final evidence/docs-only commit preserves the recorded source hashes. Exact published head/base
and current-head CI verdict are recorded in the PR review comment.

| Criterion | Result and original evidence |
| --- | --- |
| A1: outdoor startup / grounded planet radar | PASS: `scoutLocation.test.ts`, built SLICE-1 in two fresh contexts. Player starts (-6, 3), ship origin (2, -1), heading -0.12, `grounded`, contact `vesper`, physical clearance <0.0001 m. [Outdoor original](evidence/slice-outdoor-spawn-0.png), [cockpit status](evidence/radar-open-0.png), [selected Vesper](evidence/slice-grounded-radar-0.png) |
| A2: same anchor/lifecycle through time, visits and save/reload | PASS: domain integration checks repeated seating/standing, scanner discovery, outdoor and ship-local checkpoint restoration. Production journey checks exact radar position/mode/contact/heading across two cockpit visits, time and outdoor reload. [Raw journey and handoffs](evidence/slice-journeys.json), [reload state](evidence/slice-reloaded-outdoors-0-state.json) |
| A3: unsupported launch/relocation cannot diverge | PASS: direct domain throttle/alignment/held steering/reset checks retain the same anchor, orientation and contact. Browser asserts disabled throttle and Set direction, absent restart-to-flight action; actual held ArrowLeft and Reset camera preserve ship state. [Focused results](focused.log), [radar state](evidence/slice-grounded-radar-0-state.json) |
| A4: continuous traversal and scene | PASS: uninterrupted built outdoor → interior → cockpit → interior → outdoor; original SVG remains connected, sources unchanged. Both crossings change frame once, 0.04 m/tick and <3 px avatar step. Existing cockpit blackout/restoration passes. [Handoff samples](evidence/slice-journeys.json), [cockpit samples](evidence/cockpit-measurements.json), [standing return](evidence/slice-standing-return-0.png) |
| A5: deterministic checks, browser originals and owner docs | PASS locally: unit/integration, build, affected browser journeys, 11 repeat comparisons with 0 changed pixels. Wrong-checkpoint negative detects 89,817 changed pixels. Current status, foundation, radar, cockpit, architecture, maintenance map and client guides updated. Broad CI is pending at publication; final review comment records the actual result |

## Commands and actual results

From `client/` unless noted:

- `npm ci --cache /tmp/salimon-npm --no-audit --no-fund`: PASS. Default npm
  cache outside writable roots initially failed; writable temporary cache resolved it.
- `node --experimental-strip-types src/scoutLocation.test.ts`: PASS, 3 tests.
  Initial new tests failed because fixtures omitted chair selection and attempted
  an unchanged no-op checkpoint; corrected fixtures exercise actual domain writes.
- `npm test`: PASS, 22 test files, no failures/skips ([unit.log](unit.log)).
- `npm run build`: PASS, TypeScript/Vite ([build.log](build.log)); existing
  large-bundle warning remains. No dependency/lockfile changes.
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=/tmp/802-browser-initial
  npm run test:browser -- --project=cockpit -g 'COCKPIT-1|SLICE-1'`:
  COCKPIT-1 PASS (two contexts); SLICE-1 FAIL on an ambiguous contact-picker
  selector. A second attempt started before the selector edit loaded and failed
  the same way. Both failures and the original screenshot are in [failures](failures/).
- `CHROMIUM_PATH=/usr/bin/chromium SALIMON_EVIDENCE_DIR=/tmp/802-browser-final
  npm run test:browser -- --project=cockpit -g 'SLICE-1'`: PASS, 1 test / two
  contexts, 1.7 minutes ([slice.log](slice.log)). Seven original/repeat pairs,
  physical anchor assertions, repeated cockpit entry, restored save and negative
  wrong-checkpoint comparison. All pixel policies/masks/approved references unchanged.
- `git diff --check`: PASS. Broad Navigation client CI (full browser suite,
  navigation mutations, repeat approved visuals, visual mutation and baseline
  approval) and Docker smoke run once remotely on the PR; outcomes belong to
  the current-head PR review. These were not duplicated as broad local campaigns.

Environment: Linux cloud 6.18.57, Node v24.19.0, npm 11.9.0, system Chromium
154.0.8037.57. Viewport 1280×800/DPR 1, en-US/UTC, controlled browser clock
2026-11-04, real Playwright synthetic keyboard/mouse through production adapters.
SVG CPU rasterization; actual WebGL 2 / ANGLE Vulkan SwiftShader renderer recorded
in raw journey/cockpit evidence. Production preview `:4175`, development power
fixture `:5175`. Software rendering does not qualify physical input, hardware GPU,
M1/Metal/native performance or mobile/touch; those remain NOT RUN, separately owned.

## Same-session review

Implementing agent reviewed the final source diff, issue criteria, original outdoor,
radar and standing captures, raw handoff/ship measurements, repeat results and
negative comparison in this active chat. A1–A4 pass; A5's local evidence passes.
No independent execution or human baseline approval is claimed. New captures are
review evidence, with no approved baseline edits. Acceptance remains pending
required current-head CI at publication; the final PR comment records ACCEPT,
REQUEST_CHANGES or BLOCKED against exact revisions. No merge or deployment.

Remaining limitations: one fixed authored landing site; no takeoff, reentry,
planet relocation, rotating body ephemerides or server/multiplayer authority.
