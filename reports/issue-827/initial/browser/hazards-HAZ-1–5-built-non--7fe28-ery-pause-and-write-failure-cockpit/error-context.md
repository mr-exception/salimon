# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: hazards.spec.ts >> HAZ-1–5 built non-shipping fixture uses production clock/save, exact reload/recovery, pause and write failure
- Location: tests/hazards.spec.ts:6:1

# Error details

```
Error: expect(locator).toHaveAttribute(expected) failed

Locator:  getByRole('meter', { name: 'Health', exact: true })
Expected: "188.60333333333332"
Received: "189"
Timeout:  10000ms

Call log:
  - Expect "toHaveAttribute" getByRole('meter', { name: 'Health', exact: true }) with timeout 10000ms
  - waiting for getByRole('meter', { name: 'Health', exact: true })
    23 × locator resolved to <meter min="0" max="200" value="189" aria-label="Health" aria-valuetext="189 / 200 HP"></meter>
       - unexpected value "189"

```

```yaml
- meter "Health"
```

# Test source

```ts
  1  | import { test, expect } from '@playwright/test'
  2  | import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
  3  | import { decode, encode, sub, mul, decimal } from '../src/exact.ts'
  4  | import { boot, walkTicks } from './worldJourney.ts'
  5  | 
  6  | test('HAZ-1–5 built non-shipping fixture uses production clock/save, exact reload/recovery, pause and write failure', async ({ page, browser }, info) => {
  7  |   await boot(page, 'tests/hazards.html', 'http://127.0.0.1:4275')
  8  |   const save = () => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  9  |   await page.getByRole('button', { name: 'Declare test module' }).click()
  10 |   const initial = await save()
  11 |   expect(initial.hazards.modules[0].remaining).toBe('1000/1')
  12 |   await page.getByRole('button', { name: 'Enter test hazards' }).click()
  13 |   await walkTicks(page, [], 17)
  14 |   await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  15 |   const partial = await save(), ticks = partial.survival.activeTicks - initial.survival.activeTicks
  16 |   // No real timers advance between actions; all intervals are controlled RAF ticks.
  17 |   expect(partial.hazards.exposure.radiation).toBe(encode(mul(decimal(ticks), [13n, 600n])))
  18 |   expect(partial.hazards.exposure.poison).toBe(encode(mul(decimal(ticks), [1n, 480n])))
  19 |   expect(partial.hazards.modules[0].remaining).toBe(encode(sub(decimal(1000), mul(decimal(ticks), [1n, 30n]))))
  20 |   expect(partial.survival.healthExact).toBe(encode(sub(decimal(200), mul(decimal(ticks), [1n, 600n]))))
  21 |   await page.clock.runFor(2000); expect(await save()).toEqual(partial)
  22 |   await page.reload()
  23 |   // Sources are deliberately not persisted as real planetary sampling. Stop immediately to inspect restore.
  24 |   await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  25 |   const restored = await save()
  26 |   expect(restored.survival.healthExact).toBe(partial.survival.healthExact)
  27 |   expect(restored.hazards.modules).toEqual(partial.hazards.modules)
  28 |   expect(Number(decode(restored.hazards.exposure.radiation)[0])).toBeGreaterThanOrEqual(0)
  29 |   await page.getByRole('button', { name: 'Enter test hazards' }).click()
  30 |   await page.getByRole('button', { name: 'Resume [P]', exact: true }).click()
  31 |   await walkTicks(page, [], 4800) // 80s: threshold crossed; recovery heals one HP at 60s.
  32 |   await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  33 |   const full = await save()
  34 |   expect(full.hazards.exposure.radiation).toBe('100/1')
  35 |   expect(full.survival.health).toBeLessThan(192)
  36 |   await page.getByRole('button', { name: 'Exit test hazards' }).click()
  37 |   await page.getByRole('button', { name: 'Resume [P]', exact: true }).click()
  38 |   await walkTicks(page, [], 60)
  39 |   await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
  40 |   const exited = await save()
  41 |   expect(exited.hazards.exposure.radiation).toBe('93/1')
  42 |   expect(exited.hazards.modules).toEqual(full.hazards.modules)
  43 |   expect(exited.survival.healthExact).toBe(full.survival.healthExact)
  44 |   await page.getByRole('button', { name: 'Enter test hazards' }).click()
  45 |   await page.getByRole('button', { name: 'Fail test saves' }).click()
  46 |   const beforeFailure = await save()
  47 |   await page.getByRole('button', { name: 'Resume [P]', exact: true }).click()
  48 |   await page.clock.runFor(1000)
  49 |   await expect(page.getByLabel('Survival condition')).toContainText('Saving failed')
  50 |   expect(await save()).toEqual(beforeFailure)
> 51 |   await page.reload(); await expect(page.getByRole('meter', { name: 'Health', exact: true })).toHaveAttribute('value', String(beforeFailure.survival.health))
     |                                                                                               ^ Error: expect(locator).toHaveAttribute(expected) failed
  52 |   await expect(page.getByLabel('Survival condition')).toContainText(`${Math.ceil(beforeFailure.survival.health)} / 200 HP`)
  53 |   await expect.poll(async () => { await page.clock.runFor(32); return page.locator('.top-down-scene').getAttribute('data-render-ready') }).toBe('true')
  54 |   await info.attach('hazard-records.json', { body: JSON.stringify({ browser: browser.version(), initial, partial, restored, full, exited, beforeFailure, final: await save(), input: 'Playwright synthetic buttons and controlled clock; separately built test-only sources/modules' }, null, 2), contentType: 'application/json' })
  55 | })
  56 | 
  57 | test('normal built game exposes no hazard fixture, source sampling or module grants', async ({ page }) => {
  58 |   await boot(page, '?hazardsFixture=1', 'http://127.0.0.1:4175')
  59 |   await walkTicks(page, [], 60)
  60 |   await expect(page.getByRole('button', { name: 'Declare test module' })).toHaveCount(0)
  61 |   await expect(page.getByRole('meter', { name: 'Health', exact: true })).toHaveAttribute('value', '200')
  62 |   const saved = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
  63 |   expect(saved.hazards).toEqual({ version: 1, exposure: { radiation: '0/1', poison: '0/1' }, modules: [] })
  64 |   expect(await (await page.request.get('http://127.0.0.1:4175/tests/hazards.html')).text()).not.toContain('hazards.tsx')
  65 | })
  66 | 
```