import { test } from 'node:test'
import assert from 'node:assert/strict'
import { physicalRadiusM, resolveContacts, surfaceVelocity } from './contact.ts'
import { ORIGIN, relativeM, translate } from './coordinates.ts'
import { add, length, normal, scale, sub } from './math.ts'
import { planLanding, sampleLanding, rotateLanding, LANDING_MAX_ACCELERATION } from './landing.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { SAMPLE_BODIES, type Body } from './sampleUniverse.ts'
import { captureScout, decodeScout } from './scoutSave.ts'
import { startingScoutLocation, surfaceScoutLocation, transitionScout } from './scoutLocation.ts'
import { startingSurface } from './surfaceSite.ts'
import { alignTo, idleControls, initialShip, setThrottle, STEP_SECONDS } from './simulation.ts'
const serialize = (v: unknown) => JSON.parse(JSON.stringify(v, (_, v) => typeof v === 'bigint' ? String(v) : v))
function approach(height = 100, velocity = [0, 0, 0] as readonly [number, number, number]) {
  const l = startingScoutLocation()
  l.boardingDoor = 'closed'; l.binding = null
  l.frames = { ...l.frames, ship: { ...l.frames.ship, parent: null, origin: ORIGIN } }
  l.ship = { ...l.ship, mode: 'flying', contact: null, position: translate(l.ship.position, scale(normal(l.surface.radialM), height)), velocityMps: velocity, tick: 300 }
  l.scanner = { lastTick: 300, pulse: 1, selected: 'vesper', contactIds: ['vesper'] }
  const n = new NavigationRuntime(SAMPLE_BODIES, l)
  const begin = () => Object.assign(l, transitionScout(l, { kind: 'begin-landing', bodyId: 'vesper', surface: startingSurface() }))
  return { l, n, begin }
}
test('AC-1 exact range/speed boundaries, any heading, scanner authority and atomic rejection', () => {
  for (const yaw of [0, 1.2, -Math.PI]) {
    const { l, begin } = approach(100, [20, 0, 0]); l.ship = { ...l.ship, yaw, pitch: 1 }
    assert.doesNotThrow(begin); assert.equal(l.ship.mode, 'landing')
  }
  const edge = approach(.15 * SAMPLE_BODIES[1]!.radiusKm * 1000); assert.doesNotThrow(edge.begin)
  for (const fixture of [approach(.15 * SAMPLE_BODIES[1]!.radiusKm * 1000 + .01), approach(100, [20.001, 0, 0])]) {
    const before = structuredClone(fixture.l); assert.throws(fixture.begin); assert.deepEqual(fixture.l, before)
  }
  const hidden = approach(); hidden.l.scanner!.contactIds = []
  assert.throws(hidden.begin, /Scan/)
  const mismatch = approach()
  assert.throws(() => transitionScout(mismatch.l, { kind: 'begin-landing', bodyId: 'caligo', surface: startingSurface() }))
  assert.doesNotThrow(() => transitionScout(approach().l, { kind: 'begin-landing', bodyId: 'vesper' }))
})
test('AC-2 continuous derivatives, bounded speed/acceleration, safe hull and automatic touchdown', () => {
  const { l, n, begin } = approach(100, [2, -3, 0]), initial = l.ship
  begin(); const plan = l.landing!, duration = plan.durationTicks * STEP_SECONDS
  assert.ok(duration > 8)
  assert.ok(length(relativeM(sampleLanding(plan, l.body, plan.startTick).position, initial.position)) < 1e-7)
  let previous = initial
  for (let i = 1; i <= plan.durationTicks; i++) {
    n.advance(STEP_SECONDS, { ...idleControls(), up: true, left: true })
    const next = l.ship
    assert.ok(length(relativeM(next.position, l.body.position)) >= physicalRadiusM(l.body) - 1e-7)
    assert.ok(length(sub(next.velocityMps, previous.velocityMps)) / STEP_SECONDS <= LANDING_MAX_ACCELERATION + 1e-7)
    assert.ok(length(next.velocityMps) <= 100)
    if (i < plan.durationTicks - 1) {
      const h = 1, a = sampleLanding(plan, l.body, next.tick - h), b = sampleLanding(plan, l.body, next.tick + h)
      assert.ok(length(sub(scale(relativeM(b.position, a.position), 1 / (2 * h * STEP_SECONDS)), next.velocityMps)) < .002)
    }
    previous = next
  }
  assert.equal(l.ship.mode, 'grounded'); assert.equal(l.binding, surfaceScoutLocation(l.surface).binding)
  assert.equal(l.landing, null); assert.equal(l.maneuver, null); assert.equal(l.scanner?.lastTick, l.ship.tick)
  assert.deepEqual(l.ship.velocityMps, [0, 0, 0]); assert.equal(l.ship.throttle, 0)
  const pose = l.ship.position; n.advance(1); assert.deepEqual(l.ship.position, pose)
})
test('AC-2 a 930 km descent uses hours, never a fixed cinematic', () => {
  const { l, begin } = approach(930000); begin()
  const p = l.landing!, seconds = p.durationTicks * STEP_SECONDS
  assert.ok(seconds > 30000 && seconds <= 43200)
  for (const u of [0, .1, .5, .9, 1]) {
    const s = sampleLanding(p, l.body, p.startTick + p.durationTicks * u)
    assert.ok(length(s.velocityMps) <= 100)
    assert.ok(length(relativeM(s.position, l.body.position)) >= physicalRadiusM(l.body) - 1e-7)
  }
})
test('AC-3 manual controls, duplicate/new assists and finish override cannot cancel admitted landing', () => {
  const { l, n, begin } = approach(); begin(); const plan = l.landing
  assert.throws(begin, /maneuver/)
  assert.throws(() => transitionScout(l, { kind: 'begin-takeoff' }), /already running/)
  assert.throws(() => transitionScout(l, { kind: 'finish-landing' }), /automatically/)
  l.ship = alignTo(setThrottle(l.ship, 100), [1, 0, 0]); n.advance(1, { ...idleControls(), left: true })
  assert.equal(l.landing, plan); assert.equal(l.ship.throttle, 0); assert.equal(l.ship.alignment, null)
})
test('AC-4 partitioned elapsed time, durable resume and tampered plans fail closed', () => {
  const a = approach(), b = approach(); a.begin(); b.begin()
  for (let i = 0; i < 120; i++) a.n.advance(1 / 120)
  for (let i = 0; i < 10; i++) b.n.advance(.1)
  assert.deepEqual(a.l, b.l)
  const saved = serialize(captureScout(a.l)), restored = decodeScout(saved), runtime = new NavigationRuntime(SAMPLE_BODIES, restored)
  while (a.l.landing) { a.n.advance(.1); runtime.advance(.1) }
  assert.deepEqual(restored, a.l)
  for (const change of [(s: any) => { s.landing.durationTicks++ }, (s: any) => { s.landing.anchor.velocityMps[0] = 21 }, (s: any) => { s.ship.position.offsetM[0]++ }, (s: any) => { s.landing.surface.bodyId = 'caligo' }, (s: any) => { delete s.landing }]) {
    const forged = structuredClone(saved); change(forged); assert.throws(() => decodeScout(forged))
  }
})
test('AC-4 rotating moving surface retains inertial velocity and matches position derivative at handoff', () => {
  const body: Body = { id: 'spin', name: 'spin', kind: 'moon', position: { cell: [10n ** 30n, -(10n ** 31n), 10n ** 32n], offsetM: [123, 456, 789] }, radiusKm: 10, angularVelocityRadps: [0, 0, .0002] }
  const radial = [physicalRadiusM(body) + 100, 0, 0] as const
  const anchor = { ...initialShip(), position: translate(body.position, radial), velocityMps: add(surfaceVelocity(body, radial), [-1, 0, 0]) }
  const surface = { bodyId: body.id, radialM: [physicalRadiusM(body), 0, 0] as const, yaw: 1 }
  const plan = planLanding(anchor, body, surface, [body]), endTick = plan.startTick + plan.durationTicks
  const end = sampleLanding(plan, body, endTick), final = { ...surface, radialM: rotateLanding(surface.radialM, body, endTick * STEP_SECONDS) }
  assert.ok(length(relativeM(end.position, translate(body.position, final.radialM))) < 1e-6)
  assert.ok(length(end.velocityMps) > 1)
  assert.ok(length(sub(end.velocityMps, surfaceVelocity(body, final.radialM))) < 1e-9)
  for (const u of [.1, .5, .99]) {
    const tick = plan.startTick + plan.durationTicks * u, h = 1
    const a = sampleLanding(plan, body, tick - h), b = sampleLanding(plan, body, tick + h), s = sampleLanding(plan, body, tick)
    assert.ok(length(sub(scale(relativeM(b.position, a.position), 1 / (2 * h * STEP_SECONDS)), s.velocityMps)) < .002)
  }
  assert.throws(() => planLanding({ ...anchor, velocityMps: [0, 23, 0] }, body, surface, [body]), /20 m\/s/)
})
test('AC-4/5 unsafe near-contact braking, invalid sites, other bodies and terminal impact never become successful landing', () => {
  const near = approach(.01, [0, -20, 0]); assert.throws(near.begin, /Unsafe landing/)
  const touched = approach(0, [0, 0, 0]); touched.begin(); while (touched.l.landing) touched.n.advance(.1)
  assert.equal(touched.l.ship.mode, 'grounded')
  const { l } = approach()
  const obstacle: Body = { id: 'obstacle', name: 'obstacle', kind: 'asteroid', radiusKm: .001, position: translate(l.ship.position, [0, -20, 0]) }
  assert.throws(() => planLanding(l.ship, l.body, l.surface, [...SAMPLE_BODIES, obstacle]), /Unsafe/)
  for (const surface of [{ ...l.surface, radialM: [0, 0, 0] as const }, { ...l.surface, bodyId: 'caligo' }]) assert.throws(() => planLanding(l.ship, l.body, surface, SAMPLE_BODIES))
  l.ship = { ...l.ship, velocityMps: [0, -100, 0], position: translate(startingScoutLocation().ship.position, [0, .1, 0]) }
  const n = new NavigationRuntime(SAMPLE_BODIES, l); n.advance(STEP_SECONDS)
  assert.equal(l.ship.mode, 'destroyed'); assert.equal(l.binding, null)
  assert.throws(() => transitionScout(l, { kind: 'begin-landing', bodyId: 'vesper' }))
  const slow = approach(.1, [0, -1, 0]); slow.n.advance(1)
  assert.equal(slow.l.ship.mode, 'recovering'); assert.equal(slow.l.binding, null); assert.throws(slow.begin)
  const fixtureBefore = { ...l.ship, mode: 'flying' as const, contact: null, velocityMps: [0, -100, 0] as const, position: translate(startingScoutLocation().ship.position, [0, .1, 0]) }
  assert.equal(resolveContacts(fixtureBefore, { ...fixtureBefore, position: translate(startingScoutLocation().ship.position, [0, -1, 0]) }, SAMPLE_BODIES).mode, 'destroyed')
})

