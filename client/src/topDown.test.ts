import { test } from 'node:test'
import assert from 'node:assert/strict'
import { CELL_SIZE_M, ORIGIN, relativeM, translate } from './coordinates.ts'
import { convertAnchor, inPlanet, point2, relativeAnchor, rotate2, type Anchor } from './topDownFrames.ts'
import { allowedMove, interactionInReach, nearbyObject, PLAYER_RADIUS_M } from './topDownInteraction.ts'
import { cancelInteraction, createTopDownState, drawList, LAYERS, PLAYER_ID, requestInteraction, stepTopDown, TopDownClock } from './topDownRuntime.ts'

const near = (a: readonly number[], b: readonly number[], tolerance = 1e-8) => a.forEach((v, i) => assert.ok(Math.abs(v - b[i]) < tolerance, `${a} != ${b}`))
test('ship-local points round trip through rotated and translated planet poses', () => {
  for (const angle of [0, Math.PI / 2, -.71, Math.PI]) {
    const state = createTopDownState(), frames = { ...state.frames, ship: { ...state.frames.ship, origin: point2(-600, 300), angle } }
    const anchor: Anchor = { frame: 'ship:scout', point: point2(1.25, -2.75) }
    const planet = convertAnchor(anchor, 'planet:fixture', frames)
    near(relativeM(convertAnchor(planet, 'ship:scout', frames).point, anchor.point), [0, 0, 0])
    assert.equal(convertAnchor(anchor, anchor.frame, frames), anchor)
  }
})
test('passengers retain identity and anchors while pose changes; terrain stays planet-fixed', () => {
  const state = createTopDownState(), player = state.entities.find(e => e.id === PLAYER_ID)!, rock = state.entities.find(e => e.id === 'rock:west')!
  const moved = { ...state.frames, ship: { ...state.frames.ship, origin: point2(50, 70), angle: Math.PI / 2 } }
  near(relativeM(inPlanet(player.anchor, moved), ORIGIN), [48, 70, 0])
  assert.deepEqual(inPlanet(rock.anchor, moved), inPlanet(rock.anchor, state.frames))
  assert.equal(player.anchor.frame, 'ship:scout')
  assert.equal(player.id, PLAYER_ID)
})
test('camera-relative transforms cancel astronomical roots and retain sub-mm offsets across cells', () => {
  const state = createTopDownState()
  const shipOrigin = { cell: [10n ** 25n, -(10n ** 27n), 0n] as const, offsetM: [CELL_SIZE_M / 2 - 1, 0, 0] as const }
  const frames = { ...state.frames, ship: { ...state.frames.ship, origin: shipOrigin, angle: 0 } }
  const player: Anchor = { frame: 'ship:scout', point: point2(2.0001, .0002) }
  const planet: Anchor = { frame: 'planet:fixture', point: translate(shipOrigin, [2, 0, 0]) }
  // Same-frame differences never add the ship origin, even near cell boundaries.
  near(relativeAnchor(player, { ...player, point: point2(2, 0) }, frames), [.0001, .0002])
  // Cross-frame precision is bounded by the existing split offset representation.
  near(relativeAnchor(player, planet, frames), [.0001, .0002], .00003)
  near(relativeM(convertAnchor({ ...player, point: point2(2, 0) }, 'planet:fixture', frames).point, planet.point), [0, 0, 0])
  const shifted = { ...state, frames: { ...state.frames, planet: { ...state.frames.planet, universeOrigin: shipOrigin, angle: 2 } } }
  assert.deepEqual(drawList(shifted), drawList(state))
})
test('fixed stepping is repeatable across render cadence, diagonal normalized and camera follows', () => {
  const a = new TopDownClock(), b = new TopDownClock()
  let first = createTopDownState(), second = createTopDownState()
  for (let i = 0; i < 30; i++) first = a.advance(first, 1 / 60, [1, -1])
  for (let i = 0; i < 15; i++) second = b.advance(second, 1 / 30, [1, -1])
  assert.deepEqual(first, second)
  const player = first.entities.find(e => e.id === PLAYER_ID)!
  const lag = Math.hypot(...drawList(first).find(i => i.entity.id === PLAYER_ID)!.positionM)
  assert.ok(lag > 0 && lag < .22)
  const stopped = new TopDownClock().advance(first, .25, [0, 0])
  assert.ok(Math.hypot(...drawList(stopped).find(i => i.entity.id === PLAYER_ID)!.positionM) < lag)
  const delta = relativeAnchor(player.anchor, createTopDownState().camera.anchor, first.frames)
  near(delta, [1.2 / Math.sqrt(2), -1.2 / Math.sqrt(2)])
  assert.equal(new Set(first.entities.map(e => e.id)).size, first.entities.length)
})
test('visible walls bound the walk fixture and input/time validation prevent invalid transforms', () => {
  let state = createTopDownState()
  for (let i = 0; i < 2000; i++) state = stepTopDown(state, rotate2([1, 0], state.frames.ship.angle))
  const point = state.entities.find(e => e.id === PLAYER_ID)!.anchor.point.offsetM
  assert.ok(point[0] <= 2.04)
  for (const invalid of [NaN, Infinity, -1]) assert.throws(() => new TopDownClock().advance(state, invalid, [0, 0]), RangeError)
  assert.throws(() => stepTopDown(state, [NaN, 0]), RangeError)
  const clock = new TopDownClock()
  assert.equal(clock.advance(createTopDownState(), 100, [0, 0]).tick, 15)
  clock.advance(state, 1 / 120, [0, 0]); clock.reset()
  assert.equal(clock.advance(state, 1 / 120, [0, 0]).tick, state.tick)
})
test('draw order is stable independently of insertion and has no roof cover', () => {
  const state = createTopDownState(), list = drawList(state)
  assert.deepEqual(drawList({ ...state, entities: [...state.entities].reverse() }), list)
  assert.deepEqual([...new Set(list.map(i => i.entity.layer))], LAYERS.filter(layer => layer !== 'cues'))
  assert.ok(!list.some(i => /roof/i.test(i.entity.id)))
  assert.equal(list.find(i => i.entity.appearance === 'floor')!.entity.layer, 'interior')
  assert.equal(list[0].entity.appearance, 'ground')
})

