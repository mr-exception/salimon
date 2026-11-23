import { test, expect, type Page } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { point2 } from '../src/topDownFrames.ts'
import { SCOUT_CORE_FUEL_ID as fuelId, CORE_CAPACITY_J } from '../src/scoutEnergyCore.ts'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { boot, position, walkTicks, walkTo } from './worldJourney'

const saved = (page: Page) => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
const panel = (page: Page) => page.getByRole('dialog', { name: 'Energy Core', exact: true })
const origin = 'http://127.0.0.1:4175'

function fixture(state: 'empty' | 'processing' | 'full' | 'outage') {
  const initial = createTopDownState('exterior', true)
  const world = { ...initial, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(.8, 1.8) } } : e) }
  let raw: string | null = null
  const session = new ResourceSession({ getItem: () => raw, setItem: (_, value) => { raw = value } }, world)
  expect(session.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 })).toBe(true)
  // Spend through the domain so schema-v16 non-Fabricator receipts agree.
  if (state === 'outage') session.advance({ ...session.world, tick: 1 }, { watts: session.snapshot.core.reserveJ60 })
  const s = JSON.parse(raw!)
  // Valid prior expeditions: conserve mineral and initial+converted energy.
  if (state === 'processing' || state === 'full') {
    const grams = state === 'full' ? 50000 : 1000, converted = state === 'full' ? 3000 : 0
    s.sources['source:landing:mineral:01'].remainingGrams -= grams + converted
    s.stores[fuelId].quantities = { mineral: grams }; s.convertedGrams = converted ? { mineral: converted } : {}
    if (state === 'full') s.core.reserveJ60 = CORE_CAPACITY_J * 60
  }
  return JSON.stringify(s)
}

async function seed(page: Page, raw: string) {
  await page.addInitScript(({ key, raw }) => { if (!localStorage.getItem(key)) localStorage.setItem(key, raw) }, { key: RESOURCE_SAVE_KEY, raw })
}
const material = (s: any) => ({ inventory: s.inventory, stores: s.stores, converted: s.convertedGrams })

