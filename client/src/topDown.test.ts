import { test } from 'node:test'
import assert from 'node:assert/strict'
import { CELL_SIZE_M, ORIGIN, relativeM, translate } from './coordinates.ts'
import { convertAnchor, inPlanet, point2, relativeAnchor, type Anchor } from './topDownFrames.ts'
import { createTopDownState, drawList, LAYERS, PLAYER_ID, stepTopDown, TopDownClock } from './topDownRuntime.ts'

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
  assert.deepEqual(first.camera.anchor, player.anchor)
  near(drawList(first).find(i => i.entity.id === PLAYER_ID)!.positionM, [0, 0])
  const delta = relativeAnchor(player.anchor, createTopDownState().camera.anchor, first.frames)
  near(delta, [1.2 / Math.sqrt(2), -1.2 / Math.sqrt(2)])
  assert.equal(new Set(first.entities.map(e => e.id)).size, first.entities.length)
})
test('walk fixture bounds and input/time validation prevent invalid transforms', () => {
  let state = createTopDownState()
  for (let i = 0; i < 2000; i++) state = stepTopDown(state, [1, 1])
  const point = state.camera.anchor.point.offsetM
  assert.ok(point[0] <= 1.1 && point[1] <= 4.4)
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
  assert.deepEqual([...new Set(list.map(i => i.entity.layer))], LAYERS)
  assert.ok(!list.some(i => /roof/i.test(i.entity.id)))
  assert.equal(list.find(i => i.entity.appearance === 'floor')!.entity.layer, 'interior')
  assert.equal(list[0].entity.appearance, 'ground')
})