const playerAnchor = (state: ReturnType<typeof createTopDownState>) => state.entities.find(e => e.id === PLAYER_ID)!.anchor
const walk = (state: ReturnType<typeof createTopDownState>, direction: readonly [number, number], ticks: number) => {
  for (let i = 0; i < ticks; i++) state = stepTopDown(state, direction)
  return state
}
test('sweeps block high-speed motion, slide tangent/away, and resolve in an object frame', () => {
  const state = createTopDownState(), frames = { ...state.frames, ship: { ...state.frames.ship, angle: 0 } }
  const object = { id: 'box', anchor: { frame: 'ship:scout' as const, point: point2(0, 0) }, footprint: { halfWidth: .5, halfHeight: 1 } }
  const anchor = { ...object.anchor, point: point2(-3, 0) }
  const fraction = allowedMove(anchor, [10, 0], [object], frames)
  assert.ok(Math.abs(-3 + 10 * fraction + .5 + PLAYER_RADIUS_M) < 1e-7)
  assert.equal(allowedMove({ ...anchor, point: point2(-.5 - PLAYER_RADIUS_M, 0) }, [0, 10], [object], frames), 1)
  assert.equal(allowedMove({ ...anchor, point: point2(-.5 - PLAYER_RADIUS_M, 0) }, [-10, 0], [object], frames), 1)
  const rotated = { ...frames, ship: { ...frames.ship, angle: Math.PI / 2 } }
  assert.ok(Math.abs(allowedMove(anchor, [0, 10], [object], rotated) - fraction) < 1e-9)
  const planetAnchor = convertAnchor(anchor, 'planet:fixture', rotated)
  assert.ok(Math.abs(allowedMove(planetAnchor, [0, 10], [object], rotated) - fraction) < 1e-9)
})
test('crate, furniture and walls block walking; open passages reach cockpit and cargo', () => {
  let state = createTopDownState()
  state = walk(state, rotate2([0, -1], state.frames.ship.angle), 160)
  assert.ok(playerAnchor(state).point.offsetM[1] >= .36 - 1e-8, 'crate stops direct path')
  const angle = state.frames.ship.angle
  state = walk(state, rotate2([1, 0], angle), 33)
  state = walk(state, rotate2([0, -1], angle), 60)
  assert.ok(playerAnchor(state).point.offsetM[1] < -1.5, 'walk around crate')
  state = walk(state, rotate2([-1, 0], angle), 33)
  state = walk(state, rotate2([0, -1], angle), 40)
  assert.ok(playerAnchor(state).point.offsetM[1] < -3, 'pass open cockpit partition')
  const start = createTopDownState()
  const south = rotate2([0, 1], angle)
  const door = walk(start, south, 200)
  assert.ok(playerAnchor(door).point.offsetM[1] > 5.5, 'wide cargo and aft entry are open')
  const west = walk(start, rotate2([-1, 0], angle), 200)
  assert.ok(playerAnchor(west).point.offsetM[0] >= -2.04 - 1e-7)
})
test('nearby selection is deterministic, rejects missing/disabled/distant targets and supports request/cancel', () => {
  let state = createTopDownState()
  assert.equal(nearbyObject(playerAnchor(state), state.entities, state.frames), undefined)
  assert.match(requestInteraction(state).feedback, /No available object/)
  assert.equal(requestInteraction(state, 'missing').player.mode, 'standing')
  state = walk(state, [0, -1], 28)
  state = walk(state, [1, 0], 25)
  state = stepTopDown(state, [0, 0])
  assert.equal(state.selectedId, 'scout:core')
  const core = state.entities.find(e => e.id === 'scout:core')!
  assert.equal(interactionInReach(playerAnchor(state), { ...core, interaction: { ...core.interaction!, enabled: false } }, state.frames), false)
  assert.equal(nearbyObject(playerAnchor(state), [core, { ...core, id: 'a' }], state.frames)?.id, 'a')
  const active = requestInteraction(state)
  assert.equal(active.player.mode, 'interacting'); assert.equal(active.player.targetId, core.id)
  assert.equal(requestInteraction(active).player.targetId, core.id)
  assert.equal(cancelInteraction(active).player.mode, 'standing')
  assert.equal(stepTopDown(active, [-1, 0]).player.targetId, null)
  const removed = stepTopDown({ ...active, entities: active.entities.filter(e => e.id !== core.id) }, [0, 0])
  assert.equal(removed.player.mode, 'standing'); assert.match(removed.feedback, /unavailable/)
})
test('facing is retained at rest and camera follows without a snap on move/release', () => {
  const start = createTopDownState(), moved = stepTopDown(start, [1, 0])
  assert.equal(moved.player.facing, Math.PI / 2)
  assert.ok(Math.hypot(...relativeAnchor(moved.camera.anchor, start.camera.anchor, moved.frames)) < .04)
  const stopped = stepTopDown(moved, [0, 0])
  assert.equal(stopped.player.facing, moved.player.facing); assert.equal(stopped.player.mode, 'standing')
  assert.ok(Math.hypot(...relativeAnchor(playerAnchor(stopped), stopped.camera.anchor, stopped.frames)) > 0)
})


