import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createTopDownState, stepTopDown, TopDownClock, PLAYER_ID, requestInteraction, cancelInteraction } from './topDownRuntime.ts'
import { groundFromView, interactiveAt, planGroundRoute, requestGroundMovement, movementFraction } from './topDownNavigation.ts'
import { convertAnchor, point2, relativeAnchor, worldToView, type Anchor } from './topDownFrames.ts'
import { synchronizeDoor } from './scoutDoor.ts'
import { ORIGIN, relativeM } from './coordinates.ts'

const goal = (x: number, y: number): Anchor => ({ frame: 'ship:scout', point: point2(x, y) })
const player = (s: ReturnType<typeof createTopDownState>) => s.entities.find(e => e.id === PLAYER_ID)!.anchor
const place = (s: ReturnType<typeof createTopDownState>, anchor: Anchor): ReturnType<typeof createTopDownState> => ({ ...s, entities: s.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e) })
function finish(s: ReturnType<typeof createTopDownState>, max = 3000) {
  for (let i = 0; i < max && s.groundCommand?.status === 'moving'; i++) {
    const before = player(s), next = stepTopDown(s, [0, 0])
    assert.ok(movementFraction(s, before, relativeAnchor(player(next), before, s.frames)) >= 1 - 1e-7)
    s = next
  }
  assert.equal(s.groundCommand?.status, 'arrived', s.feedback)
  return s
}
test('routes around crate and hull through sole doorway, preserving frames and exact arrival', () => {
  let s = place(createTopDownState('interior'), goal(0, 1))
  const route = planGroundRoute(s, goal(0, -1.8))!
  assert.ok(route.length > 2)
  assert.deepEqual(route, planGroundRoute(s, goal(0, -1.8)))
  s = finish(requestGroundMovement(s, goal(0, -1.8)))
  assert.ok(Math.hypot(...relativeAnchor(player(s), goal(0, -1.8), s.frames)) < 1e-7)
  s = place(createTopDownState(), convertAnchor(goal(-4, 0), 'planet:fixture', s.frames))
  const body = s.scoutLocation.body.id, site = s.scoutLocation.siteId
  s = finish(requestGroundMovement(s, goal(0, 4.5)))
  assert.equal(player(s).frame, 'ship:scout')
  s = finish(requestGroundMovement(s, convertAnchor(goal(0, 7), 'planet:fixture', s.frames)))
  assert.equal(player(s).frame, 'planet:fixture')
  assert.equal(s.scoutLocation.body.id, body); assert.equal(s.scoutLocation.siteId, site)
})
test('blocked endpoints, bounded distance, sealed door and changed obstacles fail safely', () => {
  let s = place(createTopDownState('interior'), goal(0, 4))
  assert.equal(planGroundRoute(s, goal(0, -.6)), null)
  assert.equal(planGroundRoute(s, goal(500, 0)), null)
  s = synchronizeDoor({ ...s, scoutLocation: { ...s.scoutLocation, boardingDoor: 'closed' } })
  assert.equal(planGroundRoute(s, goal(0, 7)), null)
  s = requestGroundMovement(place(createTopDownState('interior'), goal(0, 4)), goal(0, 7))
  s = synchronizeDoor({ ...s, scoutLocation: { ...s.scoutLocation, boardingDoor: 'closed' } })
  for (let i = 0; i < 100; i++) s = stepTopDown(s, [0, 0])
  assert.equal(s.groundCommand?.status, 'failed')
  assert.ok(relativeM(player(s).point, ORIGIN)[1] < 5)
  s = requestGroundMovement(place(createTopDownState('interior'), goal(0, 4)), goal(0, 2))
  s = { ...s, entities: [...s.entities, { id: 'dynamic', anchor: goal(0, 3.5), layer: 'props', appearance: 'crate', footprint: { halfWidth: .5, halfHeight: .1 } }] }
  for (let i = 0; i < 30; i++) s = stepTopDown(s, [0, 0]); assert.equal(s.groundCommand?.status, 'failed')
})
test('manual input, replacement, interaction and cancellation; fixed clock cadence is deterministic', () => {
  const initial = requestGroundMovement(place(createTopDownState('interior'), goal(0, 4)), goal(0, 2))
  assert.equal(stepTopDown(initial, [1, 0]).groundCommand?.status, 'cancelled')
  assert.equal(cancelInteraction(initial).groundCommand?.status, 'cancelled')
  assert.equal(requestInteraction(initial).groundCommand?.status, 'cancelled')
  const replacement = requestGroundMovement(initial, goal(0, 7))
  assert.deepEqual(replacement.groundCommand?.destination, goal(0, 7))
  const a = new TopDownClock(), b = new TopDownClock()
  let sa = initial, sb = initial
  for (let i = 0; i < 120; i++) sa = a.advance(sa, 1 / 60, [0, 0])
  for (let i = 0; i < 30; i++) sb = b.advance(sb, 1 / 15, [0, 0])
  assert.deepEqual(sa, sb)
})
test('live camera projection inverts zoom, translated camera and rotated ship in both frames', () => {
  for (const frame of ['ship:scout', 'planet:fixture'] as const) for (const pixelsPerM of [12, 26, 64]) {
    let s = createTopDownState()
    s = { ...s, camera: { anchor: { frame, point: point2(7, -3) }, pixelsPerM } }
    const target = goal(1, 3), view = worldToView(relativeAnchor(target, s.camera.anchor, s.frames), s.frames)
    const result = groundFromView(s, [view[0] * pixelsPerM, view[1] * pixelsPerM])
    assert.ok(Math.hypot(...relativeAnchor(result, target, s.frames)) < 1e-9)
  }
})
test('interactive footprint reserves right-clicks and detached flight rejects exterior', () => {
  let s = createTopDownState('interior')
  assert.equal(interactiveAt(s, goal(1.85, 4.25)), 'scout:storage')
  assert.equal(interactiveAt(s, goal(0, 5.5)), 'scout:entry')
  assert.equal(interactiveAt(s, goal(0, 4)), null)
  s = { ...s, frames: { ...s.frames, ship: { ...s.frames.ship, parent: null } } }
  assert.equal(planGroundRoute(s, { frame: 'planet:fixture', point: point2(0, 0) }), null)
  assert.equal(planGroundRoute(s, goal(0, 8)), null)
})

