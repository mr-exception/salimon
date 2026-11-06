import type { ScoutLocation } from './scoutLocation.ts'
import { resolveContacts } from './contact.ts'
import { CatalogIndex, Scanner } from './scanner.ts'
import { NavigationClock, idleControls, initialShip, type Controls } from './simulation.ts'
import { type Body } from './sampleUniverse.ts'
// Single production composition path for fixed-step motion, contact and scanning.
export class NavigationRuntime {
 readonly bodies:readonly Body[]
 readonly surfaceLocation?: ScoutLocation
 readonly clock=new NavigationClock()
 private readonly index:CatalogIndex
 scanner:Scanner
 constructor(bodies:readonly Body[], surfaceLocation?: ScoutLocation) {
  if(surfaceLocation && !bodies.includes(surfaceLocation.body)) throw new Error('Surface body must belong to the navigation catalog')
  this.surfaceLocation=surfaceLocation
  this.bodies=bodies;this.index=new CatalogIndex(bodies);this.scanner=new Scanner(this.index)
  if(surfaceLocation) this.clock.reset(surfaceLocation.ship)
 }
 advance(seconds:number,controls:Controls=idleControls()) {
  return this.clock.advance(seconds,controls,(before,after)=>{
   // The 2D world currently supports only this immutable landed location.
   // Reassert it at the domain boundary, including orientation and contact;
   // camera/scanner input and even a restart cannot launch or relocate it.
   const next=this.surfaceLocation
    ? {...this.surfaceLocation.ship, tick:after.tick, massKg:after.massKg}
    : resolveContacts(before,after,this.bodies)
   this.scanner.update(next.tick,next.position)
   return next
  })
 }
 reset() {if(this.surfaceLocation)return;this.clock.reset(initialShip());this.scanner=new Scanner(this.index)}
}
