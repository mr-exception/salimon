import { relativeM, type Position } from './coordinates.ts'
import { dot, length, normal, scale, sub, type Vec3 } from './math.ts'
import type { Body } from './sampleUniverse.ts'

export const MAX_OPTICAL_BODIES = 32
export interface OpticalSphere {
  readonly direction: Vec3
  readonly distanceM: number
  readonly radiusRatio: number
  /** Factored on CPU to retain metres of clearance above million-metre globes. */
  readonly surfaceTerm: number
  readonly angularRadius: number
  readonly lightDirection: Vec3
  readonly color: Vec3
  readonly emissive: boolean
}
const colors: Record<Body['kind'], Vec3> = {
  star: [1, 0.88, 0.68], planet: [0.24, 0.36, 0.44], moon: [0.48, 0.46, 0.42], asteroid: [0.35, 0.31, 0.28],
}
/** Catalog geometry only: no discovery/contact/selection data crosses this API.
 * GPU receives bounded directions and ratios, never absolute split positions.
 * Inside a solid sphere the sky is fully occluded; no invented underground sky.
 */
export function opticalSpheres(bodies: readonly Body[], origin: Position): OpticalSphere[] {
  if (bodies.length > MAX_OPTICAL_BODIES) throw new RangeError('Optical catalog budget exceeded')
  const star = bodies.find(body => body.kind === 'star')
  return bodies.map(body => {
    const vector = relativeM(body.position, origin), distanceM = length(vector)
    const radius = body.radiusKm * 1000
    if (!Number.isFinite(distanceM) || !Number.isFinite(radius) || radius <= 0) throw new RangeError('Invalid optical sphere')
    const distance = Math.max(distanceM, radius * 1e-12)
    if (distance <= 0) throw new RangeError('Unrepresentable optical sphere')
    return {
      direction: normal(vector), distanceM: distance, radiusRatio: radius / distance,
      surfaceTerm: ((distanceM - radius) / distance) * ((distanceM + radius) / distance),
      angularRadius: Math.asin(Math.min(1, radius / distance)),
      lightDirection: star && star !== body ? normal(relativeM(star.position, body.position)) : [0, 1, 0],
      color: colors[body.kind], emissive: body.kind === 'star',
    }
  })
}
/** Stable near-root calculation shared with the optical shader. Physical metres. */
export function opticalRayHit(sphere: OpticalSphere, ray: Vec3): number | null {
  if (sphere.surfaceTerm < 0) return 0
  const along = dot(normal(ray), sphere.direction)
  if (along <= 0) return null
  // Cross-product form avoids cancellation for tiny telescope disks.
  const perpendicular = sub(normal(ray), scale(sphere.direction, along))
  const discriminant = sphere.radiusRatio ** 2 - dot(perpendicular, perpendicular)
  if (discriminant < 0) return null
  return sphere.surfaceTerm / (along + Math.sqrt(discriminant)) * sphere.distanceM
}
