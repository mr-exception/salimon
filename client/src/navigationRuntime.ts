import { resolveContacts } from './contact.ts'
import { CatalogIndex, Scanner } from './scanner.ts'
import { NavigationClock, idleControls, initialShip, type Controls } from './simulation.ts'
import { type Body } from './sampleUniverse.ts'
// Single production composition path for fixed-step motion, contact and scanning.
export class NavigationRuntime {
 readonly bodies:readonly Body[]
 readonly clock=new NavigationClock()
 private readonly index:CatalogIndex
 scanner:Scanner
 constructor(bodies:readonly Body[]) {this.bodies=bodies;this.index=new CatalogIndex(bodies);this.scanner=new Scanner(this.index)}
 advance(seconds:number,controls:Controls=idleControls()) {
  return this.clock.advance(seconds,controls,(before,after)=>{
   const next=resolveContacts(before,after,this.bodies)
   this.scanner.update(next.tick,next.position)
   return next
  })
 }
 reset() {this.clock.reset(initialShip());this.scanner=new Scanner(this.index)}
}
