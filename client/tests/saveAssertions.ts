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
  expect(actual.survival).toEqual({ ...before.survival, fatigue: actual.survival.fatigue, offlineAtMs: actual.survival.offlineAtMs })
  expect(actual.survival.fatigue).toBeGreaterThanOrEqual(before.survival.fatigue)
  expect(actual.survival.offlineAtMs).toBeGreaterThanOrEqual(before.survival.offlineAtMs)
}
