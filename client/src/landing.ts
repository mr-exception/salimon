import { physicalRadiusM, SAFE_LANDING_MPS, surfaceVelocity } from './contact.ts'
import { relativeM, translate } from './coordinates.ts'
import { add, dot, length, normal, rotateVector, scale, sub, type Vec3 } from './math.ts'
import type { Body } from './sampleUniverse.ts'
import { ANGULAR_SPEED, angleDifference, DRY_MASS_KG, STEP_SECONDS, type ShipState } from './simulation.ts'
import type { SurfaceAddress } from './surfaceSite.ts'

export const LANDING_FUNDING = { kind: 'unlimited-prototype', impulsePerJoule: 1 } as const
export const LANDING_MAX_SECONDS = 12 * 3600
export const LANDING_MAX_RELATIVE_MPS = 100
export const LANDING_MAX_ACCELERATION = 10
export interface LandingPlan {
  readonly startTick: number
  readonly durationTicks: number
  readonly anchor: ShipState
  readonly surface: SurfaceAddress
}
/** Rigidly rotating local frame. Catalog centres are fixed in this prototype;
 * a future ephemeris adapter must supply the corresponding moving frame. */
export function rotateLanding(v: Vec3, body: Body, seconds: number): Vec3 {
  return rotateVector(v, body.angularVelocityRadps ?? [0, 0, 0], seconds)
}
function geometry(plan: LandingPlan, body: Body) {
  const start = relativeM(plan.anchor.position, body.position), seconds = plan.durationTicks * STEP_SECONDS
  const velocity = sub(plan.anchor.velocityMps, surfaceVelocity(body, start))
  return { start, seconds, control: add(start, scale(velocity, seconds / 3)), target: rotateLanding(plan.surface.radialM, body, plan.startTick * STEP_SECONDS) }
}
/** Cubic Bezier in the rotating frame, with exact position/velocity endpoints.
 * Its convex hull provides continuous clearance and derivative bounds, rather
 * than relying on samples to discover an unsafe between-tick intersection. */
export function sampleLanding(plan: LandingPlan, body: Body, tick: number): ShipState {
  const { start, control, target, seconds } = geometry(plan, body)
  const u = Math.max(0, Math.min(1, (tick - plan.startTick) / plan.durationTicks)), v = 1 - u
  const radial = add(add(scale(start, v ** 3), scale(control, 3 * v * v * u)), scale(target, 3 * v * u * u + u ** 3))
  const derivative = scale(add(scale(sub(control, start), v * v), scale(sub(target, control), 2 * v * u)), 3 / seconds)
  const rotated = rotateLanding(radial, body, u * seconds)
  const blend = u * u * (3 - 2 * u)
  return { ...plan.anchor, tick, mode: 'landing', throttle: 0, alignment: null, contact: null,
    position: translate(body.position, rotated),
    velocityMps: add(rotateLanding(derivative, body, u * seconds), surfaceVelocity(body, rotated)),
    yaw: plan.anchor.yaw + angleDifference(plan.surface.yaw, plan.anchor.yaw) * blend,
    pitch: plan.anchor.pitch * (1 - blend) }
}
export function planLanding(anchor: ShipState, body: Body, surface: SurfaceAddress, bodies: readonly Body[]): LandingPlan {
  if (surface.bodyId !== body.id || !['planet', 'moon'].includes(body.kind)) throw Error('Choose a supported planet or moon with a matching surface site.')
  if (anchor.mode !== 'flying' || anchor.contact !== null || !anchor.position.offsetM.every(Number.isFinite)
    || !anchor.velocityMps.every(Number.isFinite) || ![anchor.yaw, anchor.pitch, anchor.massKg, body.radiusKm, ...surface.radialM, surface.yaw, ...(body.angularVelocityRadps ?? [])].every(Number.isFinite)
    || body.radiusKm <= 0 || anchor.massKg < DRY_MASS_KG || !Number.isSafeInteger(anchor.tick) || anchor.tick < 0) throw Error('Invalid landing state; recover free flight before requesting landing.')
  const radius = physicalRadiusM(body), start = relativeM(anchor.position, body.position), distance = length(start)
  if ((length(surface.radialM) < radius - 1e-7 || length(surface.radialM) > radius + .001) || Math.abs(surface.yaw) > Math.PI) throw Error('Invalid landing surface coordinates.')
  if (distance < radius - 1e-7 || distance > radius + .15 * body.radiusKm * 1000 + .0001) throw Error('Move within 0.15 body radii above the surface before landing.')
  const relative = sub(anchor.velocityMps, surfaceVelocity(body, scale(normal(start), radius)))
  if (length(relative) > SAFE_LANDING_MPS) throw Error('Reduce full surface-relative speed to 20 m/s or less, including sideways motion.')
  const target = rotateLanding(surface.radialM, body, anchor.tick * STEP_SECONDS)
  const delta = sub(target, start), n = normal(target)
  if (length(sub(start, scale(n, dot(start, n)))) > 1000) throw Error('Choose a landing site within 1 km of the point below the ship.')
  const acceleration = Math.min(LANDING_MAX_ACCELERATION, 50e6 / anchor.massKg)
  const baseSeconds = Math.max(6, 4 * length(delta) / LANDING_MAX_RELATIVE_MPS,
    Math.sqrt(12 * length(delta) / acceleration), 6 * length(relative) / acceleration,
    1.5 * Math.max(Math.abs(angleDifference(surface.yaw, anchor.yaw)), Math.abs(anchor.pitch)) / ANGULAR_SPEED)
  for (const factor of [1, 1.25, 1.5, 2, 3, 4]) {
    const durationTicks = Math.ceil(baseSeconds * factor / STEP_SECONDS), seconds = durationTicks * STEP_SECONDS
    if (seconds > LANDING_MAX_SECONDS || !Number.isSafeInteger(anchor.tick + durationTicks)) break
    const plan: LandingPlan = { startTick: anchor.tick, durationTicks, anchor: structuredClone(anchor), surface: structuredClone(surface) }
    const { control } = geometry(plan, body)
    // All four controls must lie above the target tangent plane. This proves
    // no point of the entire continuous curve enters the physical hull sphere.
    if (Math.min(dot(start, n), dot(control, n)) < radius - 1e-7) continue
    const speedBound = 3 / seconds * Math.max(length(sub(control, start)), length(sub(target, control)))
    const frameAcceleration = 6 / seconds ** 2 * Math.max(length(add(sub(target, scale(control, 2)), start)), length(sub(control, target)))
    const radialBound = Math.max(length(start), length(control), radius), omega = length(body.angularVelocityRadps ?? [0, 0, 0])
    if (speedBound + omega * Math.max(0, radialBound - radius) > LANDING_MAX_RELATIVE_MPS || frameAcceleration + 2 * omega * speedBound + omega ** 2 * radialBound > acceleration) continue
    // The curve stays inside this ball for every body rotation. Conservatively
    // reject nearby interfering bodies without a potentially huge tick sweep.
    if (bodies.some(other => other.id !== body.id && length(relativeM(other.position, body.position)) <= radialBound + physicalRadiusM(other))) continue
    return plan
  }
  throw Error('Unsafe landing trajectory: insufficient braking clearance, thrust, or duration budget. Stop farther above a clear site and retry.')
}
