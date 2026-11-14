import { test } from 'node:test'
import assert from 'node:assert/strict'
import { enterCockpit, LOCAL_PLAYER } from './cockpit.ts'
import { translate } from './coordinates.ts'
import { landingReadout, landingProgress } from './landingHud.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { ResourceSession } from './resourceSession.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { point2 } from './topDownFrames.ts'
import { createTopDownState, PLAYER_ID, stepTopDown, requestInteraction } from './topDownRuntime.ts'

function setup() {
  let raw: string | null = null
  const storage = { getItem: () => raw, setItem: (_: string, v: string) => { raw = v } }
  let world = createTopDownState('interior', true)
  world.scoutLocation.boardingDoor = 'closed'
  world = enterCockpit({ ...world, selectedId: 'scout:chair', entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout', point: point2(0, -3.1) } } : e) })
  const s = new ResourceSession(storage, world), n = new NavigationRuntime(SAMPLE_BODIES, world.scoutLocation)
  assert.equal(s.transitionScout({ kind: 'begin-takeoff' }, s.snapshot.revision, s.world.cockpit), true)
  n.advance(4); n.advance(2)
  return { s, n, storage }
}
const land = { kind: 'begin-landing', bodyId: 'vesper' } as const
test('landing HUD exposes only detected contacts, full relative speed, altitude and real progress', () => {
  const { n, s } = setup()
  assert.equal(landingReadout(n).body?.id, 'vesper')
  assert.ok(landingReadout(n).altitudeM! > 0)
  n.clock.state = { ...n.clock.state, velocityMps: [12, 16, 0] }
  assert.equal(landingReadout(n).relativeSpeedMps, 20)
  assert.equal(s.scoutActionDenial(land), null)
  n.scanner.select('nacre')
  assert.equal(landingReadout(n).body?.id, 'nacre')
  assert.match(s.scoutActionDenial({ ...land, bodyId: 'nacre' })!, /sampled planet or moon/)
  n.scanner.contacts = []; n.scanner.selected = null
  assert.equal(landingReadout(n).body, undefined)
  assert.equal(landingReadout(n).relativeSpeedMps, null)
  n.clock.state = { ...n.clock.state, velocityMps: [0, 0, 0] }
  assert.equal(s.transitionScout(land, s.snapshot.revision, s.world.cockpit), true)
  assert.match(landingProgress(n)!, /Descent · 0%/)
  n.advance(.1)
  assert.match(s.scoutActionDenial(land)!, /maneuver is already running/)
})
test('admission readout and command reject without mutating world, ledger, feedback or save', () => {
  for (const [change, reason] of [
    [(s: ResourceSession) => { s.world.scoutLocation.ship = { ...s.world.scoutLocation.ship, velocityMps: [0, 0, 20.01] } }, /20 m\/s/],
    [(s: ResourceSession) => { s.world.scoutLocation.scanner!.contactIds = [] }, /Scan/],
    [(s: ResourceSession) => { const l = s.world.scoutLocation; l.ship = { ...l.ship, position: translate(l.ship.position, [0, 1e6, 0]) } }, /0.15/],
    [(s: ResourceSession) => { s.world = { ...s.world, cockpit: { ...s.world.cockpit, ownerId: 'other' } } }, /authorized/],
    [(s: ResourceSession) => { s.world.scoutLocation.ship = { ...s.world.scoutLocation.ship, pitch: NaN } }, /Invalid landing state/],
  ] as const) {
    const { s, storage } = setup(); change(s)
    const before = structuredClone(s.world), snapshot = structuredClone(s.snapshot), raw = storage.getItem(), feedback = s.feedback
    assert.match(s.scoutActionDenial(land)!, reason)
    assert.deepEqual(s.world, before); assert.deepEqual(s.snapshot, snapshot); assert.equal(storage.getItem(), raw); assert.equal(s.feedback, feedback)
    assert.equal(s.transitionScout(land, s.snapshot.revision, s.world.cockpit), false)
    assert.match(s.feedback, reason); assert.deepEqual(s.world, before); assert.deepEqual(s.snapshot, snapshot); assert.equal(storage.getItem(), raw)
  }
})
test('verified touchdown returns standing inside and persists destination immediately', () => {
  const { s, n, storage } = setup(), camera = s.world.camera
  assert.equal(s.transitionScout(land, s.snapshot.revision, s.world.cockpit), true)
  while (s.world.scoutLocation.landing) {
    n.advance(1 / 60); s.advance(stepTopDown(s.world, [0, 0]))
  }
  assert.equal(s.world.cockpit.view, 'world'); assert.equal(s.world.cockpit.occupantId, null)
  assert.equal(s.world.player.mode, 'standing'); assert.equal(s.world.camera.anchor.frame, camera.anchor.frame)
  assert.equal(s.world.entities.find(e => e.id === PLAYER_ID)!.anchor.frame, 'ship:scout')
  const restored = new ResourceSession(storage, createTopDownState())
  assert.equal(restored.error, '')
  assert.equal(restored.world.scoutLocation.siteId, s.world.scoutLocation.siteId)
  assert.equal(restored.world.scoutLocation.binding, s.world.scoutLocation.binding)
  assert.equal(restored.world.scoutLocation.ship.mode, 'grounded')
  assert.equal(restored.world.cockpit.ownerId, LOCAL_PLAYER)
})
test('destroyed whole ship cannot walk, interact or admit a surface landing', () => {
  const { s } = setup()
  s.world.scoutLocation.ship = { ...s.world.scoutLocation.ship, mode: 'destroyed' }
  assert.match(s.scoutActionDenial(land)!, /whole ship destroyed/)
  const player = s.world.entities.find(e => e.id === PLAYER_ID)!
  s.world = { ...s.world, player: { ...s.world.player, mode: 'standing' } }
  assert.deepEqual(stepTopDown(s.world, [1, 0]).entities.find(e => e.id === PLAYER_ID)!.anchor, player.anchor)
  assert.match(requestInteraction(s.world).feedback, /whole ship destroyed/)
})
