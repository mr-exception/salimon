# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: items.spec.ts >> ITEM-1–6 fixture runs production item adapter, HUD, gathering, rejection and durable reload twice
- Location: tests/items.spec.ts:6:1

# Error details

```
Error: expect(locator).toHaveText(expected) failed

Locator:  locator('[data-inventory-weight]')
Expected: "2.25 / 10 kg"
Received: "2.5 / 10 kg"
Timeout:  10000ms

Call log:
  - Expect "toHaveText" locator('[data-inventory-weight]') with timeout 10000ms
  - waiting for locator('[data-inventory-weight]')
    24 × locator resolved to <p data-inventory-weight="true">…</p>
       - unexpected value "2.5 / 10 kg"

```

```yaml
- paragraph:
  - strong: "2.5"
  - text: / 10 kg
```

# Test source

```ts
  1  | import { test, expect } from '@playwright/test'
  2  | import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
  3  | import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
  4  | import { boot, walkTicks } from './worldJourney.ts'
  5  | 
  6  | test('ITEM-1–6 fixture runs production item adapter, HUD, gathering, rejection and durable reload twice', async ({ browser }, info) => {
  7  |   const originals: Record<string, Buffer> = {}, records = []
  8  |   for (let replay = 0; replay < 2; replay++) {
  9  |     const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
  10 |     const page = await context.newPage(), errors: string[] = []
  11 |     page.on('pageerror', e => errors.push(e.message))
  12 |     try {
  13 |       await boot(page, 'tests/items.html')
  14 |       const capture = async (name: string) => {
  15 |         await page.clock.runFor(5000)
  16 |         const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
  17 |         if (!replay) originals[name] = png
  18 |         else { const result = compareImages(originals[name]!, png); expect(result.pass, name).toBe(true); await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: result.pass, changedPixels: result.changedPixels, rawChangedPixels: result.rawChangedPixels, policy: POLICY }), contentType: 'application/json' }) }
  19 |       }
  20 |       await page.getByRole('button', { name: 'Field inventory' }).click()
  21 |       await expect(page.locator('[data-empty-items]')).toHaveText('No items carried.')
  22 |       await expect(page.locator('[data-inventory-weight]')).toHaveText('0 / 10 kg')
  23 |       await capture('items-empty')
  24 |       await page.keyboard.press('Escape'); await page.locator('body').click({position:{x:640,y:700}})
  25 |       await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
  26 |       await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  27 |       await page.keyboard.press('e'); await walkTicks(page, [], 60); await page.keyboard.press('Escape')
  28 |       await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
  29 |       await page.getByRole('button', { name: 'Field inventory' }).click()
  30 |       await capture('items-raw')
  31 |       await page.getByRole('button', { name: 'Add two singles' }).click()
  32 |       await page.getByRole('button', { name: 'Add five stacked' }).click()
  33 |       await page.clock.runFor(32)
  34 |       await expect(page.locator('[data-inventory-weight]')).toHaveText('2.5 / 10 kg')
  35 |       await expect(page.locator('[data-inventory-slots]')).toHaveText('5 / 12 slots')
  36 |       await expect(page.locator('[data-item="fixture-single"]')).toHaveText('Single fixture × 20.25 kg · 2 slots')
  37 |       await expect(page.locator('[data-item="fixture-stack"]')).toHaveText('Stack fixture × 51.25 kg · 2 slots')
  38 |       await capture('items-mixed')
  39 |       const raw = await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
  40 |       await page.getByRole('button', { name: 'Try overweight' }).click(); await page.clock.runFor(32)
  41 |       await expect(page.getByRole('status')).toContainText('Inventory exceeds its capacity')
  42 |       expect(await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)).toBe(raw)
  43 |       await capture('items-overweight')
  44 |       await page.reload(); await page.getByRole('button', { name: 'Field inventory' }).click(); await expect(page.locator('[data-inventory-weight]')).toHaveText('2.5 / 10 kg')
  45 |       await page.getByRole('button', { name: 'Remove one stacked' }).click(); await page.clock.runFor(32)
> 46 |       await expect(page.locator('[data-inventory-weight]')).toHaveText('2.25 / 10 kg')
     |                                                             ^ Error: expect(locator).toHaveText(expected) failed
  47 |       await expect(page.locator('[data-inventory-slots]')).toHaveText('4 / 12 slots')
  48 |       const after = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  49 |       expect(after.inventory.items).toEqual({ 'fixture-single': 2, 'fixture-stack': 4 })
  50 |       expect(after.sources['source:landing:mineral:01'].remainingGrams).toBe(59000)
  51 |       await page.setViewportSize({ width: 390, height: 844 }); await page.clock.runFor(32)
  52 |       const hud = page.getByLabel('Resource inventory'), bounds = await hud.boundingBox()
  53 |       expect(bounds!.x).toBeGreaterThanOrEqual(0); expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(390)
  54 |       await expect(page.locator('[data-item="fixture-stack"]')).toBeVisible()
  55 |       await page.screenshot({ path: info.outputPath(`items-narrow-${replay}.png`), fullPage: true })
  56 |       expect(errors).toEqual([]); records.push({ replay, browser: browser.version(), before: JSON.parse(raw!), after, errors })
  57 |     } finally { await context.close() }
  58 |   }
  59 |   const negative = compareImages(originals['items-raw']!, originals['items-mixed']!)
  60 |   expect(negative.pass).toBe(false)
  61 |   await info.attach('items-journeys.json', { body: JSON.stringify({ records, negative: { pass: negative.pass, changedPixels: negative.changedPixels }, policy: POLICY }, null, 2), contentType: 'application/json' })
  62 | })
  63 | test('ordinary built game has no item fixture, grants or query backdoor', async ({ page }) => {
  64 |   await boot(page, '?itemsFixture=1', 'http://127.0.0.1:4175')
  65 |   await expect(page.locator('[data-empty-items]')).toHaveText('No items carried.')
  66 |   await expect(page.locator('[data-item]')).toHaveCount(0)
  67 |   await expect(page.getByRole('button', { name: 'Add two singles' })).toHaveCount(0)
  68 |   const fixture = await page.request.get('http://127.0.0.1:4175/tests/items.html')
  69 |   expect(await fixture.text()).not.toContain('items.tsx')
  70 | })
  71 | 
  72 | test('HUD full-by-slots uses authoritative count items and never advertises Gather', async ({ page }, info) => {
  73 |   await boot(page, 'tests/items.html')
  74 |   for (let i = 0; i < 6; i++) await page.getByRole('button', { name: 'Add two singles' }).click()
  75 |   await page.locator('body').click({position:{x:640,y:700}})
  76 |   await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
  77 |   await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  78 |   await expect(page.locator('[data-inventory-weight]')).toHaveText('1.5 / 10 kg')
  79 |   await expect(page.locator('[data-inventory-slots]')).toHaveText('12 / 12 slots')
  80 |   await expect(page.getByRole('button', { name: 'Gather', exact: true })).toHaveCount(0)
  81 |   await expect(page.locator('.source-summary').first()).toHaveText('Inventory full: slot limit.')
  82 |   await page.keyboard.press('e')
  83 |   await expect(page.getByRole('status')).toContainText('slot limit')
  84 |   await expect(page.getByLabel('Resource inventory')).toHaveAttribute('data-gathering', '')
  85 |   await page.screenshot({path:info.outputPath('slot-limit.png')})
  86 | })
  87 | 
```