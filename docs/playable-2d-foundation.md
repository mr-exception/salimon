# Playable Scout / planet foundation (#767)

The browser's default `/` is a single authored 2D landing site with a permanently
exposed Scout interior. Walking, collisions, source inspection, automatic doorway
frame handoff and cockpit radar entry/exit share the existing runtime. The radar
is the existing Three/R3F navigation interface; it does not replace the world.

## Minimal desktop run

Use Node >=22.18. From the repository:

```sh
cd client
npm ci
npm run dev
```

Open the printed URL. Start outside; **WASD** walks. Approach the banded mineral
outcrop west of the ship or the purple plate colony southwest; **E** inspects,
**Escape** cancels. Walk around the hull to the wide aft doorway, north through
cargo, then around the cabin crate on its starboard side and forward through the
partition to the chair. **E / Use cockpit** opens radar. Use its labelled controls;
**Exit navigation · Stand up** returns standing in one action. Walk back outside.

For a built run: `npm run build && npm run preview`. The default built `/` has the
same gameplay. `/?scene=radar` isolates legacy navigation. Development-only
`/?powerFixture=1` exposes cockpit outage/restoration: blackout returns seated;
restoration stays seated until **E**; **Stand up / Escape** releases the chair.

## Repeatable acceptance

```sh
npm test
npm run build
npx playwright install --with-deps chromium
npm run test:browser -- --project=cockpit
```

`COCKPIT-1` covers ownership/reservation, normal exit, blackout/restoration and
grounded ship/scanner time continuity. `SLICE-1` is one uninterrupted journey through
the **built production `/`**: outdoor spawn → mineral and biological source
inspection → aft doorway → cabin/cockpit → real radar → standing → outside.
Each repeats in a fresh browser context. The original artwork is cached as
fixed-resolution sprites by `topDownSprites.tsx`; the same SVG adapter moves
them without changing gameplay frames or footprints. Inputs are actual Playwright keyboard/
mouse events; DOM attributes are read-only observations. No position, camera or
simulation setters are exposed. Fixed ticks drive movement and five simulated
seconds settle the follow camera before captures.

Both handoffs measure a 0.04 m step and <3 pixel avatar motion, changing frame
exactly once through each doorway crossing. Source IDs, kind, site, frame and
coordinates match before/after; the original SVG stays connected across radar
mounting. Existing unit suites independently verify rotated transforms, collision,
ship relocation and detached location snapshots. The normal Navigation client CI
also runs legacy desktop/deterministic/production radar checks, approved navigation
comparisons, negative mutations, repeat visuals and baseline approval.

