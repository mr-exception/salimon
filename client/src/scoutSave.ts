import { sampleTakeoff, TAKEOFF_TICKS, TAKEOFF_PEAK_ACCELERATION } from './takeoff.ts'
import { physicalRadiusM } from './contact.ts'
import { CELL_SIZE_M, relativeM } from './coordinates.ts'
import { startingScoutLocation, type ScoutLocation } from './scoutLocation.ts'
import { CatalogIndex, Scanner } from './scanner.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { DRY_MASS_KG, type ShipState } from './simulation.ts'

export type ScoutSave = Pick<ScoutLocation, 'ship' | 'binding' | 'maneuver' | 'scanner' | 'boardingDoor' | 'launch'>
export const captureScout = (location: ScoutLocation): ScoutSave => structuredClone({ ship: location.ship, binding: location.binding, maneuver: location.maneuver, scanner: location.scanner, boardingDoor: location.boardingDoor, launch: location.launch })
export function decodeScout(value: unknown, legacy = false): ScoutLocation {
  let data = value as ScoutSave
  if (!data || typeof data !== 'object' || Object.keys(data).filter(k=>k!=='launch').sort().join() !== (legacy ? 'binding,maneuver,scanner,ship' : 'binding,boardingDoor,maneuver,scanner,ship')) throw new Error('Invalid Scout snapshot')
  if (legacy) data = { ...data, boardingDoor: data.binding === null ? 'closed' : 'open' }
  if (data.boardingDoor !== 'open' && data.boardingDoor !== 'closed') throw new Error('Invalid boarding door')
  if (data.binding === null && data.boardingDoor !== 'closed') throw new Error('Open airborne boarding door')
  data = { ...data, launch: data.launch ?? null }
  const base = startingScoutLocation(), ship = data.ship as ShipState
  if (!ship || Object.keys(ship).sort().join() !== Object.keys(base.ship).sort().join()) throw new Error('Invalid ship fields')
  const vector = (v: unknown): v is [number, number, number] => Array.isArray(v) && v.length === 3 && v.every(x=>typeof x === 'number' && Number.isFinite(x))
  if (!ship.position || !Array.isArray(ship.position.cell) || ship.position.cell.length !== 3 || !vector(ship.position.offsetM) || ship.position.offsetM.some(v=>v < -CELL_SIZE_M/2 || v >= CELL_SIZE_M/2)) throw new Error('Invalid ship position')
  const cell = ship.position.cell.map(v=> {
    if (typeof v !== 'string' || !/^-?(0|[1-9][0-9]{0,127})$/.test(v)) throw new Error('Invalid ship cell')
    return BigInt(v)
  }) as [bigint,bigint,bigint]
  if (!vector(ship.velocityMps) || ![ship.yaw,ship.pitch,ship.massKg,ship.throttle].every(v=>typeof v === 'number' && Number.isFinite(v)) || ship.massKg < DRY_MASS_KG || Math.abs(ship.pitch)>Math.PI/2 || Math.abs(ship.throttle)>100 || !Number.isSafeInteger(ship.tick) || ship.tick<0) throw new Error('Invalid ship state')
  if (ship.alignment !== null && (!ship.alignment || !Number.isFinite(ship.alignment.yaw) || !Number.isFinite(ship.alignment.pitch) || Math.abs(ship.alignment.pitch)>Math.PI/2)) throw new Error('Invalid alignment')
  if (ship.contact !== null && (!ship.contact || !SAMPLE_BODIES.some(b=>b.id===ship.contact!.bodyId) || !['safe-contact','crash','destroyed'].includes(ship.contact.outcome) || !vector(ship.contact.normal) || Math.abs(Math.hypot(...ship.contact.normal)-1)>1e-6 || !Number.isFinite(ship.contact.impactSpeedMps) || ship.contact.impactSpeedMps<0 || !Number.isSafeInteger(ship.contact.ageTicks) || ship.contact.ageTicks<0)) throw new Error('Invalid contact')
  const parsed = { ...ship, position: { cell, offsetM: ship.position.offsetM } }
  if (ship.contact) {
    const body = SAMPLE_BODIES.find(b => b.id === ship.contact!.bodyId)!
    const radial = relativeM(parsed.position, body.position), radius = Math.hypot(...radial)
    // Recovery rebounds are bounded to two metres; a forged contact must not
    // teleport the ship when resolveContacts resumes its surface constraint.
    if (radius < physicalRadiusM(body) - .001 || radius > physicalRadiusM(body) + 2.001 || Math.hypot(...radial.map((v, i) => v / radius - ship.contact!.normal[i])) > 1e-6) throw new Error('Inconsistent physical contact pose')
  }
  if (data.binding === base.binding) {
    if (ship.mode !== 'grounded' || data.maneuver !== null || JSON.stringify(ship.contact) !== JSON.stringify(base.ship.contact) || Math.hypot(...relativeM(parsed.position,base.ship.position))>1e-6 || ship.yaw!==base.ship.yaw || ship.pitch!==0 || ship.throttle!==0 || ship.alignment!==null || ship.velocityMps.some((v,i)=>v!==base.ship.velocityMps[i])) throw new Error('Inconsistent surface binding')
  } else {
    if (data.binding !== null || !['taking-off','flying','landing','recovering','destroyed'].includes(ship.mode) || data.maneuver !== (ship.mode==='taking-off'?'takeoff':ship.mode==='landing'?'landing':null)) throw new Error('Inconsistent flight lifecycle')
    if ((ship.mode === 'recovering' && (!ship.contact || ship.contact.outcome === 'destroyed')) || (ship.mode === 'destroyed' && ship.contact?.outcome !== 'destroyed')) throw new Error('Missing physical contact')
    if (ship.mode !== 'flying' && (ship.throttle!==0 || ship.alignment!==null)) throw new Error('Invalid maneuver controls')
    if ((ship.mode==='taking-off' || ship.mode==='flying' || ship.mode==='landing') && ship.contact!==null) throw new Error('Unexpected contact')
    base.frames = { ...base.frames, ship: { ...base.frames.ship, parent: null, origin: {cell:[0n,0n,0n],offsetM:[0,0,0]} } }
  }
  if (ship.mode === 'taking-off') {
    const plan = data.launch
    if (!plan || Object.keys(plan).sort().join() !== 'massKg,startTick' || !Number.isSafeInteger(plan.startTick) || plan.startTick < 0 || !Number.isSafeInteger(plan.startTick + TAKEOFF_TICKS) || plan.massKg !== ship.massKg || ship.tick < plan.startTick || ship.tick >= plan.startTick + TAKEOFF_TICKS || plan.massKg * TAKEOFF_PEAK_ACCELERATION > 50e6) throw new Error('Invalid takeoff plan')
    const expected = sampleTakeoff(startingScoutLocation().ship, base.body, plan, ship.tick)
    if (Math.hypot(...relativeM(parsed.position,expected.position)) > 1e-6 || ship.velocityMps.some((v,i)=>Math.abs(v-expected.velocityMps[i])>1e-9) || ship.yaw !== expected.yaw || ship.pitch !== expected.pitch) throw new Error('Inconsistent takeoff trajectory')
  } else if (data.launch !== null) throw new Error('Unexpected takeoff plan')
  if (data.scanner !== null) {
    const scanner = new Scanner(new CatalogIndex(SAMPLE_BODIES))
    scanner.restore(data.scanner,SAMPLE_BODIES)
    if (data.scanner.lastTick !== ship.tick) throw new Error('Scanner clock mismatch')
  } else if (ship.tick !== 0) throw new Error('Missing scanner clock')
  return { ...base, ...data, ship: parsed }
}
