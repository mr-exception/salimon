# Backend architecture: authoritative simulation and networking

**Status:** Approved design baseline, **not implemented** (2026-11-05). This is a future server specification for the active browser-first 2D Salimon. It does not claim that a NestJS service, online authentication, MongoDB state, client prediction, or multiplayer gameplay is already running. The 3D holographic radar remains a cockpit-only view, not a replacement for the overhead world.

**Decisions recorded:** NestJS + TypeScript + MongoDB; a modular monolith running simulation **inside the single NestJS Node.js process initially**; **fully server-authoritative** gameplay with client-side input prediction; one canonical persistent universe; and checkpoint-based crash recovery with strict staleness suspension. The companion [persistence/recovery contract](backend-persistence-recovery.md) owns durability, write ordering, failure modes, and restart behavior.

**Specification precedence:** Current browser capabilities are inventoried in [current-client-status](current-client-status.md), [technical-architecture](technical-architecture.md) and [playable-2d-foundation](playable-2d-foundation.md). Existing approved multiplayer/presence, [cockpit/reconnect](cockpit-navigation-and-reconnect.md), [Stasis Pod/run lifecycle](stasis-pod-and-transfer.md), [resource/inventory](resource-inventory-and-gathering.md), [canonical planetary generation](canonical-planet-generation.md), and [universe expansion](universe-expansion-approved-design.md) define gameplay rules. This file introduces backend boundaries, not new balancing, damage, physics, canonical catalog, or authentication-product decisions. Historical Rust/native save or networking notes are **not** current browser implementation.

## 1. Technology and deployment scope

| Concern | Initial choice | Boundary |
| --- | --- | --- |
| Server | NestJS on Node.js, TypeScript | One process; framework only at adapters/composition |
| Database | MongoDB, with Mongoose via `@nestjs/mongoose` proposed | Repositories and persistence coordinator own database access |
| Game protocol | Native browser WebSocket via NestJS WS adapter proposed; REST for account/admin/static endpoints | Versioned typed commands and snapshots, no client-authoritative writes |
| Session identity | JWT-based authenticated sessions proposed | Exact issuance/refresh, account policies and provider are future decisions |
| Simulation | Fixed-step, framework-independent TypeScript game core | One logical authoritative writer per region, initially all owned by one process |
| Client | Existing React/TypeScript overhead 2D + cockpit 3D radar | Predict local movement, render/interpolate, reconcile; never author canonical changes |
| Packaging | `server/`, `shared/protocol/`, `shared/game-core/`, `docker/Dockerfile.server` proposed | No actual files/services are delivered by this document |
| Deployment | Docker / Coolify; future root Compose service | **One active simulation-owning server replica** until handoff/fencing exists |

Do **not** introduce Kafka, Redis, Kubernetes or microservices for the first vertical slice. Leaving domain interfaces extractable does not imply distributed execution is already supported. MongoDB transactions require replica-set or sharded deployment; a single-member replica set is suitable for local development **but not highly available**. Production backups, monitoring, TLS, secret handling and failover require implementation plans.

## 2. Domain module ownership

| Module | Primary responsibility | Explicit non-responsibility |
| --- | --- | --- |
| `AuthModule` | Authenticate account/session and authorize a connected character | Does not decide game physics or world visibility |
| `NetworkModule` | WS/REST transport, envelopes, admission, outbound dispatch | Does not mutate game state directly |
| `SimulationModule` | Clock, fixed-step scheduler, command sequence, region ownership, order of effects and publication | Does not embed Mongoose, NestJS gateways or renderer types in game core |
| `UniverseModule` | Canonical universe clock, versioned catalog, celestial frames and ephemeris queries | Does not invent missing observational/source data |
| `WorldModule` | Planet sites, stable entities, interaction footprints and mutable world deltas | Does not credit inventories |
| `PlayerModule` | Player simulation, anchored location, movement, health, alive/dead state | Does not own network sockets |
| `ShipModule` | One owner/ship, ship physics, mass, cockpit and machine simulation | Does not implement client radar UI |
| `InventoryModule` | Integer-gram and count-item accounting, capacity, conservation, transfer proposals | Does not write to collections except via persistence owner |
| `InteractionModule` | Validated use/mining/fabrication/construction command transitions | Does not independently publish or durably commit an outcome |
| `PresenceModule` | Connect/disconnect, active controls/locks cancellation, reconnect anchor and offline passenger tracking | Does not retain offline avatars in world |
| `InterestModule` | Per-subscriber visible/sensed/authorized entity selection, snapshot filtering | Does not reveal hidden world/catalog details |
| `PersistenceModule` | MongoDB codecs/repositories, idempotent critical commits, checkpoint coordinator and recovery | Does not choose domain gameplay outcomes |

`SimulationModule` composes domain functions behind explicit interfaces. Domain code is a pure/controlled-state dependency leaf; NestJS adapters, Mongoose schemas, clock services, and sockets must not leak into `shared/game-core`. Avoid circular imports and broad "god service" dependencies. The composition layer alone joins player, ship, world and ledger invariants.

