# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: panoramic.spec.ts >> HUD-1 full viewport, scanner authority, keyboard browser and priority Escape
- Location: tests/panoramic.spec.ts:12:1

# Error details

```
Error: expect(locator).toHaveCount(expected) failed

Locator:  getByRole('region', { name: 'Detailed telemetry' })
Expected: 0
Received: 1
Timeout:  10000ms

Call log:
  - Expect "toHaveCount" getByRole('region', { name: 'Detailed telemetry' }) with timeout 10000ms
  - waiting for getByRole('region', { name: 'Detailed telemetry' })
    22 × locator resolved to 1 element
       - unexpected value "1"

```

# Page snapshot

```yaml
- generic [ref=e3]:
  - generic "Panoramic universe" [ref=e6]
  - generic [aria-hidden]: +
  - banner:
    - strong: SALIMON · OBSERVATION
    - status: Scanner pulse 17 · 40 contacts
    - generic: flying · 0.0 m/s · sample inertial frame
  - status "Navigation warnings"
  - generic:
    - button "Sensor contact group of 40" [ref=e7] [cursor=pointer]: "40"
    - region "Detected contacts" [ref=e8]:
      - button "Contacts (C) · 40" [ref=e9] [cursor=pointer]
      - generic [ref=e10]:
        - status [ref=e11]: "Selected: Contact 39"
        - button "Look at" [ref=e12] [cursor=pointer]
        - button "Set direction" [ref=e13] [cursor=pointer]
  - generic:
    - figure "3D orientation compass":
      - 'img "Camera cyan at center. Heading yellow: front, right 0.0, up 0.0, depth 1.00. Velocity green: Stationary. Solid front, hollow rear."':
        - generic: UP
      - generic:
        - text: Cyan view · Yellow heading
        - emphasis: Stationary
        - text: · solid front / hollow rear
  - contentinfo [ref=e14]:
    - generic [ref=e15]:
      - text: Throttle 0%
      - slider "Throttle" [ref=e16]: "0"
    - button "Zero throttle (Space)" [ref=e17] [cursor=pointer]
    - button "Reset view" [ref=e18] [cursor=pointer]
    - button "Telemetry" [expanded] [ref=e19] [cursor=pointer]
    - button "Help" [ref=e20] [cursor=pointer]
    - button "Exit cockpit" [ref=e21] [cursor=pointer]
  - region "Detailed telemetry" [ref=e22]:
    - paragraph [ref=e23]: Mass 100,000 kg · FOV 75.00°
    - paragraph [ref=e24]: Yaw 0.0° · pitch 0.0° · Manual
    - paragraph [ref=e25]: Cell 0 / 0 / 0 · local 0.0 / 0.0 / 0.0 m
    - paragraph [ref=e26]: Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run.
```

# Test source

