import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ORIGIN, translate } from './coordinates.ts'
import { opticalRayHit, opticalSpheres, MAX_OPTICAL_BODIES } from './opticalSky.ts'
import { startingScoutLocation } from './scoutLocation.ts'
import { SAMPLE_BODIES, type Body } from './sampleUniverse.ts'
import { CatalogIndex, Scanner } from './scanner.ts'
const body: Body = { id: 'test', name: 'Hidden identity', kind: 'moon', radiusKm: 1, position: translate(ORIGIN, [0, 0, -10000]) }
test('physical angular sizes and telescope scaling without displacement', () => {
 const [sphere] = opticalSpheres([body], ORIGIN)
 assert.deepEqual(sphere.direction, [0, 0, -1])
 assert.equal(sphere.angularRadius, Math.asin(0.1))
 for (const fov of [75, 0.1]) assert.ok(Math.tan(sphere.angularRadius) / Math.tan(fov * Math.PI / 360) * 400 > 0)
 assert.equal(opticalRayHit(sphere, [0, 0, -1]), 9000)
 assert.equal(opticalRayHit(sphere, [1, 0, 0]), null)
 assert.equal(opticalRayHit(sphere, [0, 0, 1]), null)
})
test('split translation retains identical data and grounded metre clearance', () => {
 const origin = { cell: [10n ** 24n, -(10n ** 24n), 10n ** 30n] as const, offsetM: [0, 0, 0] as const }
 assert.deepEqual(opticalSpheres([{ ...body, position: translate(origin, [0, 0, -10000]) }], origin), opticalSpheres([body], ORIGIN))
 const site = startingScoutLocation(), sphere = opticalSpheres([site.body], site.ship.position)[0]
 assert.ok(Math.abs(opticalRayHit(sphere, sphere.direction)! - 5) < 0.001)
 assert.equal(opticalRayHit(sphere, [0, 1, 0]), null)
 assert.ok(opticalRayHit(sphere, [0, -1, 0])! < 5.001)
})
test('foreground blocks distant unresolved moon; underground has no sky', () => {
 const spheres = opticalSpheres([body, { ...body, radiusKm: 0.00001, position: translate(ORIGIN, [0, 0, -1e11]) }], ORIGIN)
 assert.ok(opticalRayHit(spheres[0], [0, 0, -1])! < opticalRayHit(spheres[1], [0, 0, -1])!)
 assert.equal(opticalRayHit(opticalSpheres([{ ...body, position: ORIGIN }], ORIGIN)[0], [1, 0, 0]), 0)
})
test('optical geometry carries no identity and is independent of scanner pulses', () => {
 const scanner = new Scanner(new CatalogIndex(SAMPLE_BODIES)), before = opticalSpheres(SAMPLE_BODIES, ORIGIN)
 assert.equal(scanner.contacts.length, 0)
 scanner.select('vesper'); assert.equal(scanner.selected, null)
 scanner.update(300, ORIGIN); assert.ok(scanner.contacts.length > 0)
 assert.deepEqual(opticalSpheres(SAMPLE_BODIES, ORIGIN), before)
 assert.ok(before.every(s => !('name' in s) && !('id' in s)))
})
test('invalid geometry and excessive catalogs fail explicitly', () => {
 assert.throws(() => opticalSpheres([{ ...body, radiusKm: NaN }], ORIGIN), RangeError)
 assert.throws(() => opticalSpheres(Array(MAX_OPTICAL_BODIES + 1).fill(body), ORIGIN), RangeError)
})
