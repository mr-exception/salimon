# Cloud and human verification

## Current policy (2026-10-23)

The owner selected cloud verification as the normal feature-completion path.
Use `auto-verification` and `human-verification` to describe who can establish
the acceptance criteria. Retire the `native` workflow label and `[native]` title
prefix. The game remains a native Rust/winit/wgpu application; this policy changes
verification ownership, not its architecture or supported-platform claims.

This policy supersedes earlier blanket requirements to finish every issue on an
Apple M1/Metal desktop. Keep historical evidence, failures and measured budgets
intact. Software Vulkan results establish their recorded Linux context; they do
not establish Metal compatibility, physical input, actual suspend/wake or M1 FPS.
The M1 performance target remains a hardware qualification target, not a prerequisite
for routine cloud feature delivery. Never loosen an approved numerical or physics
contract simply to complete a cloud task.

## Cloud-only routine acceptance (owner decision 2026-10-27)

**Default:** AI develops, tests, reports and proposes ordinary feature/fix
delivery entirely in ChatGPT-compatible cloud execution and repository CI.
Do not require the owner to launch the game, operate a native workstation,
perform gameplay walkthroughs, supply Apple M1/Metal screenshots, or sign off
visual polish before such tickets can be delivered.

For each implementation:
1. Convert observable gameplay requirements into deterministic portable tests,
   scripted input/session routes, state assertions, regression fixtures and
   failure/restart cases. Execute them against production code rather than mocks
   alone; keep test coverage proportionate to the change.
2. For rendering, use an actual production wgpu path on cloud Linux with
   Xvfb/Mesa software Vulkan, save original screenshots and compare measurable
   geometry, layout, pixel or temporal invariants with approved baselines.
   CPU-only DTO/pixel fixtures are supporting evidence, not substitute captures.
3. Run feasible fmt, Clippy, build, Rust tests, affected script/asset tests and
   relevant scenario/E2E suites. Record exact commands, revisions, environment,
   input/backend and outcomes (pass/fail/skip/unrun) in the issue report.
4. Fix failed required cloud-verifiable criteria before delivery. If a check
   cannot run because cloud tooling is missing, attempt to supply that tooling
   or automate an equivalent *meaningful* check; otherwise disclose it and
   keep materially unverified functionality explicitly incomplete.
5. Open/update the normal reviewable PR; do not require separate human/native
   testing for routine acceptance. The owner still reviews/merges the PR.

