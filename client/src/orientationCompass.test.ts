import { test } from 'node:test'
import assert from 'node:assert/strict'
import { compassDirection } from './orientationCompass.ts'
import { ObservationCameraState } from './observationCamera.ts'
import { forward } from './simulation.ts'
const view = new ObservationCameraState({ yaw: 0, pitch: 0 })
test('compass maps full sphere, poles and exact rear deterministically', () => {
 assert.deepEqual(compassDirection([0, 0, -1], view.snapshot()), { x: 0, y: 0, rear: false, depth: 1 })
 assert.deepEqual(compassDirection([0, 0, 1], view.snapshot()), { x: 0, y: 0, rear: true, depth: -1 })
 for (let yaw = -180; yaw <= 180; yaw += 15) for (let pitch = -90; pitch <= 90; pitch += 15) {
  const direction = forward(yaw * Math.PI / 180, pitch * Math.PI / 180)
  const marker = compassDirection(direction, view.snapshot())!
  assert.ok(Number.isFinite(marker.x) && Number.isFinite(marker.y))
  assert.ok(Math.hypot(marker.x, marker.y) <= 42.000000001)
  assert.equal(marker.rear, direction[2] > 0)
 }
 assert.equal(compassDirection([1, 0, 0], view.snapshot())!.x, 42)
 assert.equal(compassDirection([0, 1, 0], view.snapshot())!.y, -42)
 assert.equal(compassDirection([0, -1, 0], view.snapshot())!.y, 42)
})
test('camera-relative mapping preserves coincidence, suppresses zero speed, heading rotates independently', () => {
 assert.equal(compassDirection([0, 0, 0], view.snapshot()), null)
 assert.equal(compassDirection([0.001, 0, 0], view.snapshot()), null)
 const before = view.snapshot(), heading = compassDirection(forward(0, 0), before)
 assert.deepEqual(heading, compassDirection([0, 0, -100], before))
 assert.notDeepEqual(heading, compassDirection(forward(Math.PI / 2, 0), before))
 assert.deepEqual(view.snapshot(), before)
 const rotated = new ObservationCameraState({ yaw: Math.PI / 2, pitch: 0 })
 assert.ok(Math.abs(compassDirection(forward(Math.PI / 2, 0), rotated.snapshot())!.x) < 1e-10)
})
