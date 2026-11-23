import { test, expect } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState } from '../src/topDownRuntime.ts'
import { CORE_INITIAL_RESERVE_J, SCOUT_CORE_FUEL_ID } from '../src/scoutEnergyCore.ts'
import { boot, walkTo } from './worldJourney'

test.use({ trace: { mode: 'retain-on-failure', snapshots: false, screenshots: false, sources: true } })
test('CORE-875 built single clock converts through cockpit, pause, inactive tab and reload', async ({ page }, info) => {
  let raw: string | null = null
  const session = new ResourceSession({ getItem: () => raw, setItem: (_, value) => { raw = value } }, createTopDownState('interior', true), undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
  expect(session.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 10000 })).toBe(true)
  expect(session.depositCoreFuel('mineral', 10000, session.snapshot.revision)).toBe(true)
  await page.addInitScript(({ key, raw }) => { if (!localStorage.getItem(key)) localStorage.setItem(key, raw) }, { key: RESOURCE_SAVE_KEY, raw: raw! })
  await boot(page, '', 'http://127.0.0.1:4175')
  const saved = () => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  const check = (s: any) => {
    const grams = s.convertedGrams.mineral ?? 0
    expect(s.stores[SCOUT_CORE_FUEL_ID].quantities.mineral + grams).toBe(10000)
    expect(s.core.reserveJ60 + s.core.spentJ60).toBe((CORE_INITIAL_RESERVE_J + grams * 500) * 60)
    // No second view clock: exactly one 10,000/60 J generation budget per tick.
    expect(grams * 500 * 60 + s.core.conversionBudgetJ60).toBe(s.core.tick * 10000)
  }
  await page.clock.runFor(1000)
  const world = await saved(); check(world); expect(world.convertedGrams.mineral).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  const paused = await saved(); await page.clock.runFor(2000); expect(await saved()).toEqual(paused)
  await page.getByRole('button', { name: 'Resume [P]', exact: true }).click()
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: true }); document.dispatchEvent(new Event('visibilitychange')) })
  const hidden = await saved(); await page.clock.runFor(2000); expect(await saved()).toEqual(hidden)
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: false }); document.dispatchEvent(new Event('visibilitychange')) })
  for (const p of [[0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e'); await page.clock.runFor(100)
  await expect(page.getByRole('region', { name: 'Panoramic navigation', exact: true })).toBeVisible()
  await page.clock.runFor(1000)
  const cockpit = await saved(); check(cockpit)
  const backend = await page.locator('canvas').evaluate(c => { const gl = (c as HTMLCanvasElement).getContext('webgl2')!; const ext = gl.getExtension('WEBGL_debug_renderer_info')!; return gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) })
  await page.screenshot({ path: info.outputPath('cockpit.png') })
  await page.getByRole('button', { name: 'Exit cockpit' }).click(); await page.clock.runFor(100)
  await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  const beforeReload = await saved(); check(beforeReload)
  await page.reload()
  // Controlled clock has not advanced: reload/offline time cannot burn fuel.
  await expect(page.getByRole('button', { name: 'Pause [P]', exact: true })).toBeVisible()
  expect((await saved()).core).toEqual(beforeReload.core)
  await page.clock.runFor(1000)
  const restored = await saved(); check(restored)
  expect(restored.core.tick).toBeGreaterThan(beforeReload.core.tick)
  await page.screenshot({ path: info.outputPath('restored.png') })
  await info.attach('core-records.json', { body: JSON.stringify({ world, paused, hidden, cockpit, beforeReload, restored, backend }, null, 2), contentType: 'application/json' })
})
