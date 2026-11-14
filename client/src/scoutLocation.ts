import { planTakeoff, type TakeoffPlan } from './takeoff.ts'
import { surfaceVelocity } from './contact.ts'
import { ORIGIN, relativeM } from './coordinates.ts'
import { normal } from './math.ts'
import { resolveSurface, startingSurface, surfacePosition, validateSurfaceClearance, type SurfaceAddress } from './surfaceSite.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { DRY_MASS_KG, initialShip, type ShipState } from './simulation.ts'
import type { WorldFrames } from './topDownFrames.ts'

// The authored site's tangent origin is the +Y surface of sample Vesper.
// Its ship origin (local metres) selects a radial contact on that sphere.
// This is a deterministic illustrative binding, not canonical planet generation.
// Both presentations consume this domain value; only explicit domain actions
// may replace the surface binding.
export function startingScoutLocation(): ScoutLocation { return surfaceScoutLocation(startingSurface()) }
export function surfaceScoutLocation(address: SurfaceAddress): ScoutLocation {
  const site = resolveSurface(address)
  validateSurfaceClearance(site)
  const ship: ShipState = {
    ...initialShip(), position: surfacePosition(site), yaw: address.yaw, mode: 'grounded',
    velocityMps: surfaceVelocity(site.body, address.radialM).map(v => v === 0 ? 0 : v) as [number, number, number],
    contact: { bodyId: site.body.id, outcome: 'safe-contact', impactSpeedMps: 0, normal: normal(address.radialM), ageTicks: 60 },
  }
  return { siteId: site.id, surface: site.address, body: site.body, frames: site.frames, ship, binding: site.id, maneuver: null, scanner: null, boardingDoor: 'open', launch: null }
}
// One live object is shared by world and navigation. Save records clone it.
export interface ScoutLocation {
  surface: SurfaceAddress
  launch: TakeoffPlan | null
  boardingDoor: 'open' | 'closed'
  siteId: string
  body: typeof SAMPLE_BODIES[number]
  frames: WorldFrames
  ship: ShipState
  binding: string | null
  maneuver: 'takeoff' | 'landing' | null
  scanner: import('./scanner.ts').ScannerSnapshot | null
}
export type ScoutAction = { kind: 'begin-takeoff' } | { kind: 'finish-takeoff' } | { kind: 'begin-landing'; siteId?: string; bodyId: string; surface?: SurfaceAddress } | { kind: 'finish-landing' }
export function transitionScout(location: ScoutLocation, action: ScoutAction): ScoutLocation {
  const ship = location.ship
  const detached = { ...location.frames, ship: { ...location.frames.ship, parent: null, origin: ORIGIN } }
  if (action.kind === 'begin-takeoff') {
    const denial = takeoffDenial(location)
    if (denial) throw new Error(denial)
    return { ...location, frames: detached, binding: null, launch: planTakeoff(ship, location.body, SAMPLE_BODIES), maneuver: 'takeoff', ship: { ...ship, mode: 'taking-off', contact: null, throttle: 0, alignment: null } }
  }
  if (action.kind === 'finish-takeoff') throw new Error('Takeoff completes automatically; pilot override is locked.')
  if (action.kind === 'begin-landing' && (ship.mode === 'flying' || ship.mode === 'recovering' && ship.contact?.outcome === 'safe-contact')) {
    if (!action.surface && action.siteId !== 'site:illustrated-landing:v1') throw Error('Incomplete site mapping: supply body-fixed coordinates and orientation.')
    const target = surfaceScoutLocation(action.surface ?? startingSurface())
    if (action.bodyId !== target.body.id || action.siteId !== undefined && action.siteId !== target.siteId) throw Error('Planet/body mismatch or incomplete site mapping.')
    if (location.boardingDoor !== 'closed' || location.maneuver !== null || location.launch !== null) throw Error('Seal the door and finish the active maneuver before landing.')
    return { ...location, surface: target.surface, siteId: target.siteId, body: target.body,
      frames: { ...target.frames, ship: { ...target.frames.ship, parent: null, origin: ORIGIN } },
      maneuver: 'landing', ship: { ...ship, mode: 'landing', contact: null, throttle: 0, alignment: null } }
  }
  if (action.kind === 'finish-landing' && ship.mode === 'landing' && location.maneuver === 'landing') {
    const landed = surfaceScoutLocation(location.surface)
    if (!ship.position.offsetM.every(Number.isFinite) || !ship.velocityMps.every(Number.isFinite) || Math.hypot(...relativeM(ship.position, landed.ship.position)) > 1e-6 || ship.yaw !== landed.ship.yaw || ship.pitch !== 0
      || Math.hypot(...ship.velocityMps.map((v, i) => v - landed.ship.velocityMps[i]!)) > .001 || ship.contact !== null) throw new Error('Ship must reach the validated surface anchor and stop before landing completes')
    return { ...landed, boardingDoor: location.boardingDoor, scanner: location.scanner, ship: { ...landed.ship, position: ship.position, tick: ship.tick, massKg: ship.massKg } }
  }
  throw new Error('Invalid Scout transition or unsupported surface site')
}

/** Physical admission is shared by domain commands and the session authority. */
export function takeoffDenial(location: ScoutLocation): string | null {
  if (!Number.isFinite(location.ship.massKg) || location.ship.massKg < DRY_MASS_KG || !Number.isSafeInteger(location.ship.tick) || location.ship.tick < 0) return 'Scout state is invalid.'
  if (location.ship.mode === 'destroyed') return 'Ship is destroyed.'
  if (location.maneuver !== null || location.launch !== null) return 'A Scout maneuver is already running.'
  if (location.ship.mode !== 'grounded') return 'Scout must be grounded before takeoff.'
  let site: ScoutLocation
  try { site = surfaceScoutLocation(location.surface) } catch { return 'Scout is not at a validated supported surface site.' }
  if (location.body !== site.body || !location.ship.position.offsetM.every(Number.isFinite) || !location.ship.velocityMps.every(Number.isFinite) || ![location.ship.yaw, location.ship.pitch].every(Number.isFinite)) return 'Scout is not at a validated supported surface site.'
  if (location.frames.ship.id !== site.frames.ship.id || location.frames.planet.id !== site.frames.planet.id
    || location.binding !== site.binding || location.siteId !== site.siteId || location.body.id !== site.body.id
    || location.frames.ship.parent !== site.frames.ship.parent || location.frames.ship.angle !== site.frames.ship.angle
    || !(Math.hypot(...relativeM(location.frames.ship.origin, site.frames.ship.origin)) <= 1e-6)
    || !(Math.hypot(...relativeM(location.ship.position, site.ship.position)) <= 1e-6)
    || location.ship.yaw !== site.ship.yaw || location.ship.pitch !== 0
    || JSON.stringify(location.frames.planet.basis) !== JSON.stringify(site.frames.planet.basis) || location.frames.planet.angle !== site.frames.planet.angle
    || !(Math.hypot(...relativeM(location.frames.planet.universeOrigin, site.frames.planet.universeOrigin)) <= 1e-6)
    || JSON.stringify(location.ship.contact) !== JSON.stringify(site.ship.contact)
    || location.ship.contact?.bodyId !== site.body.id || location.ship.contact.outcome !== 'safe-contact'
    || location.ship.velocityMps.some((v, i) => v !== site.ship.velocityMps[i])) return 'Scout is not at a validated supported surface site.'
  if (location.ship.alignment !== null || location.ship.throttle !== 0) return 'Stop the active ship operation before takeoff.'
  if (location.boardingDoor !== 'closed') return 'Close the aft boarding door before takeoff.'
  return null
}
