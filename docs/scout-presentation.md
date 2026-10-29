# Scout presentation foundation

Verification ownership follows the [cloud/human policy](verification-workflow.md).
#483/#485/#489/#498/#502/#505/#515/#566 own cloud-renderer/interaction/audio
checks; #561 human presentation/listening, #557 physical platform checks and
#563 hardware budgets. Retain dated native evidence/failures; desktop availability
is not a prerequisite for independent cloud integration work.

The owner-approved *future* habitation/art requirements (2026-10-23) are maintained in [Scout interior art direction](scout-interior-art-direction.md). They add modular room/fixture lighting preferences, real observation-window shutters and selective glazing, independently animated furnishings, physical rail mounts, material-aware aging, clean repair appearance, continuous restrained ambience and simple doorway/screen sound attenuation. None is established as implemented merely by this note; continue to distinguish delivered #463/#465/#464 foundations from the new development issues explicitly linked in the interior contract.

## Lighting and material boundary (#463)

Warm cockpit/workshop fill at ship-local X=4/-3 metres contrasts with a cooler
exterior. Real Core bus loss removes warm fill and electrical emission while
retaining a cool visibility floor. This does not simulate emergency electrical
power, windows transmitting light, physical shadows or indirect illumination.
Physical screen atlas blank/restoration remains the existing explicit-power path.
Functional chassis/base colors and glass alpha are preserved; ordinary ship PBR
metallic/roughness/maps remain ignored. Do not use Cycles output to certify them.

Budget delta: zero new vertices, textures, buffers, draws or pipelines. The same
original 160-byte uniform carried the bus bit in its previously unused `center.w`.
Additional vertex arithmetic is bounded (two smoothstep zones, no light loop).
The machinery extension below enlarges that uniform to 192 bytes.
Historical Linux costs belong in the [delivery report](../reports/issue-463/README.md).
The [native lighting review](../reports/issue-483/README.md) retains actual M1/Metal
1280/1920 power pairs, repeat checks and separate Blender 4.5 LTS mood references.
Cloud power transitions and repeatability are reproduced in that report.
[Matched cloud timings](../reports/issue-566/README.md) are also delivered;
subjective appearance belongs to #561 and
hardware timing/budget acceptance to #563. No measured hardware budget is approved.

## Comparable native and offline reference cameras

[scout-cameras.json](../scenarios/visual/scout-cameras.json) owns cockpit, Core,
Fabricator and aft-door positions/targets in runtime ship-local metres. Native
E2E maps them through the presented ship pose without changing player/aim state.
Blender uses the same inverse axis conversion as the existing Fabricator preview.
Both use 70-degree vertical FOV and 0.05-metre near clip. Use 1280x800 for corpus
comparisons and repeat at 1920x1080 for readability review; record actual DPI.
The shared power pairs are fixed-time renders, not normal-play interpolation QA.

```sh
python scripts/model_visual_tests.py capture --binary target/debug/salimon-client --fixtures scenarios/visual/scout-fixtures.json --captures artifacts/scout-lighting/candidates
python scripts/model_visual_tests.py compare --fixtures scenarios/visual/scout-fixtures.json --captures artifacts/scout-lighting/candidates --temporal-only --output artifacts/scout-lighting/repeat-check
blender --background --python-exit-code 1 --python models/assets/ships/salimon-scout/preview_lighting.py -- artifacts/scout-lighting/offline
```

