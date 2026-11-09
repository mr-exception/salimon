import { physicalRadiusM, resolveContacts } from './contact.ts'
import { relativeM, translate } from './coordinates.ts'
import { add, length, normal, scale } from './math.ts'
import type { Body } from './sampleUniverse.ts'
import { DRY_MASS_KG, STEP_SECONDS, type ShipState } from './simulation.ts'

export interface TakeoffPlan { readonly startTick: number; readonly massKg: number }
export const TAKEOFF_TICKS = 360
export const TAKEOFF_HEIGHT_M = 50
export const TAKEOFF_SECONDS = TAKEOFF_TICKS * STEP_SECONDS
// Future Core adapter may fund this demand before admission. The prototype
// supplies unlimited energy: no reservation, spending or fuel mutation.
export const TAKEOFF_FUNDING = { kind: 'unlimited-prototype', impulsePerJoule: 1 } as const
export const TAKEOFF_PEAK_ACCELERATION = 4 * TAKEOFF_HEIGHT_M / TAKEOFF_SECONDS ** 2

/** Captured launch-frame radial lift plus inherited inertial surface velocity.
 * Acceleration is smooth, nonnegative, zero at both ends; no terminal braking
 * discontinuity. The conservative 50 MN bidirectional budget uses the approved
 * thruster law, without changing selected throttle or dry mass.
 */
export function sampleTakeoff(anchor: ShipState, body: Body, plan: TakeoffPlan, tick: number): ShipState {
  const u = Math.max(0, Math.min(1, (tick - plan.startTick) / TAKEOFF_TICKS))
  const n = normal(relativeM(anchor.position, body.position))
  const height = TAKEOFF_HEIGHT_M * (u*u + (Math.cos(2*Math.PI*u)-1)/(2*Math.PI**2))
  const speed = TAKEOFF_HEIGHT_M / TAKEOFF_SECONDS * (2*u - Math.sin(2*Math.PI*u)/Math.PI)
  return { ...anchor, massKg: plan.massKg, tick, mode: u === 1 ? 'flying' : 'taking-off', contact: null, throttle: 0, alignment: null,
    position: translate(anchor.position, add(scale(anchor.velocityMps, u*TAKEOFF_SECONDS), scale(n,height))),
    velocityMps: add(anchor.velocityMps, scale(n,speed)) }
}
export function planTakeoff(anchor: ShipState, body: Body, bodies: readonly Body[]): TakeoffPlan {
  if (!anchor.position.offsetM.every(Number.isFinite) || !anchor.velocityMps.every(Number.isFinite) || ![anchor.yaw,anchor.pitch,body.radiusKm].every(Number.isFinite) || body.radiusKm <= 0 || anchor.massKg < DRY_MASS_KG || anchor.tick < 0) throw Error('Invalid takeoff inputs.')
  if (!Number.isSafeInteger(anchor.tick + TAKEOFF_TICKS) || !Number.isFinite(anchor.massKg)
    || anchor.massKg * TAKEOFF_PEAK_ACCELERATION > 50e6) throw Error('Unsafe takeoff: insufficient thrust for loaded mass.')
  if (!anchor.velocityMps.every(Number.isFinite) || length(anchor.velocityMps) + 2*TAKEOFF_HEIGHT_M/TAKEOFF_SECONDS > 100) throw Error('Unsafe takeoff: inherited speed exceeds launch limit.')
  const plan = { startTick: anchor.tick, massKg: anchor.massKg }
  let before: ShipState = { ...anchor, mode: 'taking-off', contact: null }
  for (let i=1;i<=TAKEOFF_TICKS;i++) {
    const next = sampleTakeoff(anchor,body,plan,anchor.tick+i)
    if (length(relativeM(next.position,body.position)) < physicalRadiusM(body)-.0001
      || resolveContacts(before,next,bodies).contact !== null) throw Error('Unsafe takeoff: trajectory intersects a body.')
    before = next
  }
  return plan
}
