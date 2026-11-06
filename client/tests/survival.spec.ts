import { test, expect } from '@playwright/test'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { freshSurvival, METER } from '../src/survival.ts'
import { boot, walkTicks, walkTo, position } from './worldJourney'

// Original captures and raw state are explicit below; avoid synchronizing every
// software WebGL frame for a trace filmstrip. Failure action traces remain.
test.use({ trace: { mode: 'retain-on-failure', snapshots: false, screenshots: false, sources: true } })

test('SURV built active/pause/reload, outdoor/interior/navigation-return and terminal reset', async ({ page }, info) => {
  await boot(page, '', 'http://127.0.0.1:4175')
  const hud = page.getByLabel('Survival condition')
  await expect(page.getByRole('meter', { name: 'Health', exact: true })).toHaveAttribute('value', '200')
  const capture = async (name: string) => {
    await expect.poll(async () => { await page.clock.runFor(32); return page.locator('.top-down-scene').getAttribute('data-render-ready') }).toBe('true')
    await page.evaluate(async () => {
      await Promise.all(Array.from(document.querySelectorAll('svg image')).map(async element => {
        const bitmap = new Image(); bitmap.src = element.getAttribute('href')!; await bitmap.decode()
      }))
    })
    await page.clock.runFor(64)
    await page.screenshot({ path: info.outputPath(`${name}.png`), fullPage: true })
    const box = await hud.boundingBox(); expect(box!.width).toBeGreaterThan(150); expect(box!.x).toBeGreaterThanOrEqual(0)
  }
  await walkTicks(page, [], 60)
  const save = () => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  const active = await save(); expect(active.survival.activeTicks).toBeGreaterThanOrEqual(60)
  expect(active.survival.hunger).toBe(METER - active.survival.activeTicks * 36)
  await capture('outdoor')
  await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  const paused = await save(), before = await position(page)
  await page.keyboard.down('w'); await page.clock.runFor(3000); await page.keyboard.up('w')
  expect(await position(page)).toEqual(before); expect(await save()).toEqual(paused)
  await expect(hud).toContainText('active time stopped')
  await page.getByRole('button', { name: 'Resume [P]' }).click()
  await walkTicks(page, [], 60)
  expect((await save()).survival.activeTicks).toBeGreaterThan(paused.survival.activeTicks)
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: true }); document.dispatchEvent(new Event('visibilitychange')) })
  const background = await save()
  await page.clock.runFor(2000); expect(await save()).toEqual(background)
  await page.evaluate(() => { Object.defineProperty(document, 'hidden', { configurable: true, value: false }); document.dispatchEvent(new Event('visibilitychange')) })
  await page.reload(); await expect(hud).toContainText('200 / 200 HP')
  expect((await save()).survival.hunger).toBeLessThanOrEqual(paused.survival.hunger)
  // The fixture changes only saved meter conditions, never a runtime-only test API.
  const setCondition = async (condition: object) => {
    await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
    await page.evaluate(({ key, condition }) => {
      const saved = JSON.parse(localStorage.getItem(key)!); saved.survival = condition
      localStorage.setItem(key, JSON.stringify(saved))
    }, { key: RESOURCE_SAVE_KEY, condition })
    await page.reload(); await expect(hud).toBeVisible(); await page.clock.runFor(50)
  }
  await setCondition({ ...freshSurvival(Date.parse('2026-11-04T00:00:00Z') + 86_400_000), hunger: 0, health: 1, fatigue: METER / 10 })
  await expect(hud).toContainText('Low hunger'); await capture('low-condition')
  await page.clock.runFor(10_100)
  await expect(hud).toContainText('Run ended')
  const terminal = await save(); expect(terminal.survival.health).toBe(0)
  await expect(page.getByRole('button', { name: 'Equip gathering tool' })).toBeDisabled()
  const deadPosition = await position(page)
  await page.keyboard.down('w'); await page.clock.runFor(500); await page.keyboard.up('w')
  expect(await position(page)).toEqual(deadPosition)
  await capture('terminal')
  await page.reload(); await expect(hud).toContainText('Run ended')
  await page.getByRole('button', { name: 'Start fresh character' }).click()
  expect((await save()).sources).toEqual(terminal.sources)
  expect((await save()).inventory).toEqual(terminal.inventory)
  await expect(hud).toContainText('200 / 200 HP')
  // Real production keyboard journey through the Scout and radar WebGL adapter.
  for (const p of [[-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
  for (const p of [[0, 7], [0, 2]]) await walkTo(page, p, 'ship:scout', true)
  await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
  await capture('interior')
  for (const p of [[0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
  await page.keyboard.press('e'); await page.clock.runFor(100)
  await expect(page.getByRole('region', { name: 'Panoramic navigation', exact: true })).toBeVisible()
  await expect(hud).toBeVisible()
  for (let elapsed = 0; elapsed < 1000; elapsed += 200) await page.clock.fastForward(200)
  const radarActive = (await save()).survival.activeTicks
  await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  const shipPaused = await page.locator('main').getAttribute('data-ship')
  const viewPaused = await page.locator('.panoramic').getAttribute('data-observation')
  for (let elapsed = 0; elapsed < 1000; elapsed += 200) await page.clock.fastForward(200)
  expect(await page.locator('main').getAttribute('data-ship')).toBe(shipPaused)
  expect(await page.locator('.panoramic').getAttribute('data-observation')).toBe(viewPaused)
  expect((await save()).survival.activeTicks).toBeGreaterThanOrEqual(radarActive)
  await page.getByRole('button', { name: 'Resume [P]' }).click()
  for (let elapsed = 0; elapsed < 1100; elapsed += 100) await page.clock.fastForward(100)
  expect((await save()).survival.activeTicks).toBeGreaterThan(radarActive)
  const backend = await page.locator('canvas').evaluate(c => { const gl = (c as HTMLCanvasElement).getContext('webgl2')!; const ext = gl.getExtension('WEBGL_debug_renderer_info')!; return gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) })
  await page.getByRole('button', { name: 'Exit cockpit' }).click()
  await capture('navigation-return')
  await info.attach('survival-records.json', { body: JSON.stringify({ active, paused, terminal, final: await save(), backend, input: 'Playwright synthetic keyboard/mouse; saved-state low-condition fixture' }, null, 2), contentType: 'application/json' })
})

test('SURV-799 approved rates and partial interruption/reload preserve exact grams', async ({ browser }, info) => {
  const records = []
  for (const [hunger, fatigue, rate, totalTicks] of [[10, 100, .75, 80], [100, 20, .75, 80], [10, 20, .5625, 107]]) {
    const context = await browser.newContext(), page = await context.newPage()
    await boot(page, '', 'http://127.0.0.1:4175')
    await walkTo(page, [-7.1, 0], 'planet:fixture', true)
    await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
    await page.evaluate(({ key, hunger, fatigue, meter }) => {
      const saved = JSON.parse(localStorage.getItem(key)!)
      saved.survival = { hunger: hunger * meter / 100, fatigue: fatigue * meter / 100, health: 200, activeTicks: 0, starvationTicks: 0, recoveryTicks: 0, offlineAtMs: Date.now() + 86400000 }
      localStorage.setItem(key, JSON.stringify(saved))
    }, { key: RESOURCE_SAVE_KEY, hunger, fatigue, meter: METER })
    await page.reload(); await page.clock.runFor(100)
    await page.getByRole('button', { name: 'Equip gathering tool' }).click(); await page.keyboard.press('e')
    await expect(page.getByLabel('Resource inventory')).toContainText(`Mining rate: ${rate} kg/s`)
    const hud = page.getByLabel('Survival condition')
    await expect(hud).toContainText(hunger < 20 ? 'Movement and mining reduced' : 'mining reduced at 20')
    await walkTicks(page, [], 37)
    await page.screenshot({ path: info.outputPath(`active-mining-${hunger}-${fatigue}.png`), fullPage: true })
    await page.keyboard.press('Escape')
    const read = () => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
    const partial = await read()
    expect(partial.miningWork['source:landing:mineral:01']).toBe(37 * 16 * rate)
    expect(partial.inventory.quantities).toEqual({})
    await page.reload(); await page.clock.runFor(100)
    expect((await read()).miningWork).toEqual(partial.miningWork)
    await page.getByRole('button', { name: 'Equip gathering tool' }).click(); await page.keyboard.press('e')
    await walkTicks(page, [], totalTicks - 38)
    expect((await read()).inventory.quantities).toEqual({})
    await walkTicks(page, [], 1); await page.keyboard.press('Escape')
    const extracted = await read()
    expect(extracted.inventory.quantities).toEqual({ mineral: 1000 })
    expect(extracted.sources['source:landing:mineral:01'].remainingGrams).toBe(59000)
    expect(extracted.miningWork['source:landing:mineral:01'] ?? 0).toBe(totalTicks * 16 * rate - 960)
    await page.evaluate(async () => {
      await Promise.all(Array.from(document.querySelectorAll('svg image')).map(async element => {
        const bitmap = new Image(); bitmap.src = element.getAttribute('href')!; await bitmap.decode()
      }))
    })
    await page.screenshot({ path: info.outputPath(`mining-${hunger}-${fatigue}.png`), fullPage: true })
    const box = await hud.boundingBox(); expect(box!.width).toBeGreaterThan(150)
    await page.reload(); await page.clock.runFor(100)
    const restored = await read()
    expect(restored.inventory).toEqual(extracted.inventory); expect(restored.sources).toEqual(extracted.sources)
    expect(restored.miningWork).toEqual(extracted.miningWork)
    records.push({ hunger, fatigue, rate, totalTicks, partial, extracted, restored, browser: browser.version(), input: 'synthetic keyboard/mouse, controlled clock; saved meter fixture only' })
    await context.close()
  }
  await info.attach('mining-policy-records.json', { body: JSON.stringify(records, null, 2), contentType: 'application/json' })
})
