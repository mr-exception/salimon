import { physicalRadiusM, surfaceVelocity } from './contact.ts'
import { ORIGIN, relativeM, translate } from './coordinates.ts'
import { normal } from './math.ts'
import { STARTING_SITE } from './planetarySite.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { DRY_MASS_KG, initialShip, type ShipState } from './simulation.ts'
import type { WorldFrames } from './topDownFrames.ts'

// The authored site's tangent origin is the +Y surface of sample Vesper.
// Its ship origin (local metres) selects a radial contact on that sphere.
// This is a deterministic illustrative binding, not canonical planet generation.
// Both presentations consume this domain value; only explicit domain actions
// may replace the surface binding.
export function startingScoutLocation(): ScoutLocation {
  const body = SAMPLE_BODIES.find(body => body.id === 'vesper')!
  const [x, z] = relativeM(STARTING_SITE.shipOrigin, ORIGIN)
  const radius = physicalRadiusM(body)
  const radial = [x, Math.sqrt(radius * radius - x * x - z * z), z] as const
  const planetOrigin = translate(body.position, [0, radius, 0])
  const frames: WorldFrames = {
    planet: { id: 'planet:fixture', universeOrigin: planetOrigin, angle: 0 },
    ship: { id: 'ship:scout', parent: 'planet:fixture', origin: STARTING_SITE.shipOrigin, angle: STARTING_SITE.shipAngle },
  }
  const ship: ShipState = {
    ...initialShip(), position: translate(body.position, radial),
    yaw: STARTING_SITE.shipAngle, mode: 'grounded', velocityMps: surfaceVelocity(body, radial).map(v => v === 0 ? 0 : v) as [number, number, number],
    contact: { bodyId: body.id, outcome: 'safe-contact', impactSpeedMps: 0, normal: normal(radial), ageTicks: 60 },
  }
  return { siteId: STARTING_SITE.id, body, frames, ship, binding: STARTING_SITE.id, maneuver: null, scanner: null, boardingDoor: 'open' }
}
// One live object is shared by world and navigation. Save records clone it.
export interface ScoutLocation {
  boardingDoor: 'open' | 'closed'
  siteId: string
  body: typeof SAMPLE_BODIES[number]
  frames: WorldFrames
  ship: ShipState
  binding: string | null
  maneuver: 'takeoff' | 'landing' | null
  scanner: import('./scanner.ts').ScannerSnapshot | null
}
export type ScoutAction = { kind: 'begin-takeoff' } | { kind: 'finish-takeoff' } | { kind: 'begin-landing'; siteId: string; bodyId: string } | { kind: 'finish-landing' }
export function transitionScout(location: ScoutLocation, action: ScoutAction): ScoutLocation {
  const ship = location.ship
  const detached = { ...location.frames, ship: { ...location.frames.ship, parent: null, origin: ORIGIN } }
  if (action.kind === 'begin-takeoff') {
    const denial = takeoffDenial(location)
    if (denial) throw new Error(denial)
    return { ...location, frames: detached, binding: null, maneuver: 'takeoff', ship: { ...ship, mode: 'taking-off', contact: null, throttle: 0, alignment: null } }
  }
  if (action.kind === 'finish-takeoff' && ship.mode === 'taking-off' && location.maneuver === 'takeoff') {
    return { ...location, maneuver: null, ship: { ...ship, mode: 'flying' } }
  }
  if (action.kind === 'begin-landing' && (ship.mode === 'flying' || ship.mode === 'recovering' && ship.contact?.outcome === 'safe-contact') && action.siteId === STARTING_SITE.id && action.bodyId === location.body.id) {
    return { ...location, maneuver: 'landing', ship: { ...ship, mode: 'landing', contact: null, throttle: 0, alignment: null } }
  }
  if (action.kind === 'finish-landing' && ship.mode === 'landing' && location.maneuver === 'landing') {
    const landed = startingScoutLocation()
    if (Math.hypot(...relativeM(ship.position, landed.ship.position)) > .001) throw new Error('Ship must reach the validated surface anchor before landing completes')
    return { ...landed, boardingDoor: location.boardingDoor, scanner: location.scanner, ship: { ...landed.ship, tick: ship.tick, massKg: ship.massKg } }
  }
  throw new Error('Invalid Scout transition or unsupported surface site')
}

/** Physical admission is shared by domain commands and the session authority. */
export function takeoffDenial(location: ScoutLocation): string | null {
  if (!Number.isFinite(location.ship.massKg) || location.ship.massKg < DRY_MASS_KG || !Number.isSafeInteger(location.ship.tick) || location.ship.tick < 0) return 'Scout state is invalid.'
  if (location.ship.mode === 'destroyed') return 'Ship is destroyed.'
  if (location.maneuver !== null) return 'A Scout maneuver is already running.'
  if (location.ship.mode !== 'grounded') return 'Scout must be grounded before takeoff.'
  const site = startingScoutLocation()
  if (location.frames.ship.id !== site.frames.ship.id || location.frames.planet.id !== site.frames.planet.id
    || location.binding !== site.binding || location.siteId !== site.siteId || location.body.id !== site.body.id
    || location.frames.ship.parent !== site.frames.ship.parent || location.frames.ship.angle !== site.frames.ship.angle
    || !(Math.hypot(...relativeM(location.frames.ship.origin, site.frames.ship.origin)) <= 1e-6)
    || !(Math.hypot(...relativeM(location.ship.position, site.ship.position)) <= 1e-6)
    || location.ship.yaw !== site.ship.yaw || location.ship.pitch !== 0
    || location.frames.planet.angle !== site.frames.planet.angle
    || !(Math.hypot(...relativeM(location.frames.planet.universeOrigin, site.frames.planet.universeOrigin)) <= 1e-6)
    || location.ship.contact?.bodyId !== site.body.id || location.ship.contact.outcome !== 'safe-contact'
    || location.ship.velocityMps.some((v, i) => v !== site.ship.velocityMps[i])) return 'Scout is not at a validated supported surface site.'
  if (location.ship.alignment !== null || location.ship.throttle !== 0) return 'Stop the active ship operation before takeoff.'
  if (location.boardingDoor !== 'closed') return 'Close the aft boarding door before takeoff.'
  return null
}
