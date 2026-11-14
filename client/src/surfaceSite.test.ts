import { test } from 'node:test'
import assert from 'node:assert/strict'
import { relativeM, translate } from './coordinates.ts'
import { physicalRadiusM } from './contact.ts'
import { dot, length, normal, scale } from './math.ts'
import { enterCockpit, leaveCockpit } from './cockpit.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { ResourceSession } from './resourceSession.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { captureScout } from './scoutSave.ts'
import { surfaceScoutLocation, transitionScout, type ScoutAction } from './scoutLocation.ts'
import { resolveSurface, siteEntities, startingSurface, surfacePosition, validateSurfaceClearance, type SurfaceAddress } from './surfaceSite.ts'
import { convertAnchor, point2, type Anchor } from './topDownFrames.ts'
import { capturePlayerLocation, createTopDownState, drawList, PLAYER_ID, stepTopDown, type TopDownState } from './topDownRuntime.ts'
const addressFor = (bodyId: string, direction: readonly [number, number, number], yaw = -.12): SurfaceAddress => ({ bodyId, radialM: scale(normal(direction), physicalRadiusM(SAMPLE_BODIES.find(b => b.id === bodyId)!)), yaw })
function place(world: TopDownState, anchor: Anchor): TopDownState { return { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e), camera: { ...world.camera, anchor } } }
function setup() {
  let raw: string | null = null, fail = false
  const storage = { getItem: () => raw, setItem: (_: string, v: string) => { if (fail) throw Error('quota'); raw = v } }
  let world = createTopDownState('interior', true)
  world.scoutLocation.boardingDoor = 'closed'
  world = enterCockpit({ ...place(world, { frame: 'ship:scout', point: point2(0, -3.1) }), selectedId: 'scout:chair' })
  const s = new ResourceSession(storage, world, undefined, undefined, () => 0), n = new NavigationRuntime(SAMPLE_BODIES, s.world.scoutLocation)
  const action = (a: ScoutAction) => s.transitionScout(a, s.snapshot.revision, s.world.cockpit)
  const launch = () => { assert.equal(action({ kind: 'begin-takeoff' }), true, s.feedback); n.advance(4); n.advance(2) }
  // Synthetic interplanetary approach; admission and touchdown use the real planner.
  const approach = (surface: SurfaceAddress) => {
    const target = surfaceScoutLocation(surface).ship
    n.clock.state = { ...n.clock.state, mode: 'flying', contact: null, position: translate(target.position, scale(normal(surface.radialM), 100)), velocityMps: target.velocityMps }
    s.world.scoutLocation.scanner = { lastTick: n.clock.state.tick, pulse: Math.floor(n.clock.state.tick / 300), selected: surface.bodyId, contactIds: [surface.bodyId] }
  }
  const arrive = (surface: SurfaceAddress) => {
    approach(surface)
    assert.equal(action({ kind: 'begin-landing', bodyId: surface.bodyId, surface }), true, s.feedback)
    while (s.world.scoutLocation.landing) n.advance(.1)
    s.advance({ ...s.world, tick: s.world.tick + 1 }); s.checkpoint()
  }
  return { s, n, storage, action, launch, arrive, approach, fail: () => { fail = true }, corrupt: (change: (v: any) => void) => { const data = JSON.parse(raw!); change(data); raw = JSON.stringify(data) } }
}
test('AC-1 deterministic physical regions on planets/moon have orthonormal tangent frames, sparse art and no sources', () => {
  const ids = new Set<string>()
  for (const bodyId of ['vesper', 'vesper-moon', 'caligo']) for (const direction of [[1, 0, 0], [0, -1, 0], [1, 1, 1], [0, 0, 1], [0, 1, 0], [1, 2, -3]] as const) {
    const address = addressFor(bodyId, direction, .73), site = resolveSurface(address)
    assert.deepEqual(resolveSurface(structuredClone(address)), site)
    assert.equal(resolveSurface({ ...address, yaw: -.51 }).id, site.id)
    ids.add(site.id)
    const { east, north, up } = site.frames.planet.basis!
    for (const axis of [east, north, up]) assert.ok(Math.abs(length(axis) - 1) < 1e-12)
    for (const [a, b] of [[east, north], [east, up], [north, up]]) assert.ok(Math.abs(dot(a!, b!)) < 1e-12)
    assert.ok(Math.abs(length(relativeM(surfacePosition(site), site.body.position)) - physicalRadiusM(site.body)) < .001)
    if (site.id !== STARTING_SITE.id) {
      assert.equal(siteEntities(site).length, 1); assert.ok(siteEntities(site).every(e => !e.source && !e.interaction && !e.footprint && e.appearance === 'surface'))
      assert.doesNotThrow(() => validateSurfaceClearance(site))
      const local = { frame: 'ship:scout' as const, point: point2(1, 7.5) }
      assert.ok(Math.hypot(...relativeM(convertAnchor(convertAnchor(local, 'planet:fixture', site.frames), 'ship:scout', site.frames).point, local.point)) < 1e-9)
    }
  }
  assert.equal(ids.size, 18)
  const site = resolveSurface(addressFor('caligo', [1, 0, 0]))
  assert.equal(resolveSurface({ ...site.address, radialM: [site.address.radialM[0], .01, .01] }).id, site.id, 'same physical cell is stable for nearby points')
})
test('AC-2/3/4 original depletion, local modifications and ship-local passengers survive moon/Caligo journeys and reload', () => {
  const { s, n, storage, launch, arrive } = setup()
  assert.equal(s.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 }), true)
  // Existing local changes stay in their owning world, never cloned to new sites.
  s.world = { ...s.world, entities: s.world.entities.filter(e => e.id !== 'stones:west').map(e => e.id === 'rock:west' ? { ...e, anchor: { frame: 'planet:fixture', point: point2(-13, -3) } } : e) }
  s.checkpoint()
  const original = s.world.entities.filter(e => e.anchor.frame === 'planet:fixture'), passenger = capturePlayerLocation(s.world)
  for (const address of [addressFor('vesper-moon', [1, 0, 0], .5), addressFor('caligo', [0, 0, -1], -.7), addressFor('vesper', [1, 0, 0])]) {
    const currentPassenger = capturePlayerLocation(s.world)
    launch()
    assert.ok(s.world.entities.every(e => e.anchor.frame === 'ship:scout'))
    assert.ok(drawList(s.world).every(e => e.entity.anchor.frame === 'ship:scout'))
    assert.deepEqual(capturePlayerLocation(s.world), currentPassenger)
    arrive(address)
    assert.equal(n.clock.state, s.world.scoutLocation.ship)
    assert.equal(s.world.scoutLocation.body.id, address.bodyId)
    assert.equal(s.world.player.mode, 'standing')
    assert.equal(s.world.cockpit.view, 'world')
    assert.equal(capturePlayerLocation(s.world).anchor.frame, 'ship:scout', 'landing keeps the pilot in the cabin')
    assert.equal(s.world.entities.filter(e => e.id === 'ship:scout').length, 1)
    assert.ok(s.world.entities.every(e => !e.source))
    assert.equal(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1 }), false)
    const restored = new ResourceSession(storage, createTopDownState(), undefined, undefined, () => 0)
    assert.equal(restored.error, ''); assert.deepEqual(restored.snapshot, s.snapshot)
    assert.deepEqual(restored.world.scoutLocation, s.world.scoutLocation)
    assert.deepEqual(restored.world.entities.filter(e => e.anchor.frame === 'planet:fixture'), s.world.entities.filter(e => e.anchor.frame === 'planet:fixture'))
    // Walk the actual collision/traversal path from cabin across the doorway.
    s.world = place(leaveCockpit(s.world), { frame: 'ship:scout', point: point2(0, 4.7) })
    assert.equal(s.toggleBoardingDoor(s.snapshot.revision), true, s.feedback)
    const angle = s.world.frames.ship.angle
    for (let i = 0; i < 80; i++) s.world = stepTopDown(s.world, [-Math.sin(angle), Math.cos(angle)])
    assert.equal(capturePlayerLocation(s.world).anchor.frame, 'planet:fixture')
    s.world = place(s.world, { frame: 'ship:scout', point: point2(0, 4.7) })
    assert.equal(s.toggleBoardingDoor(s.snapshot.revision), true)
    s.world = enterCockpit({ ...place(s.world, passenger.anchor), selectedId: 'scout:chair' })
  }
  launch(); arrive(startingSurface())
  assert.deepEqual(s.world.entities.filter(e => e.anchor.frame === 'planet:fixture'), original)
  assert.equal(s.snapshot.sources['source:landing:mineral:01']!.remainingGrams, 59000)
  assert.equal(s.snapshot.inventory.quantities.mineral, 1000)
  assert.equal(Object.keys(s.snapshot.sites).length, 4)
  assert.equal(new ResourceSession(storage, createTopDownState(), undefined, undefined, () => 0).error, '')
})
test('AC-4 generated-site mid-launch and landing snapshots restore; legacy v10 migrates without changing bytes or ledger', () => {
  const { s, n, storage, launch, arrive, approach, action } = setup()
  launch(); arrive(addressFor('vesper-moon', [0, -1, 0]))
  s.world = enterCockpit({ ...s.world, selectedId: 'scout:chair' })
  assert.equal(action({ kind: 'begin-takeoff' }), true); n.advance(2); s.checkpoint()
  const restored = new ResourceSession(storage, createTopDownState(), undefined, undefined, () => 0)
  assert.equal(restored.error, '')
  n.advance(4); new NavigationRuntime(SAMPLE_BODIES, restored.world.scoutLocation).advance(4)
  assert.deepEqual(restored.world.scoutLocation.ship, s.world.scoutLocation.ship)
  assert.ok(Math.hypot(...relativeM(n.clock.state.position, surfacePosition(resolveSurface(s.world.scoutLocation.surface)))) > 49)
  s.world = enterCockpit({ ...s.world, selectedId: 'scout:chair' })
  approach(addressFor('caligo', [1, 1, 0])); assert.equal(action({ kind: 'begin-landing', bodyId: 'caligo', surface: addressFor('caligo', [1, 1, 0]) }), true)
  assert.equal(new ResourceSession(storage, createTopDownState(), undefined, undefined, () => 0).error, '')
  const old = setup(); old.s.upgrade(12000, 15)
  old.corrupt(v => { v.version = 10; delete v.sites; delete v.scout.surface })
  const raw = old.storage.getItem(), migrated = new ResourceSession(old.storage, createTopDownState(), undefined, undefined, () => 0)
  assert.equal(migrated.error, ''); assert.equal(old.storage.getItem(), raw)
  assert.deepEqual(migrated.snapshot.sources, old.s.snapshot.sources); assert.deepEqual(migrated.snapshot.inventory, old.s.snapshot.inventory)
  migrated.checkpoint(); assert.equal(old.storage.getItem(), raw, 'unchanged migration needs no write')
  migrated.upgrade(13000, 16); assert.equal(JSON.parse(old.storage.getItem()!).version, 11)
})
test('AC-5 malformed/unsupported surfaces, unsafe footprints, crash and incomplete/mismatched mapping fail atomically', () => {
  for (const surface of [addressFor('nacre', [1, 0, 0]), addressFor('relay', [0, 1, 0]), { ...startingSurface(), bodyId: 'missing' }, { ...startingSurface(), radialM: [0, 0, 0] }, { ...startingSurface(), yaw: NaN }, { ...startingSurface(), radialM: [Infinity, 0, 0] }, { ...startingSurface(), radialM: [2, 3] }]) assert.throws(() => resolveSurface(surface as SurfaceAddress))
  const unsafe = { ...startingSurface(), radialM: [-9, Math.sqrt(physicalRadiusM(SAMPLE_BODIES[1]!) ** 2 - 81), 0] as const }
  assert.throws(() => surfaceScoutLocation(unsafe), /Unsafe landing/)
  const { s, n, launch, action, fail, storage } = setup(); launch()
  for (const command of [{ kind: 'begin-landing', bodyId: 'relay' }, { kind: 'begin-landing', bodyId: 'caligo', surface: startingSurface() }, { kind: 'begin-landing', bodyId: 'vesper', siteId: 'missing', surface: startingSurface() }] as const) {
    const before = captureScout(s.world.scoutLocation), raw = storage.getItem()
    assert.equal(action(command), false); assert.deepEqual(captureScout(s.world.scoutLocation), before); assert.equal(storage.getItem(), raw)
  }
  const before = s.world
  fail(); assert.equal(action({ kind: 'begin-landing', bodyId: 'caligo', surface: addressFor('caligo', [1, 0, 0]) }), false)
  assert.equal(s.world, before); assert.equal(s.world.scoutLocation.body.id, 'vesper')
  const crash = { ...s.world.scoutLocation, ship: { ...n.clock.state, mode: 'recovering' as const, contact: { bodyId: 'vesper', outcome: 'crash' as const, impactSpeedMps: 30, normal: [0, 1, 0] as const, ageTicks: 60 } } }
  assert.throws(() => transitionScout(crash, { kind: 'begin-landing', bodyId: 'vesper', surface: startingSurface() }))
})
test('AC-5 saved site tampering, missing mapping and blocked restored footprints are rejected', () => {
  for (const change of [
    (v: any) => { delete v.scout.surface }, (v: any) => { v.siteId = 'other' },
    (v: any) => { v.sites[STARTING_SITE.id].address.bodyId = 'caligo' },
    (v: any) => { v.sites[STARTING_SITE.id].anchors.loot = [0, 0] },
    (v: any) => { v.sites[STARTING_SITE.id].anchors['rock:west'] = [2, -1] },
    (v: any) => { delete v.sites[STARTING_SITE.id].anchors['source:landing:mineral:01'] },
  ]) {
    const { s, corrupt, storage } = setup(); s.upgrade(10000, 12); corrupt(change)
    const raw = storage.getItem(), restored = new ResourceSession(storage, createTopDownState())
    assert.match(restored.error, /safely/); restored.checkpoint(); assert.equal(storage.getItem(), raw)
  }

})