test('generated steep terrain is rejected by the planner using the walking probe', async () => {
  const { geologicalDefinition } = await import('./planetDefinition.ts')
  const { localToSurface, surfaceRegion } = await import('./surfaceCoordinates.ts')
  const { normal, scale } = await import('./math.ts')
  const { sampleGeology, WALK_MAX_SLOPE_RADIANS } = await import('./planetGeology.ts')
  const { resolveSurface } = await import('./surfaceSite.ts')
  const publication = geologicalDefinition('caligo')
  const { bodyId, version, digest } = publication, definition = { bodyId, version, digest }
  const centre = { bodyId: 'caligo', radialM: scale(normal([1, 1, 1]), publication.radiusM + 5) }
  const region = surfaceRegion(centre, definition)
  const points = Array.from({ length: 241 }, (_, i) => i - 120)
  const blocked = points.find(x => sampleGeology(localToSurface(region, [x, 0]), definition).slopeRadians > WALK_MAX_SLOPE_RADIANS + .02)!
  assert.ok(Number.isFinite(blocked))
  let s = createTopDownState()
  const surface = { ...centre, yaw: 0, definition }
  s = { ...s, scoutLocation: { ...s.scoutLocation, surface, binding: region.id }, frames: resolveSurface(surface).frames,
    entities: s.entities.filter(e => e.id === PLAYER_ID).map(e => ({ ...e, anchor: { frame: 'planet:fixture', point: point2(blocked - 2, 0) } })) }
  const command = requestGroundMovement(s, { frame: 'planet:fixture', point: point2(blocked, 0) })
  assert.equal(command.groundCommand?.status, 'failed')
})