### Proposed repository layout (future, not created)

```text
client/                         # Existing 2D React client and cockpit radar
server/
  src/
    main.ts
    app.module.ts
    auth/
    network/
    simulation/
    universe/
    world/
    player/
    ship/
    inventory/
    interaction/
    presence/
    interest/
    persistence/
  test/
  package.json
shared/
  protocol/                    # Versioned wire DTOs / validators
  game-core/                   # Pure deterministic simulation and game invariants
docker/
  Dockerfile.client            # Already exists
  Dockerfile.server            # Proposed
docker-compose.yml             # Extend, do not replace existing client deployment
```

Extract the current `client/src/simulation.ts`, `topDownRuntime.ts`, `topDownFrames.ts`, `topDownTraversal.ts`, `topDownInteraction.ts`, `resourceInventory.ts`, and `resourceLedger.ts` only where tests prove behavior parity and browser dependencies have been removed. Keep camera, art, SVG/Three rendering, DOM/Web Lock and localStorage in the client. Do not equate currently illustrated radar ship motion with already integrated 2D landed-ship movement.

## 3. Authoritative tick and command pipeline

A logical region has exactly one current **simulation owner**. In phase one all regions are held by one server process. A region is a scheduling and visibility partition, **not** a separate gameplay copy/shard; players at the same canonical place must encounter each other.

1. Transport validates envelope/version, socket session, size, command rate, input sequence and authentication. Untrusted commands enter an ordered command queue; no gateway writes state.
2. At a fixed simulation tick, the owner admits commands and checks temporal order, reach, reference frames, permission, authoritative state and gameplay preconditions.
3. Framework-free domain functions compute proposed state/effects in deterministic order. Movement/collision steps and simulated machine work use controlled time; no asynchronous MongoDB calls inside pure integration.
4. **Critical effects** (inventory/source transfer, terminal death/ship loss, ownership, committed machine jobs) are finalized only after the persistence coordinator confirms their durable commit. Do not show an irreversible outcome as final before that point.
5. Noncritical physical state is advanced in memory; versioned snapshots are published on an independent cadence. Critical-event publication follows commit, using the canonical resulting revision.
6. A durable command waiting on I/O blocks conflicting entity mutation/confirmation only; the server must not block the entire fixed-step loop on a database promise. If ordering/collision dependencies make safe continuation impossible, suspend the affected simulation scope until the outcome resolves.

**Initial performance targets, not approved physics balancing:** active simulation at 60 fixed steps/s (consistent with current 2D and ship code); client movement prediction at 60 Hz; outgoing state snapshots at ~20 Hz; rendering at display refresh. Measure actual CPU, latency, dropped ticks, memory, bandwidth and network jitter before making these hard production promises. Never pretend 20 Hz networking replaces 60 Hz simulation. The event loop must not silently discard simulation debt; bounded catch-up or explicit safe suspension is required.

### Command and snapshot contract (illustrative)

```ts
type CommandEnvelope<T> = {
  protocolVersion: number;
  sessionId: string;
  inputSequence: number;           // Per active session; monotone
  clientTick: number;               // Advisory, never authority
  command: T;
};

type SnapshotEnvelope<T> = {
  protocolVersion: number;
  regionId: string;
  serverTick: number;
  stateRevision: string;            // Lossless representation
  lastProcessedInputSequence: number;
  state: T;                          // Interest-filtered authoritative state
};
```

These are proposals, not shipped API signatures. Validate at the server edge; clients cannot set mass, health, inventory totals, source remaining grams, ship owner, other player locations, or terminal state. Command IDs for once-only operations are **distinct** from volatile movement input sequence numbers.

### Client prediction and reconciliation

- Local player movement can predict against shared movement/collision contracts for responsiveness; prediction cannot mint items, complete mining, lock a cockpit, kill a ship or finalize combat.
- Retain a bounded local input buffer. When a snapshot arrives, reset predicted state to the authoritative state at its acknowledged sequence and replay only later pending inputs. Smooth presentation errors without hiding persistent mismatches or bypassing collision.
- Remote entities interpolate from interest-filtered server snapshots and never use remote client claims.
- Discard/reinitialize the prediction buffer across reconnect, epoch/catalog mismatch, incompatible snapshot versions, and authoritative frame changes.
- 2D world coordinates and body/ship-local anchors retain split large-integer cells and local offsets. Wire codecs must preserve full precision (for example decimal strings for large integer cells), never flatten galactic positions into JS `Number`.
- A cockpit user still exists as a seated player in the 2D world. The cockpit-only 3D radar is local presentation; spectators receive seated state, not a 3D camera stream.

## 4. Interest management and single-universe ownership

An observer subscribes to authorized nearby physical entities in the same canonical world. Space proximity, planet surface cells, and ship-local interior occupancy can determine simulation/interest regions, but exact partition sizing is unselected. Future radar contacts must remain filtered by the approved scanner and player-known discovery rules; "loaded by server" does not mean "visible to client".