test('CORE-UI-1 built gather, reachable Core deposit/withdraw, interruptions and durable reload', async ({ page }, info) => {
  const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  await boot(page, '', origin)
  await expect(panel(page)).toHaveCount(0)
  await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
  await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  await page.keyboard.press('e'); await walkTicks(page, [], 60); await page.keyboard.press('Escape')
  await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
  for (const p of [[-6, 3], [-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
  for (const p of [[0, 7], [0, 3], [.8, 1.8]]) await walkTo(page, p, 'ship:scout', true)
  await expect(page.locator('[data-prompt]')).toContainText('Core · Use [E]')
  await page.getByRole('button', { name: 'Use Energy Core' }).click()
  await expect(panel(page)).toBeVisible(); await expect(panel(page)).toBeFocused()
  const input = page.getByRole('textbox', { name: 'Amount in grams' })
  await input.fill('333'); await page.getByRole('button', { name: 'Review deposit' }).click()
  const draft = await saved(page); expect(draft.stores[fuelId].quantities).toEqual({})
  await page.getByRole('button', { name: 'Confirm deposit' }).press('Enter'); await page.keyboard.press('Enter')
  expect((await saved(page)).inventory.quantities.mineral).toBe(667)
  await page.clock.runFor(1000)
  await expect(page.getByRole('status', { name: 'Core operating status' })).toContainText('Processing')
  const beforeTick = await saved(page)
  await walkTicks(page, [], 1)
  const converting = await saved(page)
  await expect(page.locator('[data-core-generation]')).toHaveText(`Generation: ${converting.core.reserveJ60 - beforeTick.core.reserveJ60} W · last committed tick`)
  await expect(page.locator('[data-core-consumption]')).toHaveText('Consumption: 0 W · last committed tick')
  expect(converting.convertedGrams.mineral).toBeGreaterThan(0)
  await expect(page.locator('[data-core-reserve]')).toContainText(String(converting.core.reserveJ60 / 60))
  await input.fill('111'); await page.getByRole('button', { name: 'Review withdrawal' }).click()
  await page.clock.runFor(1000) // Conversion alone must not make confirmation unusable.
  await page.getByRole('button', { name: 'Confirm withdraw' }).click()
  expect((await saved(page)).inventory.quantities.mineral).toBe(778)
  await page.screenshot({ path: info.outputPath('core-transferred.png') })
  const committed = await saved(page)
  const check = async () => expect(material(await saved(page))).toEqual(material(committed))
  for (const value of ['99999', '0', '-1', '0.5', '1e3']) {
    await input.fill(value); await page.getByRole('button', { name: 'Review deposit' }).click()
    await expect(panel(page).locator('.container-state')).toContainText('rejected'); await check()
  }
  await input.fill('1'); await page.getByRole('button', { name: 'Review deposit' }).click()
  await page.getByRole('button', { name: 'Cancel transfer' }).click(); await check()
  await page.getByRole('button', { name: 'Review deposit' }).click(); await page.keyboard.press('Escape')
  await expect(panel(page)).toHaveCount(0); await check()
  await page.keyboard.press('e'); await page.getByRole('button', { name: 'Review deposit' }).click()
  await page.evaluate(() => window.dispatchEvent(new Event('blur')))
  await expect(panel(page)).toHaveCount(0); await check()
  await page.keyboard.press('e'); await input.fill('wasdep'); await page.keyboard.press('w')
  const before = await position(page); await walkTicks(page, [], 5); expect(await position(page)).toEqual(before)
  await page.keyboard.press('Escape')
  await walkTo(page, [0, 3], 'ship:scout', true)
  const beforeReload = await saved(page)
  await page.reload(); await expect(page.getByRole('button', { name: 'Pause [P]', exact: true })).toBeVisible()
  const restored = await saved(page)
  expect(material(restored)).toEqual(material(beforeReload)); expect(restored.core).toEqual(beforeReload.core)
  const fuel = restored.stores[fuelId].quantities.mineral ?? 0, converted = restored.convertedGrams.mineral ?? 0
  expect(restored.inventory.quantities.mineral + fuel + converted + restored.sources['source:landing:mineral:01'].remainingGrams).toBe(60000)
  expect(restored.core.reserveJ60 + restored.core.spentJ60).toBe((500000 + converted * 500) * 60)
  expect(errors).toEqual([])
  await info.attach('journey.json', { body: JSON.stringify({ draft, converting, committed, beforeReload, restored, errors, input: 'Playwright keyboard/mouse, production blur listener' }, null, 2), contentType: 'application/json' })
})

for (const [width, height] of [[960, 600], [1280, 720], [1920, 1080]]) {
  test(`CORE-UI-2 authoritative states, keyboard and visual comparisons ${width}`, async ({ page }, info) => {
    await page.setViewportSize({ width, height }); await seed(page, fixture('empty')); await boot(page, '', origin)
    const captures: Record<string, Buffer> = {}, results: any[] = []
    const capture = async (name: string) => {
      const dialog = panel(page); await dialog.focus(); await page.mouse.move(0, 0)
      await dialog.evaluate(el => { el.scrollTop = 0 })
      const original = await dialog.screenshot({ animations: 'disabled', path: info.outputPath(`${name}.png`) })
      const repeat = await dialog.screenshot({ animations: 'disabled', path: info.outputPath(`${name}-repeat.png`) })
      const result = compareImages(original, repeat)
      expect(result.pass).toBe(true); captures[name] = original
      await info.attach(`${name}-diff.png`, { body: result.diff, contentType: 'image/png' })
      if (await dialog.evaluate(el => el.scrollHeight > el.clientHeight)) {
        await dialog.evaluate(el => { el.scrollTop = el.scrollHeight })
        await dialog.screenshot({ path: info.outputPath(`${name}-controls.png`) })
      }
      results.push({ name, pass: result.pass, changedPixels: result.changedPixels, state: await saved(page) })
    }
    for (const state of ['empty', 'processing', 'full', 'outage'] as const) {
      if (state !== 'empty') { await page.evaluate(({ key, raw }) => localStorage.setItem(key, raw), { key: RESOURCE_SAVE_KEY, raw: fixture(state) }); await page.reload(); await page.clock.runFor(100) }
      await page.keyboard.press('e'); await expect(panel(page)).toBeVisible()
      const dialog = panel(page)
      await expect(dialog).toBeFocused()
      await expect(page.getByRole('status', { name: 'Core operating status' })).toContainText({ empty: 'Fuel empty', processing: 'Processing', full: 'Reserve full', outage: 'Outage' }[state])
      const box = (await dialog.boundingBox())!
      expect(box.x).toBeGreaterThanOrEqual(0); expect(box.y).toBeGreaterThanOrEqual(0)
      expect(box.x + box.width).toBeLessThanOrEqual(width); expect(box.y + box.height).toBeLessThanOrEqual(height)
      expect(await dialog.evaluate(el => el.scrollWidth > el.clientWidth)).toBe(false)
      expect(await dialog.evaluate(el => getComputedStyle(el).animationDuration)).toBe('0s')
      await page.keyboard.press('Shift+Tab'); await expect(dialog.getByRole('button', { name: 'Close Energy Core' })).toBeFocused()
      await page.keyboard.press('Tab'); await expect(dialog.getByRole('combobox')).toBeFocused()
      await capture(state)
      if (state === 'full') {
        await expect(page.getByRole('status', { name: 'Fuel warning' })).toContainText('full')
        const input = page.getByRole('textbox', { name: 'Amount in grams' })
        await input.fill('1'); await dialog.getByRole('button', { name: 'Review deposit' }).click()
        await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Fuel buffer overweight')
        await capture('rejected-full')
        await input.fill('333'); await dialog.getByRole('button', { name: 'Review withdrawal' }).click()
        await capture('pending')
        const before = await saved(page)
        await page.evaluate(key => localStorage.setItem(key, localStorage.getItem(key)! + ' '), RESOURCE_SAVE_KEY)
        const bytes = await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
        await dialog.getByRole('button', { name: 'Confirm withdraw' }).click()
        await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Saving failed')
        expect(await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)).toBe(bytes)
        expect((await saved(page)).inventory).toEqual(before.inventory)
        await expect(dialog.getByRole('button', { name: 'Review deposit' })).toBeDisabled()
        await capture('save-error')
      }
      await page.keyboard.press('Escape'); await expect(panel(page)).toHaveCount(0)
    }
    const wrong = compareImages(captures.empty!, captures.outage!); expect(wrong.pass).toBe(false)
    await info.attach('comparisons.json', { body: JSON.stringify({ viewport: { width, height }, browser: page.context().browser()!.version(), policy: POLICY, results, negative: { pass: wrong.pass, changedPixels: wrong.changedPixels }, baselineApproval: 'owner visual review pending' }, null, 2), contentType: 'application/json' })
  })
}
