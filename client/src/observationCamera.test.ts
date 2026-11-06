import { test } from 'node:test'
import assert from 'node:assert/strict'
import { Quaternion, Vector3 } from 'three'
import { CELL_SIZE_M, translate } from './coordinates.ts'
import { forward, initialShip, idleControls, setThrottle, stepShip, type ShipState } from './simulation.ts'
import { ObservationCameraState, observationFrame, observationRelativePosition } from './observationCamera.ts'
const near = (a: number, b: number, epsilon = 1e-10) => assert.ok(Math.abs(a - b) < epsilon, `${a} != ${b}`)
const direction = (view: ObservationCameraState) => new Vector3(0, 0, -1).applyQuaternion(new Quaternion(...view.snapshot().quaternion))
test('ship translation/heading/thrust preserve inertial view; observation never writes simulation', () => {
  let ship: ShipState = { ...initialShip(), position: { cell: [10n ** 30n, -(10n ** 30n), 0n] as const,
    offsetM: [CELL_SIZE_M / 2 - 1, 0, 0] as const } }
  const view = new ObservationCameraState(ship), before = view.snapshot(), start = ship.position
  ship = setThrottle(ship, 100)
  for (let i = 0; i < 120; i++) ship = stepShip(ship, { ...idleControls(), left: true, up: true })
  assert.deepEqual(view.snapshot(), before)
  assert.deepEqual(observationFrame(view, ship).origin, ship.position)
  assert.notDeepEqual(ship.position, start)
  const body = translate(ship.position, [0.125, 0.25, -1])
  assert.deepEqual(observationRelativePosition(body, ship), [0.125, 0.25, -1])
  const saved = structuredClone(ship)
  view.look(2, 3); view.zoom(-10000); view.reset(ship)
  assert.deepEqual(ship, saved)
  direction(view).toArray().forEach((value, index) => near(value, forward(ship.yaw, ship.pitch)[index]))
})
test('full turns cross poles/antiparallel continuously without local roll injection', () => {
  for (const axis of ['horizontal', 'vertical'] as const) {
    const view = new ObservationCameraState(initialShip()), delta = 2 * Math.PI / 720
    let previous = new Quaternion(...view.snapshot().quaternion)
    for (let i = 0; i < 720; i++) {
      const fraction = delta / (75 * Math.PI / 180)
      view.look(axis === 'horizontal' ? fraction : 0, axis === 'vertical' ? fraction : 0)
      const next = new Quaternion(...view.snapshot().quaternion)
      near(previous.angleTo(next), delta, 1e-9); near(next.length(), 1)
      near(previous.clone().invert().multiply(next).z, 0)
      if (i === 179 || i === 539) near(direction(view)[axis === 'vertical' ? 'y' : 'x'], i === 179 ? 1 : -1)
      if (i === 359) near(direction(view).z, 1)
      previous = next
    }
    near(previous.angleTo(new Quaternion()), 0, 1e-7)
  }
})
test('zoom is multiplicative, bounded, finite, reversible and scales telescope look', () => {
  const view = new ObservationCameraState(initialShip())
  view.zoom(-100); near(view.snapshot().fov, 75 * Math.exp(-0.1))
  view.zoom(100); near(view.snapshot().fov, 75)
  view.zoom(-1e10); near(view.snapshot().fov, 0.1)
  view.look(1, 0); near(direction(view).angleTo(new Vector3(0, 0, -1)), 0.1 * Math.PI / 180)
  const snapshot = view.snapshot()
  view.zoom(NaN); view.zoom(Infinity); view.look(Infinity, 1); assert.deepEqual(view.snapshot(), snapshot)
  view.zoom(1e10); near(view.snapshot().fov, 75)
})
test('snapshot restore validates atomically; reset samples heading once', () => {
  const view = new ObservationCameraState(initialShip())
  view.look(0.3, 2.4); view.zoom(-1000)
  const snapshot = view.snapshot(), restored = new ObservationCameraState(initialShip())
  restored.restore(snapshot)
  snapshot.quaternion.forEach((value, i) => near(restored.snapshot().quaternion[i], value))
  const before = restored.snapshot()
  for (const quaternion of [[0, 0, 0, 0], [NaN, 0, 0, 1]]) {
    assert.throws(() => restored.restore({ quaternion: quaternion as [number, number, number, number], fov: 10 }))
    assert.deepEqual(restored.snapshot(), before)
  }
  assert.throws(() => restored.restore({ ...before, fov: NaN }))
  restored.restore({ ...before, fov: -100 }); near(restored.snapshot().fov, 0.1)
  restored.restore({ ...before, fov: 100 }); near(restored.snapshot().fov, 75)
  for (const pitch of [-Math.PI / 2, Math.PI / 2, 0]) {
    restored.reset({ yaw: Math.PI, pitch })
    direction(restored).toArray().forEach((value, i) => near(value, forward(Math.PI, pitch)[i]))
    near(restored.snapshot().fov, 75)
  }
})
