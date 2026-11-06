import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ORIGIN, translate, type Position } from './coordinates.ts'
import { ObservationCameraState } from './observationCamera.ts'
import { perspectiveMarkers, currentContacts, selectContact, selectedContact } from './perspectiveContacts.ts'
import { CatalogIndex, Scanner, SCANNER_RADIUS_M } from './scanner.ts'
import type { Body } from './sampleUniverse.ts'
import type { Vec3 } from './math.ts'
const body = (id: string, vector: Vec3, radiusKm = 0.001, origin = ORIGIN): Body => ({ id, name: id, kind: 'moon', radiusKm, position: translate(origin, vector) })
const view = new ObservationCameraState({ yaw: 0, pitch: 0 })
const project = (contacts: Body[], geometry = contacts, origin: Position = ORIGIN, fov = 75) => perspectiveMarkers(contacts, geometry, origin, { ...view.snapshot(), fov }, 1280, 800)
test('scanner alone authorizes selection, cap inclusive, pulse timing and eviction independent of zoom', () => {
 const b = body('edge', [0, 0, -SCANNER_RADIUS_M]), scanner = new Scanner(new CatalogIndex([b]))
 scanner.update(299, ORIGIN); selectContact(scanner, ORIGIN, b.id)
 assert.equal(scanner.selected, null); assert.deepEqual(project([...currentContacts(scanner, ORIGIN)], [b]), [])
 scanner.update(300, ORIGIN); selectContact(scanner, ORIGIN, b.id)
 assert.equal(selectedContact(scanner, ORIGIN)?.id, b.id)
 for (const fov of [75, 0.1]) assert.equal(project([...scanner.contacts], [b], ORIGIN, fov).length, 1)
 const moved = translate(ORIGIN, [0, 0, 1])
 assert.equal(selectedContact(scanner, moved), undefined)
 assert.equal(currentContacts(scanner, moved).length, 0)
 scanner.update(301, moved); assert.equal(scanner.selected, null)
 scanner.update(302, ORIGIN); assert.equal(scanner.contacts.length, 0)
 scanner.update(600, ORIGIN); assert.equal(scanner.contacts.length, 1)
})
test('explicit camera half-space and FOV reject hit targets without affecting browser IDs', () => {
 const bodies = [body('front', [0, 0, -100]), body('back', [0, 0, 100]), body('side', [200, 0, -100]), body('plane', [100, 0, 0])]
 assert.deepEqual(project(bodies).flatMap(m => m.ids), ['front'])
 assert.equal(bodies.length, 4)
 assert.deepEqual(perspectiveMarkers(bodies, bodies, ORIGIN, view.snapshot(), 0, 800), [])
})
test('physical near-surface depth and limb edges; catalog occluder cannot add a marker', () => {
 const planet = body('planet', [0, 0, -10000], 1), hidden = body('hidden', [0, 0, -20000])
 const inside = body('inside-limb', [1990, 0, -20000]), outside = body('outside-limb', [2020, 0, -20000])
 assert.deepEqual(project([hidden, inside, outside], [planet, hidden, inside, outside]).flatMap(m => m.ids), ['outside-limb'])
 assert.deepEqual(project([planet], [planet]).flatMap(m => m.ids), ['planet'])
 const behind = body('rear-planet', [0, 0, -40000], 1)
 assert.equal(project([hidden], [behind, hidden]).length, 1)
 const ground = body('ground', [0, -1001, 0], 1), below = body('below', [0, -10000, -10000])
 assert.equal(project([below], [ground, below]).length, 0)
})
test('tiny detected bodies, huge split origin, 0.1 degree projection and motion stability', () => {
 const far = { cell: [10n ** 24n, -(10n ** 30n), 10n ** 25n] as const, offsetM: [0, 0, 0] as const }
 const bodies = [body('tiny', [20, 0, -100000], 1e-8)]
 const translated = [body('tiny', [20, 0, -100000], 1e-8, far)]
 assert.deepEqual(project(translated, translated, far, 0.1), project(bodies, bodies, ORIGIN, 0.1))
 const initial = project(bodies, bodies, ORIGIN, 0.1)[0]
 assert.ok(Math.abs(initial.x - (640 + 20 / 100000 / Math.tan(Math.PI / 3600) * 400)) < 1e-8)
 const camera = new ObservationCameraState({ yaw: 0, pitch: 0 })
 camera.restore({ ...camera.snapshot(), fov: 0.1 })
 let previous = initial.x
 for (let i = 0; i < 50; i++) {
  camera.look(0.0001, 0)
  const markers = perspectiveMarkers(bodies, bodies, ORIGIN, camera.snapshot(), 1280, 800)
  assert.equal(markers.length, 1); assert.ok(Math.abs(markers[0].x - previous) < 1); previous = markers[0].x
 }
})
test('dense/transitive overlap groups retain unique scanned IDs in stable order', () => {
 const bodies = Array.from({ length: 25 }, (_, i) => body(`body-${i}`, [i * 0.2, 0, -10000]))
 const groups = project([...bodies, bodies[0]], [])
 assert.equal(groups.length, 1)
 assert.deepEqual(groups.flatMap(m => m.ids).sort(), bodies.map(b => b.id).sort())
 assert.deepEqual(project([...bodies].reverse(), []), project(bodies, []))
})
