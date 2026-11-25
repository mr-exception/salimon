import { test } from 'node:test'
import assert from 'node:assert/strict'
import { ORIGIN, translate } from './coordinates.ts'
import { opticalRayHit, opticalSpheres, MAX_OPTICAL_BODIES } from './opticalSky.ts'
import { startingScoutLocation } from './scoutLocation.ts'
import { SAMPLE_BODIES, type Body } from './sampleUniverse.ts'
import { CatalogIndex, Scanner } from './scanner.ts'
import { bodyPalette, SURFACE_PIGMENTS } from './bodyPalette.ts'
import { geologicalDefinition, planetDefinition } from './planetDefinition.ts'
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

test('material classification uses catalog kinds, shared pigments and no inferred planet properties', () => {
 const palettes = ['planet', 'moon', 'asteroid', 'star'].map(kind => bodyPalette(kind as Body['kind']))
 assert.equal(new Set(palettes.map(p => JSON.stringify(p))).size, 4)
 for (const p of palettes) for (const c of [...p.color, ...p.accent]) assert.ok(Number.isFinite(c) && c >= 0 && c <= 1)
 const [a, b] = opticalSpheres([{ ...body, kind: 'planet', id: 'gas giant' }, { ...body, kind: 'planet', id: 'ice world' }], ORIGIN)
 assert.deepEqual(a, b)
 const encoded = bodyPalette('planet').color.map(c => Math.round(255 * (c <= .0031308 ? c * 12.92 : 1.055 * c ** (1 / 2.4) - .055)))
 assert.deepEqual(encoded, SURFACE_PIGMENTS.lowland)
})

test('polar ice follows the live canonical publication; generic and legacy bodies stay unclassified', () => {
 const moon = SAMPLE_BODIES.find(b => b.id === 'vesper-moon')!
 assert.equal(opticalSpheres([moon], ORIGIN)[0].polarIce, false)
 assert.equal(opticalSpheres([moon], ORIGIN, id => planetDefinition(id))[0].polarIce, false)
 assert.equal(opticalSpheres([moon], ORIGIN, id => geologicalDefinition(id))[0].polarIce, true)
 assert.equal(opticalSpheres([SAMPLE_BODIES.find(b => b.id === 'vesper')!], ORIGIN, id => geologicalDefinition(id))[0].polarIce, false)
 assert.equal(opticalSpheres([{ ...moon, radiusKm: 1 }], ORIGIN, () => { throw Error('unknown physical publication') })[0].polarIce, false)
})