Inactive regions are eligible for event-driven or analytic advancement only when it preserves the required motion, power, collision and causal outcomes. Do not skip destructive ship encounters because both pilots are offline. When a region becomes active, reconstruct its state at the common universe epoch and validate any scheduled/possible contacts before publishing it. If available ephemeris/collider information is insufficient, block affected activation rather than fabricate safety. No user-controlled time acceleration.

Later multiple workers need region owner handoff, lease fencing, cross-region event ordering and atomic publication. Running `node server` in two live replicas before those mechanisms exist would create conflicting authorities.

## 5. Presence, cockpit, Stasis Pod and offline behavior

Follow [Stasis Pod, logout, transfer and permadeath](stasis-pod-and-transfer.md) as the **new future reconnect authority**, alongside [cockpit-navigation-and-reconnect](cockpit-navigation-and-reconnect.md) for chair and 2D/3D interaction:

- Every character/run permanently owns **one distinct ship**, with an assigned starter pod on the Scout. A fresh run starts **outside** its landed Scout. Moving pod binding never reassigns ship ownership or grants a guest pilot rights. Ships of deceased characters remain in the world, intact if not physically destroyed.
- Disconnect removes avatar/collider, cancels interactions/held inputs and releases chair/pilot controls; it does not stop committed jobs or ship motion. A **confirmed assigned-pod interaction** is the only safe logout and preserves ordinary inventory/equipment/suit.
- An unsafe logout elsewhere **drops ordinary carried inventory, equipment and equipped suit into one recoverable loot cache at the departure frame** (primary items protected only if explicitly defined), then reconnect is to **the bound pod**, not the old planet-fixed/ship-local position.
- One **single-use, one-hour code** entered in an available friend's pod creates an online approval request; on approval, an idempotent durable transfer drops ordinary gear/suit as loot, rebinds pod, relocates with a loading screen and preserves original ship ownership.
- Offline loss of assigned pod through ship destruction or deliberate modular dismantling causes **permanent run death**; next login starts a **new** run. There is **no rescue-only state/deadline or post-death pod reassignment**. Online loss leaves the character alive in that session but requires a new assignment before the next login.
- Keep actual frame-relative world positions for physical movement, boarding, causal transactions and loot/wreck reconstruction. These are **not the normal login spawn anchor** once Stasis Pods govern reconnect.
- **TBD:** network grace/accidental disconnect, pod power/disablement, primary-item policy, suitless arrival safety, code approval expiry/rate limits, per-part salvage yields and Core-root decommission. Do not invent an implementation rule for these.

## 6. Development milestones and validation

| Phase | Delivery | Required evidence |
| --- | --- | --- |
| 1. Bootstrap | `server/` NestJS, MongoDB replica-set development config, WS/REST auth skeleton, health/readiness, ownership fence | Build, config/health, one-instance ownership, startup with unavailable DB |
| 2. Movement | Pure 2D movement and collision contracts, server 60 Hz scheduler, typed inputs, prediction/reconciliation, two clients | Deterministic tick/reference tests, malformed commands, latency/jitter/reorder, actual browser E2E |
| 3. Shared mining | Source/ledger conservation, atomic gram transfers, count-item validation, checked revisions, durable operations | Two-player depletion race, retries, full capacity, crash at each commit boundary |
| 4. Ship, pod and reconnect | Owner-only chair, modular pod placement, safe/unsafe logout, single-use approved transfer, per-run ownership and pod-loss death | Two players aboard, cargo-safe logout vs dropped suit/gear, 1h code races, online/offline pod loss, fresh run, 2D destination spawn |
| 5. Scaling readiness (later) | Region isolation/metrics, idle catch-up qualification, fenced ownership handoff | Transfer, crash/recovery and simultaneous-region acceptance; no unverified seamless scale claim |

Production browser Playwright and Node/TypeScript tests are the active gates; retired native Rust or M1-specific tests do not certify or automatically block this planned backend.

## 7. Deferred decisions (do not implement by assumption)

- Final authentication provider, account/character session limits, token lifetime/refresh/revocation and abuse prevention.
- Exact physics numeric representation/authoritative implementation across TypeScript, long-distance integration and deterministic replay fidelity.
- Tick backlog, network rate limits, interest radius, latency budgets and server capacity targets.
- Region borders, safe ownership handoff, horizontal scaling, service placement, and multiplayer combat rewind/lag compensation.
- Movement input grace on network stalls and definition of disconnect versus transient reconnection; precise blocked-anchor fallback.
- Core/Fabricator balance, full catalog/planet generation deployment, combat/economy/NPC and broader content delivery.
- Verified checkpoint target is **every 5 seconds**; strict gameplay suspension is required at **30 seconds** since the latest verified checkpoint. Implementation-specific retry/monitoring budgets and safe partition scope remain TBD.

See [backend persistence and recovery](backend-persistence-recovery.md) for the fixed durability guarantees and open implementation knobs.