test('AC-3 detached planet-frame operations reject even stale same-frame anchors; ship-local movement remains live', () => {
  const { s, launch } = setup(); launch()
  const planet: Anchor = { frame: 'planet:fixture', point: point2(0, 0) }
  assert.throws(() => convertAnchor(planet, 'planet:fixture', s.world.frames), /unavailable/)
  assert.doesNotThrow(() => convertAnchor({ frame: 'ship:scout', point: point2(0, 2) }, 'ship:scout', s.world.frames))
})
test('AC-5 nonfinite admission and saved obstacles reject before publishing a landing plan', () => {
  for (const part of ['position', 'velocity'] as const) {
    const { s, n, launch, action } = setup(); launch()
    n.clock.state = part === 'velocity' ? { ...n.clock.state, velocityMps: [NaN, 0, 0] }
      : { ...n.clock.state, position: { ...n.clock.state.position, offsetM: [NaN, 0, 0] } }
    assert.equal(action({ kind: 'begin-landing', bodyId: 'vesper', surface: startingSurface() }), false)
    assert.equal(s.world.scoutLocation.binding, null); assert.equal(s.world.scoutLocation.landing, null)
  }
  const { s, launch, action } = setup(); launch()
  const original = s.snapshot.sites[STARTING_SITE.id]!
  s.snapshot = { ...s.snapshot, sites: { ...s.snapshot.sites, [STARTING_SITE.id]: { ...original, anchors: { ...original.anchors, 'rock:west': [2, -1] } } } }
  const world = s.world, ship = s.world.scoutLocation.ship
  assert.equal(action({ kind: 'begin-landing', bodyId: 'vesper', surface: startingSurface() }), false)
  assert.match(s.feedback, /Unsafe landing/); assert.equal(s.world, world); assert.equal(s.world.scoutLocation.ship, ship)
  assert.equal(s.world.scoutLocation.binding, null); assert.equal(s.world.scoutLocation.landing, null)
})
