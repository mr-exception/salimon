# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: panoramic.spec.ts >> HUD-1 full viewport, scanner authority, keyboard browser and priority Escape
- Location: tests/panoramic.spec.ts:13:1

# Error details

```
Test timeout of 180000ms exceeded.
```

```
Error: mouse.wheel: Test timeout of 180000ms exceeded.
```

# Page snapshot

```yaml
- generic [ref=f1e3]:
  - generic "Panoramic universe" [ref=f1e6]
  - generic [aria-hidden]: +
  - banner:
    - strong: SALIMON · OBSERVATION
    - status: Scanner pulse 34 · 40 contacts
    - generic: flying · 0.0 m/s · sample inertial frame
  - status "Navigation warnings"
  - generic:
    - button "Sensor contact group of 40" [ref=f1e7] [cursor=pointer]: "40"
    - region "Detected contacts" [ref=f1e8]:
      - button "Contacts (C) · 40" [ref=f1e9] [cursor=pointer]
      - status [ref=f1e11]: No selection
  - generic:
    - figure "3D orientation compass":
      - 'img "Camera cyan at center. Heading yellow: front, right 0.0, up 0.0, depth 1.00. Velocity green: Stationary. Solid front, hollow rear."':
        - generic: UP
      - generic:
        - text: Cyan view · Yellow heading
        - emphasis: Stationary
        - text: · solid front / hollow rear
  - contentinfo [ref=f1e12]:
    - generic [ref=f1e13]:
      - text: Throttle 0%
      - slider "Throttle" [ref=f1e14]: "0"
    - button "Zero throttle (Space)" [ref=f1e15] [cursor=pointer]
    - button "Reset view" [ref=f1e16] [cursor=pointer]
    - button "Telemetry" [ref=f1e17] [cursor=pointer]
    - button "Help" [ref=f1e18] [cursor=pointer]
    - button "Exit cockpit" [ref=f1e19] [cursor=pointer]
```

# Test source

