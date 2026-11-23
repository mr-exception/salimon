import { test, expect } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState } from '../src/topDownRuntime.ts'
import { FABRICATOR_RECIPE_ID, MARKER_KIT_ID } from '../src/scoutFabricator.ts'
import { boot } from './worldJourney'

// E05 tests the production clock with a domain-admitted job. Real console
// start/claim controls and their mine-to-craft journey belong to E06 (#878).
for (const powered of [true, false]) test(`FABRICATOR-877 production ${powered ? 'paid completion' : 'power interruption'} survives pause, hidden tab and reload`, async ({ page }, info) => {
  let raw: string | null = null
  const s = new ResourceSession({ getItem: () => raw, setItem: (_, value) => { raw = value } }, createTopDownState('interior', true), undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
  expect(s.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 })).toBe(true)
  expect(s.depositFabricatorInput('mineral', 1000, s.snapshot.revision)).toBe(true)
  if (!powered) s.advance({ ...s.world, tick: 1 }, { watts: s.snapshot.core.reserveJ60 })
  expect(s.startFabricator(FABRICATOR_RECIPE_ID, s.snapshot.revision)).toBe(true)
  const admitted = s.snapshot
  await page.addInitScript(({ key, raw }) => { if (!localStorage.getItem(key)) localStorage.setItem(key, raw) }, { key: RESOURCE_SAVE_KEY, raw: raw! })
  await boot(page, '', 'http://127.0.0.1:4175')
  const saved = () => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  await page.clock.runFor(1000)
  const running = await saved()
  expect(running.fabricator.job.remainingTicks).toBe(powered ? 1200 - (running.core.spentJ60 / 5000) : 1200)
  expect(running.fabricator.job.powerPaused).toBe(!powered)
  await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  const paused = await saved(); await page.clock.runFor(2000)
  expect(await saved()).toEqual(paused)
  await page.getByRole('button', { name: 'Resume [P]', exact: true }).click()
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: true }); document.dispatchEvent(new Event('visibilitychange')) })
  const hidden = await saved(); await page.clock.runFor(2000)
  expect(await saved()).toEqual(hidden)
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: false }); document.dispatchEvent(new Event('visibilitychange')) })
  await page.reload()
  await expect(page.getByRole('button', { name: 'Pause [P]', exact: true })).toBeVisible()
  expect((await saved()).fabricator).toEqual(hidden.fabricator)
  await page.clock.runFor(powered ? 21000 : 1000)
  const final = await saved()
  expect(final.inventory.items[MARKER_KIT_ID]).toBeUndefined()
  if (powered) {
    expect(final.fabricator.job).toBeNull()
    expect(final.fabricator.tray).toEqual({ jobId: 1, itemId: MARKER_KIT_ID, count: 1 })
    expect(final.core.spentJ60).toBe(100000 * 60)
    await page.clock.runFor(1000)
    expect((await saved()).fabricator).toEqual(final.fabricator)
  } else {
    expect(final.fabricator.job.remainingTicks).toBe(1200)
    expect(final.fabricator.job.powerPaused).toBe(true)
    expect(final.core.reserveJ60).toBe(0)
  }
  await page.screenshot({ path: info.outputPath('production.png') })
  await info.attach('fabricator-records.json', { body: JSON.stringify({ admitted, running, paused, hidden, final }, (_, value) => typeof value === 'bigint' ? value.toString() : value, 2), contentType: 'application/json' })
})
