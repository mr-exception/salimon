# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: perspective.spec.ts >> SENSOR-1 scanner authority, hidden/browser contacts, group pagination and eviction
- Location: tests/perspective.spec.ts:10:1

# Error details

```
Test timeout of 180000ms exceeded.
```

```
Error: locator.click: Test timeout of 180000ms exceeded.
Call log:
  - waiting for getByRole('region', { name: 'Detected contacts' }).locator('li button').last()
    - locator resolved to <button aria-pressed="false">Detected dense-17</button>
  - attempting click action
    - waiting for element to be visible, enabled and stable

```

# Page snapshot

```yaml
- generic [ref=e2]:
  - generic [ref=e3]:
    - generic:
      - button "Sensor contact group of 18" [ref=e7]: "18"
      - button "Sensor contact Detected planet" [ref=e8]: +
      - button "Sensor contact Detected tiny" [ref=e9]: +
      - region "Detected contacts" [ref=e10]:
        - generic [ref=e11]:
          - text: Search contacts
          - textbox "Search contacts" [ref=e12]
        - button "All contacts" [ref=e13]
        - list [ref=e14]:
          - listitem [ref=e15]:
            - button "Detected dense-16" [ref=e16]
          - listitem [ref=e17]:
            - button "Detected dense-17" [ref=e18]
        - button "Previous" [ref=e19]
        - generic [ref=e20]: 3/3
        - button "Next" [disabled] [ref=e21]
        - status [ref=e22]: "Selected: Detected dense-15"
        - button "Set direction" [ref=e23]
  - navigation [ref=e24]:
    - button "Tick" [ref=e25]
    - button "Pulse" [ref=e26]
    - button "Leave range" [ref=e27]
    - button "Telescope" [ref=e28]
    - button "Reset view" [ref=e29]
```

# Test source

