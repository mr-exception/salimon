# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: perspective.spec.ts >> SENSOR-1 scanner authority, hidden/browser contacts, group pagination and eviction
- Location: tests/perspective.spec.ts:10:1

# Error details

```
Test timeout of 120000ms exceeded.
```

```
Error: locator.click: Test timeout of 120000ms exceeded.
Call log:
  - waiting for getByRole('button', { name: 'Detected hidden', exact: true })
    - locator resolved to <button aria-pressed="false">Detected hidden</button>
  - attempting click action
    - waiting for element to be visible, enabled and stable
    - element is visible, enabled and stable
    - scrolling into view if needed
    - done scrolling

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
          - textbox "Search contacts" [active] [ref=e12]: hidden
        - list [ref=e13]:
          - listitem [ref=e14]:
            - button "Detected hidden" [ref=e15]
        - button "Previous" [disabled] [ref=e16]
        - generic [ref=e17]: 1/1
        - button "Next" [disabled] [ref=e18]
        - status [ref=e19]: No selection
  - navigation [ref=e20]:
    - button "Tick" [ref=e21]
    - button "Pulse" [ref=e22]
    - button "Leave range" [ref=e23]
    - button "Telescope" [ref=e24]
    - button "Reset view" [ref=e25]
```

# Test source