**Release qualification is separate:** Genuine Metal/Windows device behavior,
physical input, suspend/wake, hardware GPU counters, reference-M1 FPS, subjective
art/audio taste, legal source rights and product/balance decisions cannot be
inferred from cloud results. Preserve these as accurately scoped release risks
or decision issues (#557, #561, #563, #659, #660, #661 as relevant); they do not
block unrelated cloud-verified implementation delivery. A release claim requiring
one of these checks remains unqualified until its real evidence exists. Never
treat an unrun, blocked or failed criterion as a pass, weaken numeric/physics
requirements, approve assets/baselines/data rights automatically, or claim
end-to-end features exist when dependencies are absent.

**Issue hygiene:** New routine implementation tickets should use
`auto-verification` and contain executable cloud acceptance criteria. For
existing mixed/native-gated tickets, migrate the implementation criteria into
cloud-executable assertions, keep any genuinely physical/subjective conditions
as separately linked non-blocking release qualifications, and preserve failed
or unrun historical evidence. Do not mass-close human issues or remove genuine
functional blockers simply because this policy changed.

## Labels and scope

Give each active issue one primary verification label, preserving unrelated labels.
An implementation issue can be `auto-verification` even while blocked by another
implementation or product decision: the label describes its verification method,
not readiness. List such blockers explicitly.

| Label | Acceptance owner | Delivery |
| --- | --- | --- |
| `auto-verification` | AI using reproducible cloud checks | Execute checks, create/update a PR and report, update the issue with evidence and remaining blockers; close on PR merge. |
| `human-verification` | Person performing a review, physical experiment or explicit decision | Follow the issue's steps, capture the evidence below and return it to Codex; Codex validates the record, updates the issue, and either opens an evidence/remediation PR or closes directly when no repository change is needed. |

Split mixed work. Automated delivery must link the human follow-up and transfer its
exact unresolved criteria before closure; dependent work follows the actual
unresolved requirement. A human visual review does not block unrelated transaction
tests; source permission still blocks production ingestion. No closure itself
approves a baseline, data rights, tuning or performance threshold.

## Auto-verification procedure

1. Read the issue, dependencies, affected owner guides and [validation matrix](validation.md).
   Use a clean tracked revision and isolated save/output directories.
2. Select exact domain/session tests, malformed-input and failure cases, then
   exercise production adapters. Use real wgpu renders for rendered acceptance;
   synthetic DTO or CPU raster tests alone do not establish in-game output.
3. Use Linux Xvfb and Mesa lavapipe/software Vulkan for graphical routes and
   screenshots. Install prerequisites from the build/validation guides. Record
   absent capabilities as setup failures or unrun checks and repair them when
   possible. A cloud label does not assert every cloud executor is already ready.
4. Capture state assertions, input method (protocol versus OS-injected), original
   screenshots/video/audio, raw logs and hashes. Record revision/build/compiler,
   CPU/OS, adapter/backend/driver, viewport/scale, scenario/seed and power/workload.
   Simulated DPI and injected controls are not physical display/input evidence.
5. Compare only approved baselines with compatible provenance. Produce candidate
   corpora and diffs without automatic promotion or expanded masks/tolerances.
   Human appearance approval can happen remotely from cloud originals.
6. Profile identical release workloads serially with matched before/after controls,
   repeated samples and retained spikes/timeouts. Software-renderer timings are
   useful diagnostics; report missing hardware GPU counters as unavailable, never
   as zero. Do not equate CPU timings or submitted frames with physical 60 FPS.
7. Fix actionable defects, run required checks, write the completion report and
   open/update the PR. Update the issue with PR/report links, actual outcomes and
   exact transferred criteria. The owner reviews and merges; AI does not merge
   merely because it ran the checks.

Existing commands, including Xvfb environment setup, are maintained in
[validation](validation.md#native-graphical-gates),
[runner usage](../scripts/README.md) and
[visual verification](../scenarios/visual/README.md). This policy does not invent
scenario names or imply future Universe/Building adapters already exist.

## Human-verification procedure

Every human issue must state the revision/artifact to use, prerequisites, ordered
actions, expected outcomes, evidence to capture and where to return it. It must be
possible to identify each passed, failed, blocked or unrun criterion independently.

1. Obtain the exact build or original cloud evidence linked by the AI. For visual
   review inspect originals as well as contact sheets; for audio listen to exported
   WAVs/recordings with channel labels on suitable playback hardware.
2. Follow the issue's named views/routes/experiments. Record actual outcomes,
   including defects and uncertainty. Review cloud evidence remotely unless a
   physical screen/device/OS experiment is explicitly required. No general M1
   availability requirement applies to image review, listening or product decisions.
3. Capture originals and raw logs, not just a statement that it looked fine.
   Record reviewer/date, revision and artifact hashes; device/OS/display/DPI/audio
   context when relevant; expected versus observed behavior; timestamps and
   recordings for physical input or sleep/wake. Record exact approved scope and
   limits for decisions, masks, tolerances, source rights and budget exceptions.
4. Return the issue link, completed checklist and files/accessible evidence links
   to Codex in chat. Redact unrelated private content without destroying the test
   evidence. State every remaining failed/unrun check.
5. Codex checks coverage/provenance, updates the issue and writes an evidence or
   remediation PR if repository artifacts, baselines, code or canonical docs change.
   Use direct issue closure only for a fully evidenced human decision/review with
   no repository change: the issue update itself is the durable record. This is
   an explicit exception to the report-file/PR requirement for that case only.
   Incomplete evidence leaves the issue open or transfers exact residual work
   through the ordinary follow-up workflow; it never becomes a pass.

Baseline promotion still requires a separate reviewed PR. Preserve historical
Metal and software-Vulkan approval independently. Physical platform tests can use
an available suitable machine; an M1-specific claim requires that actual machine.
Gather optional platform/hardware checks in scheduled release sessions rather
than blocking routine cloud PRs on desktop availability.

## Active verification ownership

| Work | Auto issues | Human evidence/decision owner |
| --- | --- | --- |
| Universe render/flight/scanner/restore and scientific workload | #436–#439, #521 and implementation issues | #561 presentation; #557 physical platform checks; #563 hardware budgets |
| Scout lighting, machinery, engine, UI, Archive, recovery and audio | #483, #485, #489, #498, #502, #505, #515, #566 | #561 visual/audio/handling review; #660 route/balance decisions |
| Construction implementation and integrated routes | #538–#550, #551 | #561 spatial/preview review; #557 physical platform checks; #563 budgets |
| Physics stress optimization | #559 | #563 stress limits and hardware budget decision |
| Corpus/experience/balance automation | #568 | #659 source rights; #660 balance; #661 experience decisions; #561 corpus approval |
| Actual OS suspend/wake and physical display/input | AI prepares harnesses and analyzes evidence | #557 |

This table is routing, not a claim any blocked implementation, human review or
hardware target has passed. Issue bodies carry detailed current scope; dated
original requirements retained there are historical after the explicit migration.
