import { test, expect } from '@playwright/test'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { boot, walkTicks } from './worldJourney.ts'

test('ITEM-1–6 fixture runs production item adapter, HUD, gathering, rejection and durable reload twice', async ({ browser }, info) => {
  const originals: Record<string, Buffer> = {}, records = []
  for (let replay = 0; replay < 2; replay++) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
    const page = await context.newPage(), errors: string[] = []
    page.on('pageerror', e => errors.push(e.message))
    try {
      await boot(page, 'tests/items.html')
      const capture = async (name: string) => {
        await page.clock.runFor(5000)
        const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
        if (!replay) originals[name] = png
        else { const result = compareImages(originals[name]!, png); expect(result.pass, name).toBe(true); await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: result.pass, changedPixels: result.changedPixels, rawChangedPixels: result.rawChangedPixels, policy: POLICY }), contentType: 'application/json' }) }
      }
      await expect(page.locator('[data-empty-items]')).toHaveText('No items carried.')
      await expect(page.locator('[data-inventory-weight]')).toHaveText('0 / 10 kg')
      await capture('items-empty')
      await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
      await page.getByRole('button', { name: 'Equip gathering tool' }).click()
      await page.keyboard.press('e'); await walkTicks(page, [], 60); await page.keyboard.press('Escape')
      await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
      await capture('items-raw')
      await page.getByRole('button', { name: 'Add two singles' }).click()
      await page.getByRole('button', { name: 'Add five stacked' }).click()
      await page.clock.runFor(32)
      await expect(page.locator('[data-inventory-weight]')).toHaveText('2.5 / 10 kg')
      await expect(page.locator('[data-inventory-slots]')).toHaveText('5 / 12 slots')
      await expect(page.locator('[data-item="fixture-single"]')).toHaveText('Single fixture × 20.25 kg · 2 slots')
      await expect(page.locator('[data-item="fixture-stack"]')).toHaveText('Stack fixture × 51.25 kg · 2 slots')
      await capture('items-mixed')
      const raw = await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
      await page.getByRole('button', { name: 'Try overweight' }).click(); await page.clock.runFor(32)
      await expect(page.getByRole('status')).toContainText('Inventory exceeds its capacity')
      expect(await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)).toBe(raw)
      await capture('items-overweight')
      await page.reload(); await expect(page.locator('[data-inventory-weight]')).toHaveText('2.5 / 10 kg')
      await page.getByRole('button', { name: 'Remove one stacked' }).click(); await page.clock.runFor(32)
      await expect(page.locator('[data-inventory-weight]')).toHaveText('2.25 / 10 kg')
      await expect(page.locator('[data-inventory-slots]')).toHaveText('4 / 12 slots')
      const after = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
      expect(after.inventory.items).toEqual({ 'fixture-single': 2, 'fixture-stack': 4 })
      expect(after.sources['source:landing:mineral:01'].remainingGrams).toBe(59000)
      await page.setViewportSize({ width: 390, height: 844 }); await page.clock.runFor(32)
      const hud = page.getByLabel('Resource inventory'), bounds = await hud.boundingBox()
      expect(bounds!.x).toBeGreaterThanOrEqual(0); expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(390)
      await expect(page.locator('[data-item="fixture-stack"]')).toBeVisible()
      await page.screenshot({ path: info.outputPath(`items-narrow-${replay}.png`), fullPage: true })
      expect(errors).toEqual([]); records.push({ replay, browser: browser.version(), before: JSON.parse(raw!), after, errors })
    } finally { await context.close() }
  }
  const negative = compareImages(originals['items-raw']!, originals['items-mixed']!)
  expect(negative.pass).toBe(false)
  await info.attach('items-journeys.json', { body: JSON.stringify({ records, negative: { pass: negative.pass, changedPixels: negative.changedPixels }, policy: POLICY }, null, 2), contentType: 'application/json' })
})
test('ordinary built game has no item fixture, grants or query backdoor', async ({ page }) => {
  await boot(page, '?itemsFixture=1', 'http://127.0.0.1:4175')
  await expect(page.locator('[data-empty-items]')).toHaveText('No items carried.')
  await expect(page.locator('[data-item]')).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Add two singles' })).toHaveCount(0)
  const fixture = await page.request.get('http://127.0.0.1:4175/tests/items.html')
  expect(await fixture.text()).not.toContain('items.tsx')
})