Seven original slice checkpoints make the presentation reviewable: outdoor spawn,
mineral inspection, exterior with simultaneous exposed Scout interior, standing
cockpit return, return outdoors, grounded Vesper radar and reloaded outdoors. Replays use the repository's unchanged
`compareImages` policy, without masks or tolerance changes. A comparison of the
wrong checkpoint must fail. These are review evidence/candidates; they do not
promote or alter approved navigation reference PNGs. Results, original captures,
measurements, visual-review verdict and limitations live in the
[#767 report](../reports/issue-767/README.md); cockpit evidence lives in the
[#766 report](../reports/issue-766/README.md). Review is explicitly by the same
implementing agent in the active chat, with actual browser evidence; no separate
reviewer or independent execution is claimed.

## Shared foundation used by gathering #757

| Need | Existing owner / integration surface | Gathering integration |
| --- | --- | --- |
| Player movement, collisions and camera | `topDownRuntime.ts`, `topDownInteraction.ts`, `TopDownView.tsx` | Same world and input adapter |
| Stable source identity | `planetarySite.ts`: `source:landing:mineral:01` and `source:landing:biology:01`, `site:illustrated-landing:v1` | Versioned gram quantities and depletion in `resourceSession.ts` |
| Source selection and interaction | `requestInteraction` / `cancelInteraction`, reach tests, stable target ID | Tool action with capacity/success feedback and interruption semantics |
| Exact spatial locations | `capturePlayerLocation` / `copyPlayerLocation` retain split cells, offsets, facing and frame | Validated durable local save envelope; atomic with inventory/depletion |
| Ship-local / planet-fixed continuity | `crossScoutEntry`, `convertAnchor`; cockpit seat stays ship-local | Existing frame handoff preserved, payload counted once aboard |

**Gathering (#757):** Equip the gathering tool in Field inventory, approach either
source and use **E / Gather**. It collects up to 1 kg/s directly into the player's
10 kg / 12-slot unified inventory. Movement, Escape or stowing the tool stops it;
unfinished extraction progress is discarded. Weight and remaining source
mass are shown in kg; there is no volume limit. Completed transfers, source depletion, capacities
and frame-relative locations save together in this browser; reload never resumes
gathering automatically. See [the resource contract](resource-inventory-and-gathering.md)
for the catalog, transaction and interruption details. The multi-inventory
ledger foundation (#783) preserves this journey and mining behavior; schema v4
enables the identified 50 kg Scout Resource Container domain (#784), with
its playable E/Use deposit/withdraw panel (#785). Schema v5 adds trusted count-item definitions and exact atomic add/remove (#786),
with count/mass/shared-slot presentation in Field inventory. The production item
catalog remains empty pending approved content. Schema v6 adds the [active survival clock and HUD](browser-survival.md), 200 HP, pause/background handling and launch-only offline fatigue recovery (#787). Existing v1–v5 saves migrate losslessly under the same key.

**Unsupported future features:** loot/machine transfers; actual Core power; Fabricator recipes or
production; multiplayer/backend authority and reconnect; ship damage/destruction;
full planetary generation. Visible Core/Fabricator objects are inspection
placeholders; the Resource Container supports confirmed gram transfers. Gathering reuses this same world and resource-source IDs, with
integer grams and atomic local persistence; no independent scene was added.

The cockpit radar shares the authored Scout location on sample Vesper. Time and
scanner pulses continue, but the Scout stays grounded: throttle, turning, alignment,
takeoff and relocation cannot change its surface anchor. There is no restart-to-flight
action in the cockpit. The separate `/?scene=radar` fixture retains sample free flight.
Existing local saves bind to the versioned site and restore the same grounded ship;
scanner contacts, selection and elapsed radar ticks are session-only. Synthetic desktop input and
SwiftShader WebGL qualify the existing cloud path; physical input, hardware GPU,
mobile/touch, M1/Metal and native performance are unverified separately.

**Container loop (#785):** Mine material, enter through the aft entry and approach
the starboard Resource Container. Use E, enter positive integer grams, review
deposit/withdrawal and confirm. Escape/Close/focus loss cancels only the draft;
committed amounts survive exiting and reload. The built storage browser scenarios
join the existing cockpit project and retain panel/full/error captures.

## Viewport and HUD (#803)

The default production `/` and the isolated `/?scene=top-down` fixture render the
world edge to edge in a `100dvh` stage. There is no outer page padding or separate
header/footer space. The accessible scene title remains available to screen
readers. `TopDownScene` observes its actual SVG size and expands the reference
1200×640 view symmetrically to match the viewport aspect ratio, preserving uniform
world scale and the overhead camera. Resizing reveals more world along one axis
rather than stretching or cropping the reference scene; SVG screen transforms
remain authoritative at any device pixel ratio.

Survival/pause is anchored top left, inventory/save feedback top right, and
nearby actions/live status plus compact key hints bottom left. Panels intercept
input only inside their visible bounds, retain native button focus, and scroll
internally if their content grows. The followed character remains in the open
center. The existing resize/blur/visibility handlers release walking and cancel
uncommitted interactions. Storage uses a modal dialog in the browser top layer;
cockpit navigation retains its separate layout and survival HUD positioning.

`tests/viewport.spec.ts` covers the built default route at 1280×720 and 960×600,
HUD bounds/non-overlap, no page scrolling, uniform full-edge projection, focused
buttons versus walking, pause/resume, resize, DPR 2, and the dev fixture. Each
reference-size scene is captured twice and compared using the existing pixel
policy; an intentional inset-stage mutation must fail comparison. These captures
are repeat evidence, not approved navigation baselines.

### Universal ship-up orientation

The active ship's bow always points to the top of the 2D screen, whether the
character is outdoors, aboard, inspecting, gathering, paused, or returning from
the cockpit. `worldToView` rotates planet-axis relative positions by the inverse
ship heading; ship fixtures therefore render at zero angle. Terrain, player
facing and selection cues use the same basis. `viewToWorld` inversely maps
screen-direction W/A/S/D into the existing physical movement/collision adapter.
This is a presentation rule for every heading, not a change to the authored site
or saved physical orientation. Frame handoff does not change the camera basis.
The independent 3D radar keeps its existing orientation controls.
