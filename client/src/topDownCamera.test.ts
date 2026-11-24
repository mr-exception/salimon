import { test } from 'node:test'
import assert from 'node:assert/strict'
import { CameraDrag, CAMERA_MAX_SCALE, CAMERA_MIN_SCALE, CAMERA_PAN_RADIUS_M, followWorldCamera, panWorldCamera, zoomWorldCamera } from './topDownCamera.ts'
import { convertAnchor, point2, relativeAnchor, worldToView } from './topDownFrames.ts'
import { groundFromView, requestGroundMovement } from './topDownNavigation.ts'
import { createTopDownState, PLAYER_ID, rechartWalking, stepTopDown } from './topDownRuntime.ts'
import { localToSurface, surfaceRegion } from './surfaceCoordinates.ts'
const player = (s: ReturnType<typeof createTopDownState>) => s.entities.find(e => e.id === PLAYER_ID)!.anchor
const lag = (s: ReturnType<typeof createTopDownState>) => Math.hypot(...relativeAnchor(player(s), s.camera.anchor, s.frames))

test('zoom clamps and pan is bounded; malformed deltas never corrupt state or gameplay', () => {
  const initial = createTopDownState()
  assert.equal(zoomWorldCamera(initial, NaN), initial)
  assert.equal(panWorldCamera(initial, [Infinity, 0]), initial)
  let s = initial
  for (let i = 0; i < 20; i++) s = zoomWorldCamera(s, -1000)
  assert.equal(s.camera.pixelsPerM, CAMERA_MAX_SCALE)
  for (let i = 0; i < 20; i++) s = zoomWorldCamera(s, 1000)
  assert.equal(s.camera.pixelsPerM, CAMERA_MIN_SCALE)
  s = panWorldCamera(s, [1e6, -1e6])
  assert.ok(Math.abs(lag(s) - CAMERA_PAN_RADIUS_M) < 1e-9)
  assert.equal(s.entities, initial.entities)
  assert.equal(s.scoutLocation, initial.scoutLocation)
  assert.equal(s.cockpit, initial.cockpit)
})

test('projection round trips after actual zoom/pan operations in each frame and detached interior', () => {
  for (const frame of ['ship:scout', 'planet:fixture'] as const) for (const zoom of [-500, 700]) {
    let s = createTopDownState('interior')
    s = { ...s, camera: { ...s.camera, anchor: convertAnchor(s.camera.anchor, frame, s.frames) } }
    s = panWorldCamera(zoomWorldCamera(s, zoom), [121, -83])
    const target = { frame: 'ship:scout' as const, point: point2(1, 3) }
    const view = worldToView(relativeAnchor(target, s.camera.anchor, s.frames), s.frames)
    assert.ok(Math.hypot(...relativeAnchor(groundFromView(s, [view[0] * s.camera.pixelsPerM, view[1] * s.camera.pixelsPerM]), target, s.frames)) < 1e-9)
  }
  let s = createTopDownState('interior')
  s = { ...s, frames: { ...s.frames, ship: { ...s.frames.ship, parent: null } } }
  s = panWorldCamera(s, [80, 0])
  assert.ok(Number.isFinite(lag(s)))
  assert.equal(groundFromView(s, [0, 0]).frame, 'ship:scout')
})

test('pan stays still across idle ticks; Space, WASD and route requests restore the original smooth follow', () => {
  let s = panWorldCamera(createTopDownState('interior'), [160, 0])
  const camera = s.camera, avatar = player(s)
  for (let i = 0; i < 120; i++) s = stepTopDown(s, [0, 0])
  assert.equal(s.camera, camera)
  assert.deepEqual(player(s), avatar)
  const restored = followWorldCamera(s)
  assert.deepEqual(restored.camera.anchor, s.camera.anchor) // command never snaps
  const next = stepTopDown(restored, [0, 0])
  assert.ok(Math.abs(lag(next) / lag(s) - .84) < 1e-9)
  assert.deepEqual(player(next), avatar)
  assert.equal(stepTopDown(s, [1, 0]).camera.panned, false)
  const route = requestGroundMovement(s, { frame: 'ship:scout', point: point2(0, 4) })
  assert.equal(route.groundCommand?.status, 'moving')
  assert.equal(route.camera.panned, false)
  assert.deepEqual(route.camera.anchor, camera.anchor)
})

