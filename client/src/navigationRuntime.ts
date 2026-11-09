import { sampleTakeoff, TAKEOFF_TICKS } from './takeoff.ts'
import { startingScoutLocation } from './scoutLocation.ts'
import type { ScoutLocation } from './scoutLocation.ts'
import { resolveContacts } from './contact.ts'
import { CatalogIndex, Scanner } from './scanner.ts'
import { NavigationClock, idleControls, initialShip, type Controls } from './simulation.ts'
import { type Body } from './sampleUniverse.ts'
// Single production composition path for fixed-step motion, contact and scanning.
export class NavigationRuntime {
 readonly bodies:readonly Body[]
 readonly location?: ScoutLocation
 get surfaceLocation() { return this.location?.binding ? this.location : undefined }
 private scanRecord: ScoutLocation['scanner'] = null
 readonly clock=new NavigationClock()
 private readonly index:CatalogIndex
 scanner:Scanner
 constructor(bodies:readonly Body[], surfaceLocation?: ScoutLocation) {
  if(surfaceLocation && !bodies.includes(surfaceLocation.body)) throw new Error('Surface body must belong to the navigation catalog')
  this.location=surfaceLocation
  if (surfaceLocation) Object.defineProperty(this.clock, 'state', { get: () => surfaceLocation.ship, set: value => { surfaceLocation.ship = value } })
  this.bodies=bodies;this.index=new CatalogIndex(bodies);this.scanner=new Scanner(this.index, snapshot => { if(this.location) { this.location.scanner=snapshot;this.scanRecord=snapshot } })
  if(surfaceLocation) this.clock.reset(surfaceLocation.ship)
 }
 advance(seconds:number,controls:Controls=idleControls()) {
  if (this.location?.scanner && this.location.scanner !== this.scanRecord) this.scanner.restore(this.location.scanner, this.bodies)
  const result = this.clock.advance(seconds,controls,(before,after)=>{
   if (this.location?.launch && before.mode === 'taking-off') {
    const plan = this.location.launch
    const next = sampleTakeoff(startingScoutLocation().ship, this.location.body, plan, after.tick)
    if (after.tick >= plan.startTick + TAKEOFF_TICKS) { this.location.launch = null; this.location.maneuver = null }
    this.scanner.update(next.tick,next.position)
    return next
   }
   // Contact remains physical in flight, but cannot invent a playable site.
   const contact = this.location && !this.location.binding ? resolveContacts(before,after,this.bodies) : null
   let next=this.surfaceLocation
    ? {...this.surfaceLocation.ship, tick:after.tick, massKg:after.massKg}
    : contact ?? resolveContacts(before,after,this.bodies)
   if (this.location && !this.location.binding && next.mode === 'grounded') next={...next,mode:'recovering'}
   this.scanner.update(next.tick,next.position)
   return next
  })
  if (this.location) { this.location.scanner=this.scanner.snapshot(); this.scanRecord=this.location.scanner }
  return result
 }
 reset() {if(this.location)return;this.clock.reset(initialShip());this.scanner=new Scanner(this.index)}
}