```ts
  1   | import { test, expect, type Page, type TestInfo } from '@playwright/test'
  2   | type State = { ship: { yaw: number; pitch: number; throttle: number; velocityMps: number[] }; view: { quaternion: number[]; fov: number }; renderer: string; contacts: string[]; exits: number }
  3   | const state = (page: Page) => page.evaluate(() => (window as unknown as { hudSnapshot(): State }).hudSnapshot())
  4   | const scenario = (page: Page, kind: string) => page.evaluate(kind => (window as unknown as { hudScenario(kind: string): void }).hudScenario(kind), kind)
  5   | async function boot(page: Page, grounded = false) {
  6   |  await page.goto(`/tests/panoramic.html${grounded ? '?grounded' : ''}`)
  7   |  await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
  8   | }
  9   | async function capture(page: Page, name: string, info: TestInfo) {
  10  |  await info.attach(name, { body: await page.screenshot(), contentType: 'image/png' })
  11  | }
  12  | test('HUD-1 full viewport, scanner authority, keyboard browser and priority Escape', async ({ page }, info) => {
  13  |  await boot(page)
  14  |  await expect(page.getByLabel('Search contacts')).toHaveCount(0)
  15  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toHaveCount(0)
  16  |  const dimensions = await page.locator('canvas').boundingBox()
  17  |  expect(dimensions).toMatchObject({ x: 0, y: 0, width: 1280, height: 720 })
  18  |  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(720)
  19  |  await expect(page.getByText('Stationary', { exact: false })).toBeVisible()
  20  |  await expect.poll(async () => (await state(page)).contacts.length).toBe(40)
  21  |  await capture(page, 'desktop-1280x720.png', info)
  22  |  await page.locator('canvas').focus(); await page.keyboard.press('Tab')
  23  |  for (let i = 0; i < 45 && !await page.getByRole('button', { name: /Contacts \(C\)/ }).evaluate(el => el === document.activeElement); i++) await page.keyboard.press('Tab')
  24  |  await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
  25  |  await page.keyboard.press('Enter')
  26  |  await expect(page.getByLabel('Search contacts')).toBeFocused()
  27  |  const focused = await state(page)
  28  |  await page.keyboard.press('KeyD'); await page.keyboard.press('ArrowLeft')
  29  |  await page.mouse.move(500, 400); await page.mouse.wheel(0, -500)
  30  |  expect((await state(page)).view).toEqual(focused.view)
  31  |  expect((await state(page)).ship.yaw).toBe(focused.ship.yaw)
  32  |  await page.getByLabel('Search contacts').fill('')
  33  |  const ids = new Set<string>()
  34  |  for (let i = 0; i < 5; i++) {
  35  |   for (const name of await page.locator('#scanned-browser li button').allTextContents()) ids.add(name)
  36  |   if (i < 4) await page.getByRole('button', { name: 'Next', exact: true }).click()
  37  |  }
  38  |  expect(ids.size).toBe(40)
  39  |  await page.getByLabel('Search contacts').fill('Contact 39')
  40  |  await page.getByRole('button', { name: 'Contact 39', exact: true }).focus(); await page.keyboard.press('Enter')
  41  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeEnabled()
  42  |  await page.keyboard.press('Escape')
  43  |  await expect(page.getByLabel('Search contacts')).toHaveCount(0)
  44  |  await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
  45  |  expect((await state(page)).exits).toBe(0)
  46  |  await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
  47  |  await page.getByRole('button', { name: 'Help', exact: true }).click()
  48  |  await expect(page.getByRole('dialog')).toBeVisible()
  49  |  await page.keyboard.press('Escape')
  50  |  await expect(page.getByRole('dialog')).toHaveCount(0)
  51  |  await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toBeVisible()
  52  |  await page.keyboard.press('Escape')
> 53  |  await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toHaveCount(0)
      |                                                                         ^ Error: expect(locator).toHaveCount(expected) failed
  54  |  expect((await state(page)).exits).toBe(0)
  55  |  await page.keyboard.press('Escape'); expect((await state(page)).exits).toBe(1)
  56  |  await info.attach('browser-state.json', { body: JSON.stringify(await state(page), (_, value) => typeof value === 'bigint' ? String(value) : value), contentType: 'application/json' })
  57  | })
  58  | test('HUD-2 compass render states, ship rotation, zoom and responsive warning priority', async ({ page }, info) => {
  59  |  await boot(page)
  60  |  await scenario(page, 'rear')
  61  |  await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'true')
  62  |  await expect(page.locator('[data-compass=velocity]')).toHaveAttribute('data-rear', 'true')
  63  |  await capture(page, 'rear.png', info)
  64  |  const rear = await page.locator('.orientation-compass').screenshot()
  65  |  expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(true)
  66  |  await scenario(page, 'overlap')
  67  |  await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'false')
  68  |  await capture(page, 'overlap.png', info)
  69  |  expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(false)
  70  |  await scenario(page, 'stationary')
  71  |  await expect(page.locator('[data-compass=velocity]')).toHaveCount(0)
  72  |  await page.locator('canvas').focus()
  73  |  const before = await state(page)
  74  |  await page.keyboard.down('ArrowLeft')
  75  |  await expect.poll(async () => (await state(page)).ship.yaw).not.toBe(before.ship.yaw)
  76  |  await page.keyboard.up('ArrowLeft')
  77  |  expect((await state(page)).view).toEqual(before.view)
  78  |  await page.mouse.move(500, 400); await page.mouse.wheel(0, -1000)
  79  |  await expect.poll(async () => (await state(page)).view.fov).toBeLessThan(75)
  80  |  await page.getByRole('button', { name: 'Reset view', exact: true }).click()
  81  |  await expect.poll(async () => (await state(page)).view.fov).toBe(75)
  82  |  await page.getByRole('slider', { name: 'Throttle', exact: true }).fill('30')
  83  |  await expect.poll(async () => (await state(page)).ship.throttle).toBe(30)
  84  |  await page.getByRole('button', { name: 'Zero throttle (Space)', exact: true }).click()
  85  |  expect((await state(page)).ship.throttle).toBe(0)
  86  |  await page.setViewportSize({ width: 960, height: 600 })
  87  |  await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
  88  |  await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
  89  |  await scenario(page, 'warning')
  90  |  await expect(page.getByLabel('Navigation warnings')).not.toBeEmpty()
  91  |  await capture(page, 'warning-960x600.png', info)
  92  |  await scenario(page, 'destroyed')
  93  |  await expect(page.getByText(/Journey ended/)).toBeVisible()
  94  |  await expect(page.getByRole('button', { name: 'Restart navigation', exact: true })).toBeVisible()
  95  |  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(600)
  96  |  await capture(page, 'terminal-960x600.png', info)
  97  |  await page.getByRole('button', { name: 'Restart navigation', exact: true }).click()
  98  |  await expect(page.getByText(/Journey ended/)).toHaveCount(0)
  99  | })
  100 | test('HUD-3 grounded authority and no sidebar at smaller desktop size', async ({ page }, info) => {
  101 |  await page.setViewportSize({ width: 960, height: 600 }); await boot(page, true)
  102 |  await expect(page.getByRole('slider', { name: 'Throttle', exact: true })).toBeDisabled()
  103 |  await expect.poll(async () => (await state(page)).contacts.length).toBeGreaterThan(0)
  104 |  await page.locator('canvas').focus(); await page.keyboard.press('KeyC')
  105 |  await page.getByLabel('Search contacts').fill('Vesper')
  106 |  await page.locator('#scanned-browser li button').first().click()
  107 |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeDisabled()
  108 |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
  109 |  await page.keyboard.press('Escape')
  110 |  await capture(page, 'grounded-960x600.png', info)
  111 |  expect(await page.locator('aside').count()).toBe(0)
  112 |  expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, width: 960, height: 600 })
  113 | })
  114 |
```
