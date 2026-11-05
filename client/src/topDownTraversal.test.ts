import { test } from 'node:test'
import assert from 'node:assert/strict'
import { relativeM } from './coordinates.ts'
import { convertAnchor, inPlanet, point2, relativeAnchor, rotate2, type Anchor } from './topDownFrames.ts'
import { capturePlayerLocation, createTopDownState, PLAYER_ID, stepTopDown, type TopDownState } from './topDownRuntime.ts'
import { crossScoutEntry } from './topDownTraversal.ts'

const player = (s: TopDownState) => s.entities.find(e => e.id === PLAYER_ID)!.anchor
const local = (s: TopDownState) => convertAnchor(player(s), s.frames.ship.id, s.frames).point.offsetM
const near = (a: readonly number[], b: readonly number[], tolerance = 1e-8) => a.forEach((v, i) => assert.ok(Math.abs(v - b[i]) < tolerance, `${a} != ${b}`))
function at(s: TopDownState, x: number, y: number, frame: Anchor['frame']): TopDownState {
  const anchor = convertAnchor({ frame: s.frames.ship.id, point: point2(x, y) }, frame, s.frames)
  return { ...s, entities: s.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e), camera: { ...s.camera, anchor } }
}
function walk(s: TopDownState, x: number, y: number, ticks: number) {
  for (let i = 0; i < ticks; i++) s = stepTopDown(s, rotate2([x, y], s.frames.ship.angle))
  return s
}
test('TR-1 entry and exit preserve world point, camera lag, identity, zoom and scene contents at rotated poses', () => {
  for (const angle of [0, -.12, Math.PI / 2, 2.8]) {
    let s = createTopDownState()
    s = { ...s, frames: { ...s.frames, ship: { ...s.frames.ship, origin: point2(200, -80), angle } } }
    s = at(s, 0, 5.52, s.frames.planet.id)
    const objects = s.entities.filter(e => e.id !== PLAYER_ID)
    for (const dy of [-1, 1]) {
      const before = s, direction = rotate2([0, dy], angle)
      s = stepTopDown(s, direction)
      assert.equal(player(s).frame, dy < 0 ? s.frames.ship.id : s.frames.planet.id)
      near(relativeAnchor(player(s), player(before), s.frames), direction.map(v => v * .04))
      near(relativeAnchor(s.camera.anchor, before.camera.anchor, s.frames), relativeAnchor(player(s), before.camera.anchor, s.frames).map(v => v * .16))
      assert.equal(s.camera.anchor.frame, player(s).frame)
      assert.equal(s.camera.pixelsPerM, before.camera.pixelsPerM)
      assert.deepEqual(s.entities.filter(e => e.id !== PLAYER_ID), objects)
    }
  }
})
test('TR-2 only a crossing through the clear aperture changes frame; standing and tangent motion do not', () => {
  const frames = createTopDownState().frames
  const anchor = (x: number, y: number, frame: Anchor['frame']) => convertAnchor({ frame: frames.ship.id, point: point2(x, y) }, frame, frames)
  for (const frame of [frames.ship.id, frames.planet.id]) {
    for (const [a, b] of [[[2, 5.6], [2, 5.4]], [[-2, 0], [0, 0]], [[0, 5.5], [.2, 5.5]], [[0, 5.5], [0, 5.5]]] as const) {
      const after = anchor(b[0], b[1], frame)
      assert.equal(crossScoutEntry(anchor(a[0], a[1], frame), after, frames), after)
    }
  }
  for (const frame of [frames.ship.id, frames.planet.id]) {
    const after = anchor(0, frame === frames.ship.id ? 5.54 : 5.46, frame)
    assert.notEqual(crossScoutEntry(anchor(0, 5.5, frame), after, frames).frame, frame)
  }
})
test('TR-3 actual swept movement blocks aft jambs, side hull and nose in both frames', () => {
  const start = createTopDownState()
  for (const [x, y, dx, dy] of [[2, 7, 0, -1], [-4, 0, 1, 0], [4, 0, -1, 0], [0, -8, 0, 1]]) {
    const s = walk(at(start, x, y, start.frames.planet.id), dx, dy, 200)
    assert.equal(player(s).frame, start.frames.planet.id)
    assert.ok(Math.hypot(local(s)[0] - x, local(s)[1] - y) < 2, 'intact hull stops entry')
  }
  const inside = walk(at(start, 0, 4.8, start.frames.ship.id), 1, 0, 200)
  assert.equal(player(inside).frame, start.frames.ship.id)
  assert.ok(local(inside)[0] < 1, 'storage blocks lateral movement')
})
test('TR-4 location hooks retain exact frame/cells/facing; relocation carries passengers but leaves surface anchors fixed', () => {
  const original = createTopDownState()
  const outside = capturePlayerLocation(original)
  const boarded = walk(at(original, 0, 6, original.frames.planet.id), 0, -1, 30)
  const saved = capturePlayerLocation(boarded)
  assert.equal(saved.anchor.frame, original.frames.ship.id)
  assert.notEqual(saved.anchor.point, player(boarded).point)
  assert.deepEqual(saved.anchor, player(boarded))
  const frames = { ...boarded.frames, ship: { ...boarded.frames.ship, origin: point2(70, 90), angle: Math.PI / 2 } }
  const moved = { ...boarded, frames }
  near(local(moved), local(boarded))
  assert.notDeepEqual(inPlanet(saved.anchor, frames), inPlanet(saved.anchor, boarded.frames))
  assert.deepEqual(inPlanet(outside.anchor, frames), inPlanet(outside.anchor, boarded.frames))
  const exited = walk(moved, 0, 1, 40)
  assert.equal(player(exited).frame, frames.planet.id)
  const exitLocation = capturePlayerLocation(exited)
  const movedAgain = { ...frames, ship: { ...frames.ship, origin: point2(-100, -200) } }
  assert.deepEqual(inPlanet(exitLocation.anchor, movedAgain), inPlanet(exitLocation.anchor, frames))
  const huge = { ...original, entities: original.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { ...e.anchor, point: { cell: [10n ** 24n, -(10n ** 25n), 0n] as const, offsetM: [.0001, -.0002, 0] as const } } } : e) }
  assert.deepEqual(capturePlayerLocation(huge).anchor, player(huge))
  near(relativeM(saved.anchor.point, player(boarded).point), [0, 0, 0])
})
