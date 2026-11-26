import test from 'node:test'
import assert from 'node:assert/strict'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { CatalogIndex, Scanner, SCANNER_RADIUS_M } from './scanner.ts'
import { selectedContact } from './perspectiveContacts.ts'
import { assessScannerEnvironment } from './scannerEnvironment.ts'
import { bodyEnvironmentSummary, samplePlanetEnvironment } from './planetEnvironment.ts'
import { freshEquipment, type EquipmentState } from './equipment.ts'
import { ORIGIN, translate } from './coordinates.ts'

const vesper = SAMPLE_BODIES.find(body => body.id === 'vesper')!
const moon = SAMPLE_BODIES.find(body => body.id === 'vesper-moon')!
const surface = { bodyId: vesper.id, radialM: [0, 6200005, 0] as const }
const source = (equipment = freshEquipment()) => ({
  environmentAt: samplePlanetEnvironment,
  environmentSummary: bodyEnvironmentSummary,
  snapshot: { inventory: { equipment } },
})

test('scanner environment requires a live selected contact and known planetary publication', () => {
  const scanner = new Scanner(new CatalogIndex(SAMPLE_BODIES))
  const unreadable = { ...source(), environmentSummary() { throw Error('Unscanned environment requested') } }
  assert.equal(assessScannerEnvironment(selectedContact(scanner, ORIGIN), unreadable), undefined)
  assert.equal(assessScannerEnvironment(SAMPLE_BODIES[0], unreadable), undefined)
  scanner.update(300, ORIGIN)
  scanner.select(vesper.id)
  assert.equal(assessScannerEnvironment(selectedContact(scanner, ORIGIN), source())?.summary.bodyId, vesper.id)
  scanner.update(301, translate(ORIGIN, [2 * SCANNER_RADIUS_M, 0, 0]))
  assert.equal(assessScannerEnvironment(selectedContact(scanner, ORIGIN), unreadable), undefined)
})

test('site preview uses canonical coordinates and distinguishes body baselines from the safe exception', () => {
  const unknown = assessScannerEnvironment(vesper, source())!
  assert.deepEqual(unknown.summary.temperatureRangeC, [-180, 180])
  assert.ok(unknown.rows.every(row => row.status === 'Site requirement unknown'))
  const local = assessScannerEnvironment(vesper, source(), surface)!
  assert.deepEqual(local.local, samplePlanetEnvironment(surface))
  assert.ok(local.rows.every(row => row.status === 'No protection needed'))
  const elsewhere = assessScannerEnvironment(moon, source(), surface)!
  assert.equal(elsewhere.local, undefined)
  assert.equal(elsewhere.rows[1].incoming, bodyEnvironmentSummary(moon.id).radiationHpPerSecond)
})

test('assessment uses only functioning installed modules, with exact hazard thresholds', () => {
  const equipment = freshEquipment()
  const hazardousSite = { ...surface, radialM: [6200005, 0, 0] as const }
  const active = assessScannerEnvironment(vesper, source(equipment), hazardousSite)!
  assert.equal(active.rows[0].available, 2)
  assert.equal(active.rows[0].status, 'Insufficient protection')
  const destroy = (index: number): EquipmentState => ({
    ...equipment, instances: equipment.instances.map((gear, i) => i === index ? { ...gear, remaining: '0/1', location: index === 1 ? { kind: 'destroyed' } : gear.location } : gear),
  })
  for (const index of [0, 1]) {
    const assessment = assessScannerEnvironment(vesper, source(destroy(index)), hazardousSite)!
    assert.equal(assessment.rows[0].available, 0)
    assert.equal(assessment.rows[0].status, 'Missing protection')
  }
  const exactThreshold = { ...source(), environmentAt: () => ({ ...samplePlanetEnvironment(surface), temperatureC: 44 }) }
  assert.equal(assessScannerEnvironment(vesper, exactThreshold, surface)?.rows[0].status, 'Protection sufficient')
})