test('AC-2/4 runtime rotating-body touchdown keeps the body-fixed site and continues surface motion', () => {
  const body: Body = { id: 'spin', name: 'Spin', kind: 'moon', position: ORIGIN, radiusKm: 10, angularVelocityRadps: [0, 0, .0002] }
  const surface = { bodyId: body.id, radialM: [physicalRadiusM(body), 0, 0] as const, yaw: .3 }
  const l = surfaceScoutLocation(surface, [body], 300)
  l.binding = null; l.boardingDoor = 'closed'
  l.frames = { ...l.frames, ship: { ...l.frames.ship, parent: null, origin: ORIGIN } }
  const radial = relativeM(l.ship.position, body.position)
  l.ship = { ...l.ship, mode: 'flying', contact: null, position: translate(l.ship.position, scale(normal(radial), 100)) }
  l.scanner = { lastTick: 300, pulse: 1, selected: body.id, contactIds: [body.id] }
  const n = new NavigationRuntime([body], l)
  Object.assign(l, transitionScout(l, { kind: 'begin-landing', bodyId: body.id, surface }, [body]))
  while (l.landing) n.advance(STEP_SECONDS)
  const siteId = l.siteId, before = l.ship
  n.advance(STEP_SECONDS)
  assert.equal(l.ship.mode, 'grounded'); assert.equal(l.siteId, siteId)
  assert.deepEqual(l.surface, surface)
  const expected = surfaceScoutLocation(surface, [body], l.ship.tick).ship
  assert.deepEqual(l.ship, expected)
  assert.deepEqual(l.frames.planet, surfaceScoutLocation(surface, [body], l.ship.tick).frames.planet)
  assert.ok(length(sub(scale(relativeM(l.ship.position, before.position), 1 / STEP_SECONDS), l.ship.velocityMps)) < .0001)
})
