import { advanceSurvival } from '../src/survival.ts'
import { decodeScout } from '../src/scoutSave.ts'
import { CatalogIndex, SCAN_TICKS } from '../src/scanner.ts'
import { SAMPLE_BODIES } from '../src/sampleUniverse.ts'
import { expect } from '@playwright/test'
import type { ResourceSave } from '../src/resourceSession.ts'

/** Resource-only actions keep resources and physical state fixed while active time advances. */
export function expectSameResourceState(actual: ResourceSave, before: ResourceSave) {
  const { survival: actualClock, revision: _actualRevision, scout: actualScout, ...actualResources } = actual
  const { survival: beforeClock, revision: _beforeRevision, scout: beforeScout, ...beforeResources } = before
  expect(actualResources).toEqual(beforeResources)
  const { ship: { tick: actualTick, ...actualShip }, scanner: actualScan, ...actualLocation } = actualScout
  const { ship: { tick: beforeTick, ...beforeShip }, scanner: beforeScan, ...beforeLocation } = beforeScout
  expect(actualLocation).toEqual(beforeLocation)
  expect(actualShip).toEqual(beforeShip)
  expect(actualTick - beforeTick).toBeGreaterThanOrEqual(0)
  expect(actualTick - beforeTick).toBe(actualClock.activeTicks - beforeClock.activeTicks)

  // Validate both snapshots, including scanner/ship tick and pulse coherence.
  // A first scan may legitimately discover contacts during an open storage panel.
  decodeScout(beforeScout)
  const location = decodeScout(actualScout)
  if (actualScan) {
    expect(actualScan.selected).toBe(beforeScan?.selected ?? null)
    const discovered = Math.floor(actualTick / SCAN_TICKS) > Math.floor(beforeTick / SCAN_TICKS)
    expect(actualScan.contactIds).toEqual(discovered
      ? new CatalogIndex(SAMPLE_BODIES).query(location.ship.position).map(body => body.id)
      : beforeScan?.contactIds ?? [])
  }
}
export function expectSafeSurvivalReload(actual: ResourceSave, before: ResourceSave) {
  expectSameResourceState(actual, before)
  // pagehide flushes the partial second since the observed periodic checkpoint.
  const ticks = actual.survival.activeTicks - before.survival.activeTicks
  expect(ticks).toBeGreaterThanOrEqual(0); expect(ticks).toBeLessThanOrEqual(60)
  const expected = advanceSurvival(before.survival, ticks)
  expect(actual.survival).toEqual({ ...expected, fatigue: actual.survival.fatigue, offlineAtMs: actual.survival.offlineAtMs })
  expect(actual.survival.fatigue).toBeGreaterThanOrEqual(expected.fatigue)
  expect(actual.survival.offlineAtMs).toBeGreaterThanOrEqual(before.survival.offlineAtMs)
}
