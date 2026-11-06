import { test } from 'node:test'
import assert from 'node:assert/strict'
import { distances } from './contact.ts'
import { ORIGIN, relativeM } from './coordinates.ts'
import { enterCockpit, leaveCockpit } from './cockpit.ts'
import { NavigationRuntime } from './navigationRuntime.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { ResourceSession, RESOURCE_SAVE_KEY } from './resourceSession.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { alignTo, setThrottle } from './simulation.ts'
import { point2 } from './topDownFrames.ts'
import { capturePlayerLocation, createTopDownState, PLAYER_ID, stepTopDown } from './topDownRuntime.ts'

test('one deterministic Scout surface anchor binds 2D frames, lifecycle, Vesper and physical radar contact', () => {
  const world = createTopDownState('exterior', true), location = world.scoutLocation
  const runtime = new NavigationRuntime(SAMPLE_BODIES, location)
  assert.equal(world.frames, location.frames)
  assert.equal(location.siteId, STARTING_SITE.id)
  assert.deepEqual(world.frames.ship.origin, STARTING_SITE.shipOrigin)
  assert.deepEqual(capturePlayerLocation(world).anchor, STARTING_SITE.spawn)
  assert.equal(location.body.id, 'vesper')
  assert.equal(runtime.clock.state, location.ship)
  assert.equal(runtime.clock.state.mode, 'grounded')
  assert.notDeepEqual(runtime.clock.state.position, ORIGIN)
  assert.ok(Math.abs(distances(runtime.clock.state, location.body).surfaceDistanceM) < .0001)
  const [x, y, z] = relativeM(location.ship.position, location.body.position)
  assert.equal(x, 2); assert.equal(z, -1); assert.ok(y > location.body.radiusKm * 1000)
  assert.throws(() => new NavigationRuntime([], location), /Surface body/)
})

test('ticks, steering, throttle, alignment, reset and repeated cockpit entry cannot relocate a grounded Scout', () => {
  let world = createTopDownState('interior', true)
  world = { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout', point: point2(0, -3.1) } } : e) }
  const runtime = new NavigationRuntime(SAMPLE_BODIES, world.scoutLocation), original = runtime.clock.state
  for (let visit = 0; visit < 3; visit++) {
    world = enterCockpit({ ...world, selectedId: 'scout:chair' })
    assert.equal(world.cockpit.view, 'radar')
    runtime.clock.state = setThrottle(alignTo(runtime.clock.state, [1, 0, 0]), 100)
    runtime.advance(4, { up: true, down: false, left: true, right: false })
    runtime.advance(2)
    const tick = runtime.clock.state.tick, scanner = runtime.scanner
    runtime.reset()
    assert.equal(runtime.clock.state.tick, tick); assert.equal(runtime.scanner, scanner)
    assert.deepEqual(runtime.clock.state, { ...original, tick })
    assert.ok(runtime.scanner.contacts.some(b => b.id === 'vesper'))
    world = stepTopDown(leaveCockpit(world), [0, 0])
    assert.equal(world.frames, world.scoutLocation.frames)
    assert.equal(world.scoutLocation.ship, original)
    assert.equal(world.player.mode, 'standing')
  }
})

test('local checkpoints restore outdoor and ship-local players with the same surface and lifecycle', () => {
  for (const view of ['exterior', 'interior'] as const) {
    const data = new Map<string, string>()
    const storage = { getItem: (key: string) => data.get(key) ?? null, setItem: (key: string, value: string) => { data.set(key, value) } }
    const session = new ResourceSession(storage, createTopDownState(view, true))
    session.world = stepTopDown(session.world, [1, 0])
    session.checkpoint()
    assert.ok(data.has(RESOURCE_SAVE_KEY))
    const restored = new ResourceSession(storage, createTopDownState('exterior', true))
    assert.equal(restored.error, '')
    assert.deepEqual(capturePlayerLocation(restored.world), capturePlayerLocation(session.world))
    assert.deepEqual(restored.world.scoutLocation, session.world.scoutLocation)
    const runtime = new NavigationRuntime(SAMPLE_BODIES, restored.world.scoutLocation)
    runtime.advance(4); runtime.advance(4)
    assert.equal(runtime.clock.state.mode, 'grounded')
    assert.deepEqual(runtime.clock.state.position, session.world.scoutLocation.ship.position)
    assert.equal(runtime.clock.state.contact!.bodyId, 'vesper')
  }
})