```ts
  1   | import { test, expect, type Page } from '@playwright/test'
  2   | import { PNG } from 'pngjs'
  3   | type State = { tick: number; contacts: string[]; selected: string | null; actions: number; renderer: string; view: { fov: number; quaternion: number[] }; markers: { id: string; ids: string[]; x: number; y: number }[] }
  4   | const state = (page: Page) => page.evaluate(() => (window as unknown as { sensorSnapshot(): State }).sensorSnapshot())
  5   | const sceneMarkers = (page: Page) => page.getByRole('button', { name: /^Sensor contact / })
  6   | async function boot(page: Page, translated = false) {
  7   |  await page.goto(`/tests/perspective.html${translated ? '?translated' : ''}`)
  8   |  await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
  9   | }
  10  | test('SENSOR-1 scanner authority, hidden/browser contacts, group pagination and eviction', async ({ page }, info) => {
  11  |  const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  12  |  await boot(page)
  13  |  expect((await state(page)).contacts).toEqual([])
  14  |  await expect(sceneMarkers(page)).toHaveCount(0)
  15  |  await expect(page.getByText('Detected planet', { exact: true })).toHaveCount(0)
  16  |  // The visible optical globe has no discovery target before the pulse.
  17  |  await page.locator('canvas').click({ position: { x: 480, y: 300 } })
  18  |  expect((await state(page)).selected).toBeNull()
  19  |  const before = await page.screenshot()
  20  |  const png = PNG.sync.read(before), center = (300 * png.width + 480) * 4
  21  |  expect(png.data[center] + png.data[center + 1] + png.data[center + 2]).toBeGreaterThan(10)
  22  |  await info.attach('before-pulse.png', { body: before, contentType: 'image/png' })
  23  |  await page.getByRole('button', { name: 'Tick', exact: true }).click()
  24  |  await expect(sceneMarkers(page)).toHaveCount(0)
  25  |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  26  |  await expect(sceneMarkers(page)).toHaveCount(3)
  27  |  const scanned = await state(page)
  28  |  expect(scanned.contacts).toHaveLength(23)
  29  |  expect(scanned.markers.flatMap(m => m.ids).sort()).toEqual(['planet', 'tiny', ...Array.from({ length: 18 }, (_, i) => `dense-${String(i).padStart(2, '0')}`)].sort())
  30  |  await info.attach('scanned.png', { body: await page.screenshot(), contentType: 'image/png' })
  31  |  await page.getByRole('button', { name: 'Sensor contact Detected tiny', exact: true }).click()
  32  |  await expect(page.getByRole('status')).toHaveText('Selected: Detected tiny')
  33  |  for (const id of ['hidden', 'back', 'outside']) {
  34  |   await page.getByRole('textbox').fill(id)
  35  |   await page.getByRole('button', { name: `Detected ${id}`, exact: true }).click()
  36  |   await expect(page.getByRole('status')).toHaveText(`Selected: Detected ${id}`)
  37  |  }
  38  |  await page.getByRole('button', { name: 'Set direction', exact: true }).click()
  39  |  await expect.poll(async () => (await state(page)).actions).toBe(1)
  40  |  await page.getByRole('textbox').fill('')
  41  |  await page.getByRole('button', { name: 'Sensor contact group of 18', exact: true }).click()
  42  |  const ids: string[] = []
  43  |  for (let i = 0; i < 3; i++) {
  44  |   const buttons = page.getByRole('region', { name: 'Detected contacts' }).locator('li button')
  45  |   ids.push(...await buttons.allTextContents())
> 46  |   await buttons.last().click()
      |                        ^ Error: locator.click: Test timeout of 180000ms exceeded.
  47  |   if (i < 2) await page.getByRole('button', { name: 'Next', exact: true }).click()
  48  |  }
  49  |  expect(ids).toHaveLength(18); expect(new Set(ids).size).toBe(18)
  50  |  await expect(page.getByRole('button', { name: 'Next', exact: true })).toBeDisabled()
  51  |  await page.getByRole('button', { name: 'All contacts', exact: true }).click()
  52  |  await page.getByRole('textbox').fill('tiny')
  53  |  await page.getByRole('button', { name: 'Detected tiny', exact: true }).click()
  54  |  await page.getByRole('button', { name: 'Telescope', exact: true }).click()
  55  |  await expect(sceneMarkers(page)).toHaveCount(1)
  56  |  await expect(page.getByRole('status')).toHaveText('Selected: Detected tiny')
  57  |  await page.getByRole('button', { name: 'Leave range', exact: true }).click()
  58  |  await expect(page.getByRole('status')).toHaveText('No selection')
  59  |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toHaveCount(0)
  60  |  await expect(sceneMarkers(page)).toHaveCount(0)
  61  |  expect((await state(page)).selected).toBeNull()
  62  |  expect((await state(page)).contacts).toEqual([])
  63  |  expect(errors).toEqual([])
  64  |  await info.attach('measurements.json', { body: JSON.stringify({ scanned, groupIds: ids, evicted: await state(page), errors }, null, 2), contentType: 'application/json' })
  65  | })
  66  | test('SENSOR-2 repeat/translated pixels, telescope motion and stale group suppression', async ({ page }, info) => {
  67  |  await boot(page)
  68  |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  69  |  await expect(sceneMarkers(page)).toHaveCount(3)
  70  |  // Blur Pulse so focus decorations cannot differ between captures.
  71  |  await page.locator('canvas').click({ position: { x: 50, y: 550 } })
  72  |  const first = await page.screenshot(), initial = await state(page)
  73  |  expect((await page.screenshot()).equals(first)).toBe(true)
  74  |  await boot(page, true)
  75  |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  76  |  await expect(sceneMarkers(page)).toHaveCount(3)
  77  |  await page.locator('canvas').click({ position: { x: 50, y: 550 } })
  78  |  expect((await page.screenshot()).equals(first)).toBe(true)
  79  |  expect((await state(page)).markers).toEqual(initial.markers)
  80  |  await page.getByRole('button', { name: 'Sensor contact group of 18', exact: true }).click()
  81  |  await page.getByRole('button', { name: 'Telescope', exact: true }).click()
  82  |  await expect(page.getByText('Group no longer visible')).toBeVisible()
  83  |  await expect(page.getByRole('region', { name: 'Detected contacts' }).locator('li button')).toHaveCount(0)
  84  |  await page.getByRole('button', { name: 'All contacts', exact: true }).click()
  85  |  await page.getByRole('textbox').fill('back')
  86  |  await page.keyboard.press('KeyD')
  87  |  const focused = (await state(page)).view
  88  |  await page.keyboard.down('KeyD'); await page.waitForTimeout(100); await page.keyboard.up('KeyD')
  89  |  expect((await state(page)).view).toEqual(focused)
  90  |  await page.locator('canvas').click({ position: { x: 50, y: 550 } })
  91  |  await page.keyboard.down('KeyD')
  92  |  const motion = await page.evaluate(async () => {
  93  |   const samples: State[] = []
  94  |   for (let i = 0; i < 6; i++) { await new Promise(requestAnimationFrame); samples.push((window as unknown as { sensorSnapshot(): State }).sensorSnapshot()) }
  95  |   return samples
  96  |  })
  97  |  await page.keyboard.up('KeyD')
  98  |  expect(motion.every(s => s.markers.length === 1 && s.markers.every(m => Number.isFinite(m.x) && Number.isFinite(m.y)))).toBe(true)
  99  |  await expect.poll(async () => (await state(page)).view.quaternion).not.toEqual(focused.quaternion)
  100 |  expect((await state(page)).contacts).toEqual(initial.contacts)
  101 |  expect((await state(page)).tick).toBe(300)
  102 |  await info.attach('telescope.png', { body: await page.screenshot(), contentType: 'image/png' })
  103 |  await info.attach('measurements.json', { body: JSON.stringify({ initial, motion, moved: await state(page), repeat: 'byte-identical', translated: 'byte-identical' }, null, 2), contentType: 'application/json' })
  104 | })
  105 |
```
