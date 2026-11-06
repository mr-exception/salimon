import { advanceSurvival } from '../src/survival.ts'
import { expect } from '@playwright/test'
import type { ResourceSave } from '../src/resourceSession.ts'
/** Clock-only commits may revise a save while resource state must stay identical. */
export function expectSameResourceState(actual: ResourceSave, before: ResourceSave) {
  const { survival: _actualClock, revision: _actualRevision, ...actualResources } = actual
  const { survival: _beforeClock, revision: _beforeRevision, ...beforeResources } = before
  expect(actualResources).toEqual(beforeResources)
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
