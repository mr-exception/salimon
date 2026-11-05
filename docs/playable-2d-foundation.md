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
nonzero ship simulation continuity. `SLICE-1` is one uninterrupted journey through
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

Five original slice checkpoints make the presentation reviewable: outdoor spawn,
mineral inspection, exterior with simultaneous exposed Scout interior, standing
cockpit return and return outdoors. Replays use the repository's unchanged
`compareImages` policy, without masks or tolerance changes. A comparison of the
wrong checkpoint must fail. These are review evidence/candidates; they do not
promote or alter approved navigation reference PNGs. Results, original captures,
measurements, visual-review verdict and limitations live in the
[#767 report](../reports/issue-767/README.md); cockpit evidence lives in the
[#766 report](../reports/issue-766/README.md). Review is explicitly by the same
implementing agent in the active chat, with actual browser evidence; no separate
reviewer or independent execution is claimed.

## Reuse boundary for mining #757

| Need | Existing owner / integration surface | Remaining mining work |
| --- | --- | --- |
| Player movement, collisions and camera | `topDownRuntime.ts`, `topDownInteraction.ts`, `TopDownView.tsx` | Reuse the same world and input adapter |
| Stable source identity | `planetarySite.ts`: `source:landing:mineral:01` and `source:landing:biology:01`, `site:illustrated-landing:v1` | Connect quantities, density catalog/version and depletion |
| Source selection and interaction | `requestInteraction` / `cancelInteraction`, reach tests, stable target ID | Add gathering actions, capacity/success feedback and cancellation semantics |
| Exact spatial locations | `capturePlayerLocation` / `copyPlayerLocation` retain split cells, offsets, facing and frame | Serialize/validate/restore authoritative durable saves; snapshots are currently in-memory only |
| Ship-local / planet-fixed continuity | `crossScoutEntry`, `convertAnchor`; cockpit seat stays ship-local | Preserve these frames during gathering/save integration |

**Unsupported future features:** mining/harvesting extraction, source depletion,
loot/transfer and persistent inventory; actual Core power; Fabricator recipes or
production; multiplayer/backend authority and reconnect; ship damage/destruction;
full planetary generation. Visible machine/storage objects are inspection
placeholders. The pure resource domain is not wired to the world: #757 must
update its old integer-kg model to whole litres with integer grams/L, enforce
independent 10 kg / 10 L limits, and implement durable nonduplicating transactions.
No new independent game runtime is needed or approved by this slice.

The sample radar's ship simulation runs continuously, but its illustrative flight
does not relocate the authored landed 2D ship. Synthetic desktop input and
SwiftShader WebGL qualify the existing cloud path; physical input, hardware GPU,
mobile/touch, M1/Metal and native performance are unverified separately.