test('zoom and follow preserve continuous camera coordinates across the Scout doorway', () => {
  let s = createTopDownState('interior')
  const start = { frame: 'ship:scout' as const, point: point2(0, 5.45) }
  s = { ...s, camera: { anchor: start, pixelsPerM: CAMERA_MAX_SCALE }, entities: s.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: start } : e) }
  s = requestGroundMovement(s, { frame: 'ship:scout', point: point2(0, 6.5) })
  let crossed = false
  for (let i = 0; i < 40; i++) {
    const old = s.camera.anchor, frames = s.frames
    s = stepTopDown(s, [0, 0])
    crossed ||= player(s).frame === 'planet:fixture'
    assert.ok(Math.hypot(...relativeAnchor(s.camera.anchor, old, frames)) < .041)
    assert.ok(lag(s) < .22)
    assert.equal(s.camera.pixelsPerM, CAMERA_MAX_SCALE)
  }
  assert.ok(crossed)
})

test('panned camera is reprojected with generated walking charts without losing physical address', () => {
  let s = createTopDownState()
  const landing = surfaceRegion(s.scoutLocation.surface, s.scoutLocation.surface.definition)
  const anchor = { frame: 'planet:fixture' as const, chart: landing, point: point2(280, 0) }
  s = { ...s, entities: s.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e), camera: { ...s.camera, anchor } }
  s = panWorldCamera(s, [50, 30])
  const physical = localToSurface(landing, s.camera.anchor.point.offsetM.slice(0, 2) as [number, number])
  s = rechartWalking(s)
  assert.notEqual(s.frames.surface?.chart.id, landing.id)
  assert.equal(s.camera.panned, true)
  const after = localToSurface(s.camera.anchor.chart!, s.camera.anchor.point.offsetM.slice(0, 2) as [number, number])
  assert.ok(Math.hypot(...physical.radialM.map((n, i) => n - after.radialM[i]!)) < 1e-7)
})

test('pointer release/cancel ignores other pointers and clears coordinates before another drag', () => {
  const drag = new CameraDrag()
  assert.equal(drag.begin(3, [10, 20]), true)
  assert.equal(drag.begin(4, [0, 0]), false)
  assert.equal(drag.move(4, [100, 100]), null)
  assert.deepEqual(drag.move(3, [13, 25]), [3, 5])
  assert.equal(drag.cancel(4), null)
  assert.equal(drag.id, 3)
  assert.equal(drag.cancel(), 3)
  assert.equal(drag.move(3, [50, 50]), null)
  assert.equal(drag.id, null)
  assert.equal(drag.begin(4, [100, 100]), true)
  assert.deepEqual(drag.move(4, [102, 97]), [2, -3])
})

test('subpixel picked routes sweep the planned diagonal past the Core/crate corner', () => {
  for (const x of [-.008, .008]) {
    let s = createTopDownState('interior')
    const anchor = { frame: 'ship:scout' as const, point: point2(x, 4.4) }
    s = { ...s, entities: s.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e) }
    s = requestGroundMovement(s, { ...anchor, point: point2(x, -1.65) })
    for (let i = 0; i < 400 && s.groundCommand?.status === 'moving'; i++) s = stepTopDown(s, [0, 0])
    assert.equal(s.groundCommand?.status, 'arrived')
    assert.ok(Math.hypot(...relativeAnchor(player(s), { ...anchor, point: point2(x, -1.65) }, s.frames)) < 1e-9)
  }
})