test('Scout fixtures are addressable, room boundaries physical and exterior camera remains stable', () => {
  const state = createTopDownState(), frames = { ...state.frames, ship: { ...state.frames.ship, angle: 0 } }
  for (const id of ['scout:desk', 'scout:chair', 'scout:core', 'scout:fabricator', 'scout:storage']) {
    const object = state.entities.find(e => e.id === id)!
    assert.equal(object.anchor.frame, 'ship:scout'); assert.ok(object.interaction)
    const p = { ...object.anchor, point: translate(object.anchor.point, [id === 'scout:fabricator' ? 1.2 : id === 'scout:storage' ? -1.2 : 0, id === 'scout:fabricator' || id === 'scout:storage' ? 0 : id === 'scout:chair' ? .85 : 1.1, 0]) }
    assert.equal(interactionInReach(p, object, state.frames), true)
    const fixture = { ...state, entities: state.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: p } : e) }
    assert.equal(requestInteraction(fixture, id).player.targetId, id)
  }
  const anchor = (x: number, y: number): Anchor => ({ frame: 'ship:scout', point: point2(x, y) })
  assert.equal(allowedMove(anchor(0, 2), [0, 4.5], state.entities, frames), 1)
  assert.ok(allowedMove(anchor(1.6, 2), [0, 1], state.entities, frames) < 1, 'cargo bulkhead blocks outside its opening')
  assert.ok(allowedMove(anchor(0, 4.8), [4, 0], state.entities, frames) < 1, 'intact hull blocks')
  assert.ok(allowedMove(anchor(-1, -1), [-3, 0], state.entities, frames) < 1, 'bunk blocks')
  let outside = createTopDownState('exterior')
  const originalCamera = outside.camera.anchor
  for (let i = 0; i < 100; i++) outside = stepTopDown(outside, [0, 0])
  assert.deepEqual(outside.camera.anchor, originalCamera)
  assert.deepEqual(outside.entities.filter(e => e.id.startsWith('scout:')), state.entities.filter(e => e.id.startsWith('scout:')))
})