```ts
  1   | import { test, expect, type Page, type TestInfo } from '@playwright/test'
  2   | type State = { ship: { yaw: number; pitch: number; throttle: number; velocityMps: number[] }; view: { quaternion: number[]; fov: number }; renderer: string; contacts: string[]; exits: number }
  3   | const state = (page: Page) => page.evaluate(() => (window as unknown as { hudSnapshot(): State }).hudSnapshot())
  4   | const acquireContacts = (page: Page) => page.evaluate(() => (window as unknown as { hudAdvance(seconds: number): void }).hudAdvance(5))
  5   | const scenario = (page: Page, kind: string) => page.evaluate(kind => (window as unknown as { hudScenario(kind: string): void }).hudScenario(kind), kind)
  6   | async function boot(page: Page, grounded = false) {
  7   |  await page.goto(`/tests/panoramic.html${grounded ? '?grounded' : ''}`)
  8   |  await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
  9   | }
  10  | async function capture(page: Page, name: string, info: TestInfo) {
  11  |  await info.attach(name, { body: await page.screenshot(), contentType: 'image/png' })
  12  | }
  13  | test('HUD-1 full viewport, scanner authority, keyboard browser and priority Escape', async ({ page }, info) => {
  14  |  await boot(page)
  15  |  await expect(page.getByLabel('Search contacts')).toHaveCount(0)
  16  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toHaveCount(0)
  17  |  const dimensions = await page.locator('canvas').boundingBox()
  18  |  expect(dimensions).toMatchObject({ x: 0, y: 0, width: 1280, height: 720 })
  19  |  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(720)
  20  |  await expect(page.getByText('Stationary', { exact: false })).toBeVisible()
  21  |  // Advance five domain seconds; scanner cadence/range still belongs to the runtime.
  22  |  await acquireContacts(page)
  23  |  await expect.poll(async () => (await state(page)).contacts.length).toBe(40)
  24  |  await capture(page, 'desktop-1280x720.png', info)
  25  |  await page.locator('canvas').focus(); await page.keyboard.press('Tab')
  26  |  for (let i = 0; i < 45 && !await page.getByRole('button', { name: /Contacts \(C\)/ }).evaluate(el => el === document.activeElement); i++) await page.keyboard.press('Tab')
  27  |  await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
  28  |  await page.keyboard.press('Enter')
  29  |  await expect(page.getByLabel('Search contacts')).toBeFocused()
  30  |  const focused = await state(page)
  31  |  await page.keyboard.press('KeyD'); await page.keyboard.press('ArrowLeft')
> 32  |  await page.mouse.move(500, 400); await page.mouse.wheel(0, -500)
      |                                                    ^ Error: mouse.wheel: Test timeout of 180000ms exceeded.
  33  |  expect((await state(page)).view).toEqual(focused.view)
  34  |  expect((await state(page)).ship.yaw).toBe(focused.ship.yaw)
  35  |  await page.getByLabel('Search contacts').fill('')
  36  |  const ids = new Set<string>()
  37  |  for (let i = 0; i < 5; i++) {
  38  |   for (const name of await page.locator('#scanned-browser li button').allTextContents()) ids.add(name)
  39  |   if (i < 4) await page.getByRole('button', { name: 'Next', exact: true }).click()
  40  |  }
  41  |  expect(ids.size).toBe(40)
  42  |  await page.getByLabel('Search contacts').fill('Contact 39')
  43  |  await page.getByRole('button', { name: 'Contact 39', exact: true }).focus(); await page.keyboard.press('Enter')
  44  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeEnabled()
  45  |  await page.keyboard.press('Escape')
  46  |  await expect(page.getByLabel('Search contacts')).toHaveCount(0)
  47  |  await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
  48  |  expect((await state(page)).exits).toBe(0)
  49  |  await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
  50  |  await page.getByRole('button', { name: 'Help', exact: true }).click()
  51  |  await expect(page.getByRole('dialog')).toBeVisible()
  52  |  await capture(page, 'help.png', info)
  53  |  await page.keyboard.press('Escape')
  54  |  await expect(page.getByRole('dialog')).toHaveCount(0)
  55  |  await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toBeVisible()
  56  |  await page.keyboard.press('Escape')
  57  |  await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toHaveCount(0)
  58  |  expect((await state(page)).exits).toBe(0)
  59  |  await page.keyboard.press('Escape'); expect((await state(page)).exits).toBe(1)
  60  |  await info.attach('browser-state.json', { body: JSON.stringify(await state(page), (_, value) => typeof value === 'bigint' ? String(value) : value), contentType: 'application/json' })
  61  | })
  62  | test('HUD-2 compass render states, ship rotation, zoom and responsive warning priority', async ({ page }, info) => {
  63  |  await boot(page)
  64  |  await scenario(page, 'rear')
  65  |  await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'true')
  66  |  await expect(page.locator('[data-compass=velocity]')).toHaveAttribute('data-rear', 'true')
  67  |  await capture(page, 'rear.png', info)
  68  |  const rear = await page.locator('.orientation-compass').screenshot()
  69  |  expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(true)
  70  |  await scenario(page, 'overlap')
  71  |  await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'false')
  72  |  await capture(page, 'overlap.png', info)
  73  |  expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(false)
  74  |  await scenario(page, 'stationary')
  75  |  await expect(page.locator('[data-compass=velocity]')).toHaveCount(0)
  76  |  await page.locator('canvas').focus()
  77  |  const before = await state(page)
  78  |  await page.keyboard.down('ArrowLeft')
  79  |  await expect.poll(async () => (await state(page)).ship.yaw).not.toBe(before.ship.yaw)
  80  |  await page.keyboard.up('ArrowLeft')
  81  |  expect((await state(page)).view).toEqual(before.view)
  82  |  await page.mouse.move(500, 400); await page.mouse.wheel(0, -1000)
  83  |  await expect.poll(async () => (await state(page)).view.fov).toBeLessThan(75)
  84  |  await page.getByRole('button', { name: 'Reset view', exact: true }).click()
  85  |  await expect.poll(async () => (await state(page)).view.fov).toBe(75)
  86  |  await page.getByRole('slider', { name: 'Throttle', exact: true }).fill('30')
  87  |  await expect.poll(async () => (await state(page)).ship.throttle).toBe(30)
  88  |  await page.getByRole('button', { name: 'Zero throttle (Space)', exact: true }).click()
  89  |  expect((await state(page)).ship.throttle).toBe(0)
  90  |  await page.setViewportSize({ width: 960, height: 600 })
  91  |  await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
  92  |  await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
  93  |  await scenario(page, 'warning')
  94  |  await expect(page.getByLabel('Navigation warnings')).not.toBeEmpty()
  95  |  await capture(page, 'warning-960x600.png', info)
  96  |  await scenario(page, 'destroyed')
  97  |  await expect(page.getByText(/Journey ended/)).toBeVisible()
  98  |  await expect(page.getByRole('button', { name: 'Restart navigation', exact: true })).toBeVisible()
  99  |  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(600)
  100 |  await capture(page, 'terminal-960x600.png', info)
  101 |  await page.getByRole('button', { name: 'Restart navigation', exact: true }).click()
  102 |  await expect(page.getByText(/Journey ended/)).toHaveCount(0)
  103 | })
  104 | test('HUD-3 grounded authority and no sidebar at smaller desktop size', async ({ page }, info) => {
  105 |  await page.setViewportSize({ width: 960, height: 600 }); await boot(page, true)
  106 |  await expect(page.getByRole('slider', { name: 'Throttle', exact: true })).toBeDisabled()
  107 |  await acquireContacts(page)
  108 |  await expect.poll(async () => (await state(page)).contacts.length).toBeGreaterThan(0)
  109 |  await page.locator('canvas').focus(); await page.keyboard.press('KeyC')
  110 |  await page.getByLabel('Search contacts').fill('Vesper')
  111 |  await page.locator('#scanned-browser li button').first().click()
  112 |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeDisabled()
  113 |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
  114 |  await page.keyboard.press('Escape')
  115 |  await capture(page, 'grounded-960x600.png', info)
  116 |  expect(await page.locator('aside').count()).toBe(0)
  117 |  expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, width: 960, height: 600 })
  118 | })
  119 |
```
