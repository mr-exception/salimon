import { Quaternion, Vector3 } from 'three'
import { relativeM, type Position } from './coordinates.ts'
import { length } from './math.ts'
import type { ObservationSnapshot } from './observationCamera.ts'
import { opticalRayHit, opticalSpheres } from './opticalSky.ts'
import type { Body } from './sampleUniverse.ts'
import { SCANNER_RADIUS_M, type Scanner } from './scanner.ts'

export interface PerspectiveMarker { id: string; ids: string[]; x: number; y: number }
/** Defense at the presentation/action boundary; never acquires a contact. */
export function currentContacts(scanner: Scanner, origin: Position): readonly Body[] {
 return scanner.contacts.filter(body => length(relativeM(body.position, origin)) <= SCANNER_RADIUS_M)
}
export function selectedContact(scanner: Scanner, origin: Position): Body | undefined {
 return currentContacts(scanner, origin).find(body => body.id === scanner.selected)
}
export function selectContact(scanner: Scanner, origin: Position, id: string): void {
 if (currentContacts(scanner, origin).some(body => body.id === id)) scanner.select(id)
}
/** Catalog geometry can suppress markers but can never authorize them. Projection
 * uses unit directions, so physical near/far distances cannot clip astronomy.
 * A marker denotes the body's center ray: a partially occulted center is hidden.
 * The shared globe is also the simplified grounded terrain/horizon occluder.
 */
export function perspectiveMarkers(contacts: readonly Body[], geometry: readonly Body[], origin: Position,
 view: ObservationSnapshot, width: number, height: number): PerspectiveMarker[] {
 if (![width, height, view.fov, ...view.quaternion].every(Number.isFinite) || width <= 0 || height <= 0 || view.fov < 0.1 || view.fov > 75) return []
 const inverse = new Quaternion(...view.quaternion).normalize().invert()
 const spheres = opticalSpheres(geometry, origin)
 const tan = Math.tan(view.fov * Math.PI / 360), aspect = width / height
 const markers: PerspectiveMarker[] = [], seen = new Set<string>()
 for (const body of contacts) {
  if (seen.has(body.id)) continue
  seen.add(body.id)
  const vector = relativeM(body.position, origin), distance = length(vector)
  if (!Number.isFinite(distance) || distance <= 0 || distance > SCANNER_RADIUS_M) continue
  const ray = new Vector3(...vector).divideScalar(distance)
  const local = ray.clone().applyQuaternion(inverse)
  if (local.z >= 0) continue
  const x = local.x / (-local.z * tan * aspect), y = local.y / (-local.z * tan)
  if (![x, y].every(Number.isFinite) || Math.abs(x) > 1 || Math.abs(y) > 1) continue
  // Compare physical surface depths, not centers or GPU depth-buffer values.
  const targetHit = opticalRayHit(opticalSpheres([body], origin)[0], ray.toArray())
  if (targetHit === null) continue
  if (spheres.some((sphere, i) => geometry[i].id !== body.id && (() => {
   const hit = opticalRayHit(sphere, ray.toArray())
   return hit !== null && hit < targetHit - Math.max(1e-6, targetHit * 1e-12)
  })())) continue
  markers.push({ id: body.id, ids: [body.id], x: (x + 1) * width / 2, y: (1 - y) * height / 2 })
 }
 // Connected screen-overlap components; merge transitively without losing IDs.
 const groups: PerspectiveMarker[] = []
 const pending = markers.sort((a, b) => a.id.localeCompare(b.id))
 while (pending.length) {
  const members = [pending.shift()!]
  for (let i = 0; i < members.length; i++) {
   for (let j = pending.length - 1; j >= 0; j--) {
    if (Math.hypot(members[i].x - pending[j].x, members[i].y - pending[j].y) < 32) members.push(...pending.splice(j, 1))
   }
  }
  members.sort((a, b) => a.id.localeCompare(b.id))
  groups.push({ ...members[0], ids: members.map(m => m.id) })
 }
 return groups
}