```ts
  1  | import { test, expect, type Page } from '@playwright/test'
  2  | import { PNG } from 'pngjs'
  3  | type State = { tick: number; contacts: string[]; selected: string | null; actions: number; renderer: string; view: { fov: number; quaternion: number[] }; markers: { id: string; ids: string[]; x: number; y: number }[] }
  4  | const state = (page: Page) => page.evaluate(() => (window as unknown as { sensorSnapshot(): State }).sensorSnapshot())
  5  | const sceneMarkers = (page: Page) => page.getByRole('button', { name: /^Sensor contact / })
  6  | async function boot(page: Page, translated = false) {
  7  |  await page.goto(`/tests/perspective.html${translated ? '?translated' : ''}`)
  8  |  await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
  9  | }
  10 | test('SENSOR-1 scanner authority, hidden/browser contacts, group pagination and eviction', async ({ page }, info) => {
  11 |  const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  12 |  await boot(page)
  13 |  expect((await state(page)).contacts).toEqual([])
  14 |  await expect(sceneMarkers(page)).toHaveCount(0)
  15 |  await expect(page.getByText('Detected planet', { exact: true })).toHaveCount(0)
  16 |  // The visible optical globe has no discovery target before the pulse.
  17 |  await page.locator('canvas').click({ position: { x: 640, y: 400 } })
  18 |  expect((await state(page)).selected).toBeNull()
  19 |  const before = await page.screenshot()
  20 |  const png = PNG.sync.read(before), center = (400 * png.width + 640) * 4
  21 |  expect(png.data[center] + png.data[center + 1] + png.data[center + 2]).toBeGreaterThan(10)
  22 |  await info.attach('before-pulse.png', { body: before, contentType: 'image/png' })
  23 |  await page.getByRole('button', { name: 'Tick', exact: true }).click()
  24 |  await expect(sceneMarkers(page)).toHaveCount(0)
  25 |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  26 |  await expect(sceneMarkers(page)).toHaveCount(3)
  27 |  const scanned = await state(page)
  28 |  expect(scanned.contacts).toHaveLength(23)
  29 |  expect(scanned.markers.flatMap(m => m.ids).sort()).toEqual(['planet', 'tiny', ...Array.from({ length: 18 }, (_, i) => `dense-${String(i).padStart(2, '0')}`)].sort())
  30 |  await info.attach('scanned.png', { body: await page.screenshot(), contentType: 'image/png' })
  31 |  for (const id of ['hidden', 'back', 'outside']) {
  32 |   await page.getByRole('textbox').fill(id)
> 33 |   await page.getByRole('button', { name: `Detected ${id}`, exact: true }).click()
     |                                                                           ^ Error: locator.click: Test timeout of 120000ms exceeded.
  34 |   await expect(page.getByRole('status')).toHaveText(`Selected: Detected ${id}`)
  35 |  }
  36 |  await page.getByRole('button', { name: 'Set direction', exact: true }).click()
  37 |  await expect.poll(async () => (await state(page)).actions).toBe(1)
  38 |  await page.getByRole('textbox').fill('')
  39 |  await page.getByRole('button', { name: 'Sensor contact group of 18', exact: true }).click()
  40 |  const ids: string[] = []
  41 |  for (let i = 0; i < 3; i++) {
  42 |   const buttons = page.getByRole('region', { name: 'Detected contacts' }).locator('li button')
  43 |   ids.push(...await buttons.allTextContents())
  44 |   await buttons.last().click()
  45 |   if (i < 2) await page.getByRole('button', { name: 'Next', exact: true }).click()
  46 |  }
  47 |  expect(ids).toHaveLength(18); expect(new Set(ids).size).toBe(18)
  48 |  await expect(page.getByRole('button', { name: 'Next', exact: true })).toBeDisabled()
  49 |  await page.getByRole('button', { name: 'All contacts', exact: true }).click()
  50 |  await page.getByRole('textbox').fill('tiny')
  51 |  await page.getByRole('button', { name: 'Detected tiny', exact: true }).click()
  52 |  await page.getByRole('button', { name: 'Telescope', exact: true }).click()
  53 |  await expect(sceneMarkers(page)).toHaveCount(1)
  54 |  await expect(page.getByRole('status')).toHaveText('Selected: Detected tiny')
  55 |  await page.getByRole('button', { name: 'Leave range', exact: true }).click()
  56 |  await expect(page.getByRole('status')).toHaveText('No selection')
  57 |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toHaveCount(0)
  58 |  await expect(sceneMarkers(page)).toHaveCount(0)
  59 |  expect((await state(page)).selected).toBeNull()
  60 |  expect((await state(page)).contacts).toEqual([])
  61 |  expect(errors).toEqual([])
  62 |  await info.attach('measurements.json', { body: JSON.stringify({ scanned, groupIds: ids, evicted: await state(page), errors }, null, 2), contentType: 'application/json' })
  63 | })
  64 | test('SENSOR-2 repeat/translated pixels, telescope motion and stale group suppression', async ({ page }, info) => {
  65 |  await boot(page)
  66 |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  67 |  await expect(sceneMarkers(page)).toHaveCount(3)
  68 |  // Blur Pulse so focus decorations cannot differ between captures.
  69 |  await page.locator('canvas').click({ position: { x: 50, y: 650 } })
  70 |  const first = await page.screenshot(), initial = await state(page)
  71 |  expect((await page.screenshot()).equals(first)).toBe(true)
  72 |  await boot(page, true)
  73 |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  74 |  await expect(sceneMarkers(page)).toHaveCount(3)
  75 |  await page.locator('canvas').click({ position: { x: 50, y: 650 } })
  76 |  expect((await page.screenshot()).equals(first)).toBe(true)
  77 |  expect((await state(page)).markers).toEqual(initial.markers)
  78 |  await page.getByRole('button', { name: 'Sensor contact group of 18', exact: true }).click()
  79 |  await page.getByRole('button', { name: 'Telescope', exact: true }).click()
  80 |  await expect(page.getByText('Group no longer visible')).toBeVisible()
  81 |  await expect(page.getByRole('region', { name: 'Detected contacts' }).locator('li button')).toHaveCount(0)
  82 |  await page.getByRole('button', { name: 'All contacts', exact: true }).click()
  83 |  await page.getByRole('textbox').fill('back')
  84 |  await page.keyboard.press('KeyD')
  85 |  const focused = (await state(page)).view
  86 |  await page.keyboard.down('KeyD'); await page.waitForTimeout(100); await page.keyboard.up('KeyD')
  87 |  expect((await state(page)).view).toEqual(focused)
  88 |  await page.locator('canvas').click({ position: { x: 50, y: 650 } })
  89 |  await page.keyboard.down('KeyD'); await page.waitForTimeout(100); await page.keyboard.up('KeyD')
  90 |  await expect.poll(async () => (await state(page)).view.quaternion).not.toEqual(focused.quaternion)
  91 |  expect((await state(page)).contacts).toEqual(initial.contacts)
  92 |  expect((await state(page)).tick).toBe(300)
  93 |  await info.attach('telescope.png', { body: await page.screenshot(), contentType: 'image/png' })
  94 |  await info.attach('measurements.json', { body: JSON.stringify({ initial, moved: await state(page), repeat: 'byte-identical', translated: 'byte-identical' }, null, 2), contentType: 'application/json' })
  95 | })
  96 |
```
