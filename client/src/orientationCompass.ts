import { Quaternion, Vector3 } from 'three'
import type { ObservationSnapshot } from './observationCamera.ts'
import type { Vec3 } from './math.ts'

/** Azimuthal equidistant hemisphere disk: center is the view axis, rim is
 * sideways. Rear directions fold onto the disk, retaining their hemisphere.
 * Exact rear has a deterministic center glyph, distinguished by its hollow ring.
 */
export function compassDirection(direction: Vec3, view: ObservationSnapshot) {
  const vector = new Vector3(...direction)
  if (!direction.every(Number.isFinite) || vector.length() <= 0.01) return null
  vector.normalize().applyQuaternion(new Quaternion(...view.quaternion).invert())
  const transverse = Math.hypot(vector.x, vector.y)
  const radius = Math.atan2(transverse, Math.abs(vector.z)) / (Math.PI / 2) * 42
  return { x: transverse < 1e-12 ? 0 : radius * vector.x / transverse,
    y: transverse < 1e-12 ? 0 : -radius * vector.y / transverse,
    rear: vector.z > 0, depth: -vector.z }
}
