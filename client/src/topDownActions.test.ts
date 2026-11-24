import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createTopDownState, PLAYER_ID, type TopDownState } from './topDownRuntime.ts'
import { interactiveAt, groundFromView } from './topDownNavigation.ts'
import { point2, relativeAnchor, worldToView } from './topDownFrames.ts'
import { executeObjectAction, objectActions } from './topDownActions.ts'
import { ResourceSession } from './resourceSession.ts'

const anchor = (x: number, y: number) => ({ frame: 'ship:scout' as const, point: point2(x, y) })
const place = (world: TopDownState, x: number, y: number): TopDownState => ({ ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: anchor(x, y) } : e) })
const make = (world = createTopDownState('interior', true)) => new ResourceSession({ getItem: () => null, setItem: () => {} }, world)
test('picking follows rendered overlap order regardless of entity storage order, sprite bounds and zoom', () => {
  const base = createTopDownState('interior', true), chair = base.entities.find(e => e.id === 'scout:chair')!
  for (const scale of [14, 26, 64]) {
    const world = { ...base, camera: { ...base.camera, pixelsPerM: scale }, entities: [...base.entities, { ...chair, id: 'overlap:a' }, { ...chair, id: 'overlap:z' }].reverse() }
    const p = worldToView(relativeAnchor(chair.anchor, world.camera.anchor, world.frames), world.frames)
    assert.equal(interactiveAt(world, groundFromView(world, [p[0] * scale, p[1] * scale])), 'scout:chair')
    assert.equal(interactiveAt(world, anchor(0, -3.5), chair.id), chair.id) // selectable sprite outside footprint
    assert.equal(interactiveAt(world, anchor(20, 20)), null)
  }
})
test('hidden/detached and culled coordinates or forged sprite IDs never grant picking/actions', () => {
  const base = createTopDownState('interior', true)
  const hidden = { ...base.entities.find(e => e.interaction)!, id: 'hidden', anchor: { frame: 'planet:fixture' as const, point: point2(0, 0) } }
  const far = { ...hidden, id: 'far', anchor: anchor(1000, 0) }
  const world = { ...base, frames: { ...base.frames, ship: { ...base.frames.ship, parent: null } }, entities: [...base.entities, hidden, far] }
  assert.equal(interactiveAt(world, hidden.anchor, hidden.id), null)
  assert.equal(interactiveAt(world, far.anchor, far.id), null)
  assert.deepEqual(objectActions({ world, paused: false }, hidden.id), [])
  assert.deepEqual(objectActions({ world, paused: false }, 'missing'), [])
})
test('remote Inspect is label-only; physical range, power, pause and stale targets are revalidated', () => {
  let world = createTopDownState('interior', true)
  const get = () => ({ world, paused: false })
  assert.equal(objectActions(get(), 'scout:chair')[0]!.available, true)
  assert.equal(objectActions(get(), 'scout:chair')[1]!.requiresApproach, true)
  assert.equal(objectActions(get(), 'scout:chair')[1]!.available, true)
  assert.match(objectActions({ ...get(), phase: 'execution' }, 'scout:chair')[1]!.reason!, /reach/)
  world = place(world, 0, -3.1)
  assert.equal(objectActions(get(), 'scout:chair')[1]!.available, true)
  let calls = 0
  const handlers = { inspect: () => { calls++ }, interact: () => { calls++ } }
  world = { ...world, cockpit: { ...world.cockpit, powered: false } }
  assert.equal(executeObjectAction(get(), 'scout:chair', 'cockpit', handlers), false)
  assert.equal(executeObjectAction({ ...get(), paused: true }, 'scout:chair', 'inspect', handlers), false)
  world = { ...world, entities: world.entities.filter(e => e.id !== 'scout:chair') }
  assert.equal(executeObjectAction(get(), 'scout:chair', 'inspect', handlers), false)
  assert.equal(calls, 0)
})
test('registry exposes only implemented capability and tool, exhaustion, capacity and door guards', () => {
  const session = make()
  let world = session.world
  const source = world.entities.find(e => e.source)!
  world = { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: source.anchor } : e) }
  const context = () => ({ world, session, paused: false })
  assert.match(objectActions(context(), source.id)[1]!.reason!, /tool/)
  session.toggleTool()
  assert.equal(objectActions(context(), source.id)[1]!.available, true)
  session.survival = { ...session.survival, fatigue: 0 }
  assert.match(objectActions(context(), source.id)[1]!.reason!, /exhausted/)
  session.survival = { ...session.survival, fatigue: 100_000 }
  session.snapshot = { ...session.snapshot, inventory: { ...session.snapshot.inventory, quantities: { mineral: 10_000 } } }
  assert.match(objectActions(context(), source.id)[1]!.reason!, /weight/)
  world = place(world, 0, 5.5)
  assert.match(objectActions(context(), 'scout:entry')[1]!.reason!, /occupied/)
  assert.deepEqual(objectActions({ world, paused: false }, 'scout:core').map(a => a.id), ['inspect'])
  assert.deepEqual(objectActions(context(), 'scout:core').map(a => a.id), ['inspect', 'core'])
})
