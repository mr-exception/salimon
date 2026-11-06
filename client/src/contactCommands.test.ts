import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ORIGIN, translate } from './coordinates.ts'
import { contactCommandAvailability, lookAtContact, setContactDirection } from './contactCommands.ts'
import { ObservationCameraState } from './observationCamera.ts'
import { SAMPLE_BODIES, type Body } from './sampleUniverse.ts'
import { CatalogIndex, Scanner, SCANNER_RADIUS_M } from './scanner.ts'
import { startingScoutLocation } from './scoutLocation.ts'
import { initialShip, idleControls, stepShip } from './simulation.ts'

test('scanner commands mutate view or ship independently; manual ship input cancels only alignment', () => {
 const body: Body = { id: 'target', name: 'Target', kind: 'moon', radiusKm: 1, position: translate(ORIGIN, [10000, 0, 10000]) }
 const scanner = new Scanner(new CatalogIndex([body])), ship = initialShip(), saved = structuredClone(ship), view = new ObservationCameraState(ship)
 assert.equal(lookAtContact(scanner, ship, view, body.id), false)
 assert.equal(setContactDirection(scanner, ship, body.id), ship)
 scanner.update(300, ship.position)
 assert.equal(lookAtContact(scanner, ship, view, 'unknown'), false)
 assert.equal(lookAtContact(scanner, ship, view, body.id), true); assert.deepEqual(ship, saved)
 const before = view.snapshot(), aligned = setContactDirection(scanner, ship, body.id)
 assert.ok(aligned.alignment); assert.deepEqual(view.snapshot(), before)
 const turned = stepShip(aligned, idleControls()); assert.notEqual(turned.yaw, ship.yaw)
 assert.deepEqual(turned.velocityMps, ship.velocityMps)
 assert.equal(stepShip(aligned, { ...idleControls(), left: true }).alignment, null)
 assert.equal(view.lookingAt, true)
 view.look(0.1, 0); assert.ok(aligned.alignment); assert.equal(view.lookingAt, false)
 assert.equal(setContactDirection(scanner, ship, body.id, false), ship)
 lookAtContact(scanner, ship, view, body.id)
 const moved = { ...ship, position: translate(ship.position, [SCANNER_RADIUS_M * 2, 0, 0]) }
 scanner.select(body.id); scanner.update(301, moved.position)
 assert.equal(scanner.selected, null); assert.equal(lookAtContact(scanner, moved, view, body.id), false)
 view.advanceLookAt(1); assert.equal(view.lookingAt, false)
})
test('shared grounded Scout excludes Vesper center but allows other detected bodies', () => {
 const { ship, body } = startingScoutLocation(), view = new ObservationCameraState(ship)
 const scanner = new Scanner(new CatalogIndex(SAMPLE_BODIES)); scanner.update(300, ship.position)
 assert.deepEqual(contactCommandAvailability(ship, body), { currentPlanet: true, lookAt: false, setDirection: false })
 assert.equal(lookAtContact(scanner, ship, view, body.id), false)
 const other = scanner.contacts.find(b => b.id !== body.id)!
 assert.ok(other); assert.equal(lookAtContact(scanner, ship, view, other.id), true)
 assert.equal(setContactDirection(scanner, ship, other.id), ship)
 assert.equal(setContactDirection(scanner, ship, body.id), ship)
})