On headless Linux use an X display and software Vulkan per the validation guide.
The scenario starts at real blackout, captures all four cameras, feeds the Core
through the production interaction, advances one tick, and captures them again.
Repeated frames must stay identical within the declared tolerance. Keep native
captures separate from Blender mood references and from approved image corpora.
Inspect screens, glass, walking-lane edges and dark housings; never promote a
candidate without human image/mask/tolerance review. The [#451 native report](../reports/issue-451/README.md) verifies the current Core
blackout condition. Broader lighting/art/baseline approval remains #561;
cloud/reference fixture passes do not grant that acceptance.

## Soundscape boundary (#465)

`scout_audio.rs` observes immutable session/ship/character snapshots. Core,
conversion, funded Fabricator work, selected powered thrust, moving ramp,
grounded interior footsteps, power transitions and low-energy crossing retain
existing owners. Quiet ventilation is present only in a powered cabin. Idle
Core gain is restrained; proximity/thrust change actual operating sources.
Surface/Space/NearbyBody have no invented transmitted machinery, wind, music or
chatter. DoorwayBlend retains cabin admission. A blackout retires loops and
leaves only its finite transition. No audio state enters gameplay or run saves.

Listener/source positions remain ship-local metres, with look-relative panning,
`1/(1+d*d/9)` attenuation, 20 m cutoff and equal-power stereo. No HRTF, room
occlusion or atmosphere solver is implied. Footsteps still require real local
travel, walking input, artificial gravity and floor-level eye height. Survival
threshold/hysteresis/content remain `condition_feedback.rs`; its one-shot cues
join the same output rather than a second OS player.

`native_audio.rs` uses maintained [CPAL 0.18.2](https://docs.rs/cpal/0.18.2/cpal/)
(CoreAudio, WASAPI, ALSA defaults) for one continuous default-device stream.
`native_audio/mix.rs` owns deterministic original sample synthesis. Thirteen loop
slots, eleven ship transient slots and three survival slots bound the callback
to 27 voices. Runtime admits at most 24 scene voices and 12 pending events;
repeated callback event counters coalesce to the latest event. Physical partitions
and tactile mechanism channels follow [the acoustics contract](scout-acoustics.md).
Gain/pan converge over 10 ms without chunk-boundary fades. Low-pass procedural
noise and smooth saturation preserve headroom; no downloaded media is used.
Original survival timbre/envelope/relative level remain observational.

The producer publishes a fixed 240-byte atomic mailbox; the callback attempts at
most three coherent reads and retains its previous observation on contention.
Callback state is 1,080 bytes and neither allocates, locks, writes files, logs nor
calls back into CPAL. Sample phase persists across arbitrary buffer sizes. A
250 ms producer watchdog fades stalled input to silence and consumes stale event
counters. Pause/focus/occlusion/terminal/menu/reset/master mute retire events and
fade to silence; unmuting never replays discarded cues. Cabin and survival gates
remain independent. The silent stream stays resident until device replacement
or application drop, avoiding repeated startup and delayed queued audio.

A default-device ID poll runs on the main thread at most once per second. A
changed/failed/missing device drops the old stream, clears its mailbox and
reopens from current source observations. Errors are explicit and retry no more
than once per second; the callback only sets an atomic error flag. There is no
subprocess, WAV rewrite, persistent PCM history or settings-path dependency.
Supported default PCM formats convert at the boundary; mono downmixes, additional
channels are silent. Linux build environments need `libasound2-dev`; CI recipes
install it. Actual backend/frames/buffer/CPU-overrun telemetry is exposed by
native inspection. Default hardware buffering is observed, not an invented
universal latency guarantee.

Existing master mute, ship-audio toggle, ship volume and survival-audio toggle
retain the same separately persisted settings schema and visual alternatives.
E2E/benchmarks suppress device playback. Pure tests verify chunk invariance,
bounded onset, headroom, stale/reset/mute admission, exterior silence, real
production mappings and unchanged saved authority. Opt-in original stereo WAVs
and callback telemetry are retained in the [#582 report](../reports/issue-582/README.md).
Historical #465/#485 chunk/process evidence remains historical; those subprocess
lifecycle tests are replaced by callback lifecycle tests. Human listening #561,
physical output/device switching #557 and whole-frame hardware budgets #563
remain distinct from deterministic mixer evidence.

## Machinery feedback boundary (#464)

`runtime/scout_feedback.rs` maps immutable ship and Fabricator snapshots to typed
`MachineryFeedback`. This is a stateless presentation adapter; physical power,
energy, recipes, interaction, transforms, elapsed time and saves retain their
existing owners. Automation exposes the same mapping in `machinery` inspection.
Loading/restoration takes the current snapshot immediately; no event replay,
work catch-up or extra animation history enters a checkpoint.

| Surface | Authoritative observation and feedback |
| --- | --- |
| Engine glow | Signed powered free-flight throttle; magnitude controls drive emission, zero while coasting/landed/assisted/unpowered; separate running lamps remain bus powered |
| Core / spine / charge gauge | Charge-proportional steady emission, with a restrained four-second conversion modulation while the bus is powered; blackout extinguishes electrical accents |
| Fabricator panels | Quiet idle, progress-positioned teal work sweep only when funded, steady amber blocked tray, no emission during blackout; progress freezes with the actual job |
| Cockpit | Ready indicators follow powered cockpit authority; opposed rudder pedals move at most 1.5 cm from actual ramped yaw; monitor retains signed throttle/telemetry |
| Door / ramp / landing supports | Existing presented door fraction and hinge/extension geometry; supports remain authored fixed structure attached to the ship, including touchdown/bounce motion |
| Output / carried / loose cargo | Existing deliberate tray appearance/collection, carried grip and coherent fragment/ship transforms; no invented collection animation or duplicate item |

The phase uses integer active elapsed modulo four seconds before conversion to
f32. Paused/terminal worlds freeze it. Fabricator work uses actual normalized
job progress, so interruption cannot accumulate a hidden animation backlog.
Only named functional surfaces receive tags; structural meshes, colliders,
walking lanes, GLB authoring and mass accounting are unchanged. Existing physical
screen statuses and prompts are the nonanimated/text alternatives. There are no
flashing alerts, camera shake, particles, lights or postprocessing passes.

Renderer adds one f32 role per immutable ship vertex (72 -> 76-byte stride) and
two vec4 uniform slots (160 -> 192 bytes per rendered ship frame). Geometry counts,
textures, two opaque/glass draws, buffers and pipelines remain fixed. The role
shader adds bounded arithmetic with no source loop or dynamic allocation.
[Report](../reports/issue-464/README.md) records actual software-Vulkan debug costs
and limits, rather than claiming a full-frame or M1 budget pass.

An opt-in offscreen production-pipeline test records 14 synthetic reference
states, raw CPU/GPU samples and byte-identical repeated PNGs without a desktop.
These images test shader/DTO behavior, not production input or a gameplay run.
Run with an absolute output directory:

```sh
SALIMON_SCOUT_FEEDBACK_DIR="$PWD/artifacts/scout-feedback" VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json cargo test -p salimon-renderer --locked scout_feedback_gpu_evidence -- --ignored --nocapture
```

The [#489 cloud walkthrough](../reports/issue-489/README.md) retains production
window/session state, actual GPU close views and stepped motion sequences for
pedals, work sweep, blocked output, Core recovery and ramp reversal. Explicit
initial fixture placement is distinct from physical mining/travel. Human motion/
readability review remains #561, physical input/platform checks #557 and hardware
budgets #563. Fabricator orientation retains its three-screen geometry regression;
cloud images do not approve readability.
The physical scanner projector follows the Scout face and functional palette (#503),
using the existing contextual action bar and scanner-owned observations.
No native appearance, mask/tolerance or baseline approval is granted here.

## Scout UI readability contract (#466)

The renderer's [ui_style.rs](../client/renderer/src/ui_style.rs) owns one 5x7
bitmap face and functional palette shared by physical instruments, runtime
prompts/crafting/status text, toolbar and Q wheel. Runtime prose uses uppercase
labels; instruments preserve `m`, `k` and `s` to distinguish SI readings such as
`m/s` and `Mm/s`. Brackets, selection chevrons, plus, percent and recipe punctuation
have visible glyphs. Ivory is primary text, muted blue-gray is secondary, cyan
marks ordinary controls and amber identifies a warning **with explicit text**.
The opaque prompt background and screen palette have CPU sRGB contrast tests
(>=4.5:1 for these inks); perspective filtering, glass and native DPI still need
physical review. No per-pixel/native accessibility certification is implied.

| Surface | Presentation and source | Authority / verification boundary |
| --- | --- | --- |
| Door, cockpit, Core and Fabricator interactions | Compact `[E]` action labels in runtime `app/interaction.rs`; one `action_bar.rs` raster and authored world anchors | Existing range/aim/location/occlusion, input, expiry and power rules stay with their owners |
| Cargo/mining prompts | Existing resource names, mass, F carry/drop and left-mouse mining guidance use the same prompt face and palette | `resource_context.rs`/carrying/mining own eligibility; depth/fitting remain renderer-owned |
| Cockpit flight/braking/navigation | Five atlas sections total; cockpit uses sections 0–2. Long numeric readings use bounded scientific notation with units; text fits at integer density >=2 or shows an explicit ellipsis | Renderer formats immutable telemetry; no navigation/scanner data is invented |
| Core screen | Section 3; energy, draw, conversion, depletion and textual low-energy warning | Core bus state owns blackout; screens blank and restore through the existing cache |
| Fabricator physical screen | Section 4; IDLE/WORKING/NO POWER/OUTPUT BLOCKED, separate job/time rows and screen/port/tray action hints | Status/job/time observations only. Unknown DTO values show STATUS UNKNOWN; blackout still blanks the whole atlas |
| Fabricator recipe panel | Existing modal; readable status words, exact ingredients, queue selection `>` and no-refund controls | Existing atomic enqueue/cancel, focus/range revalidation and exclusive input remain authoritative |
| Toolbar, Q and H | Shared face/ivory/cyan accents; original food/medical artwork, category colors, layout and modal rules retained | Equipment/inventory/survival selection and consumption stay with existing owners |
| Reticle | Existing outlined white dot / equipped cross at the exact camera center; fixed physical size retained | Equipment decides the shape; no aim/ray/interaction rule changes |
| Physical scanner | Three bounded rows: explicit FRESH / NO FRESH CONTACTS / OFFLINE, center distance in km, sampled age in s, bracketed M/P controls and logarithmic range notice; shared face and palette | Runtime maps authoritative same-frame contacts and power to renderer `ScannerReadout`; no detection, expiry, clock, save or input rule changes |

Physical screens remain self-lit opaque geometry behind their existing housings;
no glass opacity, material, draw, shader or pipeline change is introduced. The
checked-in Fabricator UVs increase toward ship +X, which is viewer-left from its
cabin-facing -Z side. `ship_mesh.rs` flips **only that surface's U** at import so
text reads viewer-right. This is an explicit presentation interpretation of the
existing export, not an edit to generated geometry or collision. A regression
checks all three screens' U against viewer-right and V against world-up. Future
asset UV changes must deliberately reconcile/remove this mapping to avoid a
double flip. Core and cockpit mappings retain their authored orientation.

Prompt density remains 2/3/3 at 1x/1.5x/2x UI scale; the user's compact label and
enlarged-dot choices are retained. Unchanged instrument sections are not
rerasterized; unchanged full display keys skip GPU uploads. The five-section
2560x256 atlas is unchanged in size (2.5 MiB per CPU/GPU RGBA copy). Native M1
appearance/input/frame-tail acceptance is [#502](https://github.com/mr-exception/salimon/issues/502),
separate from CPU rasters and software Vulkan. See the
[delivery report and evidence](../reports/issue-466/README.md).

## Core contrast verification condition (#451)

Use the existing production Core **bus-off state** as the dark-cabin condition,
paired with bus-on from the identical named Core camera. The #463 lighting
contract above supersedes the fixed-warm-light premise in the historical #299
report: the shader's `ship.center.w` bus bit now removes warm/task fill and
electrical emission while retaining cool visibility fill. Core screen blanking
uses the same authoritative power state. Do not add a separate brightness knob,
emergency energy supply or synthetic night fixture for this verification.

The maintained `scout-lighting.json` route starts with a depleted Core, captures
the bus-off camera, feeds the real carried fragment through production intake
and advances one tick before the bus-on capture. Named cameras change only
presentation. `reports/issue-299/core-views.json` adds actual-player-camera close,
oblique and receding views. Keep their original state assertions; native captures
verify the shader/material/screen contrast at the recorded context without
promoting any baseline or weakening an authored contract.

See [#451 native evidence](../reports/issue-451/README.md) for the exact binary,
shader digest, Metal/drawable context, images, pass results and remaining wider
appearance/performance limits. The dated #299 report is retained unchanged.

## Matched cloud lighting timings (#566)

The [#566 report](../reports/issue-566/README.md) owns serial current/control
software-Vulkan measurements and original repeated1280×800 captures. The control
changes only the retained pre-#463 lighting expressions within current interfaces;
it is not a historical whole-game build. The named cloud admission profile keeps
PID/focus/visibility, viewport, frozen source and consecutive-frame checks; the
default M1 profile remains strict. Frozen fixture intervals are not normal-play
FPS. Actual hardware GPU counters/budgets remain #563, physical platform checks
#557 and original-image/mood/projection/baseline approval #561. Existing1920/M1
and Blender mood originals remain separate dated corpora.


## Tactile audio and physical partitions (#583)

The [maintained acoustics contract](scout-acoustics.md) describes actual mechanism/attachment events, finite cargo/privacy separation, differentiated floor contact and bounded continuous CPAL filtering. E2E mapping stays silent; actual output captures and human listening remain separately identified.

## Scanner readability (#503)

`scanner_readout.rs` formats observed values into at most 39 cells per row.
Invalid readings show `--`; large finite values use scientific notation without
losing km/s unit case. Labels are bounded to twelve visible ASCII cells. Existing
`action_bar.rs` provides the shared opaque contrast-tested background, face and
integer DPI scaling, without a second overlay. The 3D projector converts the
shared sRGB palette to linear instance colors: cyan fresh/control, amber selected,
muted history and ivory leads/origin. Selection also enlarges the marker; text
names current freshness and power independently of color. Chart mode retains its
separate explicit historical/lead provenance. No blink, new draw or detection is
added. #438 owns the integration matrix; #502's historical review now follows
#561 presentation, #557 physical devices and #563 hardware qualification.
See [#466 foundation report](../reports/issue-466/README.md) and
[PR #504](https://github.com/mr-exception/salimon/pull/504).

The [#438 cloud matrix](universe-scanner.md#cloud-scanner-acceptance-438) adds a top-right placement for fresh scanner status at 1280+ physical pixels. It keeps the enlarged 2x toolbar/status stack from covering the selected 3D contact; compact and chart-mode placement remains bottom-aligned. See the report for original failed and corrected captures.
