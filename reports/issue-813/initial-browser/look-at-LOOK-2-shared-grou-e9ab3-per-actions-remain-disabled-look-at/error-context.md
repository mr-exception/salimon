# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: look-at.spec.ts >> LOOK-2 shared grounded Scout observes other bodies, Vesper actions remain disabled
- Location: tests/look-at.spec.ts:75:1

# Error details

```
Test timeout of 180000ms exceeded.
```

```
Error: locator.click: Test timeout of 180000ms exceeded.
Call log:
  - waiting for getByRole('button', { name: 'Caligo', exact: true })

```

# Page snapshot

```yaml
- generic [ref=f3e2]:
  - region "Detected contacts" [ref=f3e7]:
    - group [ref=f3e8]:
      - generic "Observation help" [ref=f3e9]
    - generic [ref=f3e10]:
      - text: Search contacts
      - textbox "Search contacts" [ref=f3e11]
    - list
    - button "Previous" [disabled] [ref=f3e12]
    - generic [ref=f3e13]: 1/1
    - button "Next" [disabled] [ref=f3e14]
    - status [ref=f3e15]: No selection
  - navigation [ref=f3e16]:
    - button "Pulse" [ref=f3e17]
    - button "Leave range" [ref=f3e18]
    - button "Reset view" [ref=f3e19]
    - button "Zoom" [ref=f3e20]
    - button "Toggle hidden optical control" [ref=f3e21]
```

# Test source

```ts
  1   | import { test, expect, type Page } from '@playwright/test'
  2   | type State = { ship: { yaw: number; pitch: number; velocityMps: number[]; throttle: number; alignment: unknown; mode: string }; view: { quaternion: number[]; fov: number }; lookingAt: boolean; selected: string | null; contacts: { id: string; name: string }[]; markers: { ids: string[] }[]; renderer: string }
  3   | const state = (page: Page) => page.evaluate(() => (window as unknown as { lookSnapshot(): State }).lookSnapshot())
  4   | async function boot(page: Page, grounded = false) {
  5   |  await page.goto(`/tests/look-at.html${grounded ? '?grounded' : ''}`)
  6   |  await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
  7   | }
  8   | async function choose(page: Page, name: string) {
  9   |  await page.getByRole('textbox').fill(name)
> 10  |  await page.getByRole('button', { name, exact: true }).click()
      |                                                        ^ Error: locator.click: Test timeout of 180000ms exceeded.
  11  | }
  12  | test('LOOK-1 rear browser contact, opaque foreground, independent ship alignment and cancellation', async ({ page }, info) => {
  13  |  const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  14  |  await boot(page)
  15  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toHaveCount(0)
  16  |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  17  |  const initial = await state(page)
  18  |  expect(initial.markers).toHaveLength(0)
  19  |  await choose(page, 'Behind')
  20  |  await page.getByRole('button', { name: 'Zoom', exact: true }).click()
  21  |  const start = await state(page)
  22  |  await page.getByRole('button', { name: 'Look at', exact: true }).click()
  23  |  await expect.poll(async () => (await state(page)).view.quaternion).not.toEqual(start.view.quaternion)
  24  |  await expect.poll(async () => (await state(page)).lookingAt).toBe(false)
  25  |  const looked = await state(page)
  26  |  expect(looked.ship).toEqual(initial.ship)
  27  |  expect(looked.view.fov).toBe(start.view.fov)
  28  |  expect(Math.abs(looked.view.quaternion[1])).toBeCloseTo(1, 8)
  29  |  expect(looked.markers.flatMap(m => m.ids)).not.toContain('Behind')
  30  |  await page.locator('canvas').click({ position: { x: 30, y: 500 } })
  31  |  const opaque = await page.locator('canvas').screenshot()
  32  |  await page.getByRole('button', { name: 'Toggle hidden optical control', exact: true }).click()
  33  |  await page.locator('canvas').click({ position: { x: 30, y: 500 } })
  34  |  expect((await page.locator('canvas').screenshot()).equals(opaque)).toBe(true)
  35  |  await info.attach('occluded.png', { body: opaque, contentType: 'image/png' })
  36  |  await page.getByRole('button', { name: 'Set direction', exact: true }).click()
  37  |  await expect.poll(async () => (await state(page)).ship.yaw).not.toBe(initial.ship.yaw)
  38  |  const turned = await state(page)
  39  |  expect(turned.view).toEqual(looked.view)
  40  |  expect(turned.ship.velocityMps).toEqual(initial.ship.velocityMps)
  41  |  // Camera input does not cancel ongoing physical alignment.
  42  |  await page.locator('canvas').click({ position: { x: 30, y: 500 } })
  43  |  await page.keyboard.press('KeyD')
  44  |  expect((await state(page)).ship.alignment).not.toBeNull()
  45  |  await page.keyboard.press('ArrowLeft')
  46  |  expect((await state(page)).ship.alignment).toBeNull()
  47  |  // Help disclosure is a keyboard control, never a steering surface.
  48  |  await page.locator('summary').focus()
  49  |  const helpFocus = await state(page)
  50  |  await page.keyboard.press('KeyD'); await page.keyboard.press('ArrowLeft')
  51  |  expect((await state(page)).view).toEqual(helpFocus.view)
  52  |  expect((await state(page)).ship.yaw).toBe(helpFocus.ship.yaw)
  53  |  // Focused typing cannot move either controller.
  54  |  await page.getByRole('textbox').focus()
  55  |  const focused = await state(page)
  56  |  await page.keyboard.press('KeyD'); await page.keyboard.press('ArrowLeft')
  57  |  expect((await state(page)).view).toEqual(focused.view)
  58  |  expect((await state(page)).ship.yaw).toEqual(focused.ship.yaw)
  59  |  expect((await state(page)).ship.pitch).toEqual(focused.ship.pitch)
  60  |  expect((await state(page)).ship.velocityMps).toEqual(focused.ship.velocityMps)
  61  |  await page.getByRole('button', { name: 'Reset view', exact: true }).click()
  62  |  await page.getByRole('button', { name: 'Look at', exact: true }).click()
  63  |  await expect.poll(async () => (await state(page)).lookingAt).toBe(true)
  64  |  await page.locator('canvas').click({ position: { x: 30, y: 500 } })
  65  |  await expect.poll(async () => (await state(page)).lookingAt).toBe(false)
  66  |  const cancelled = await state(page)
  67  |  await page.getByRole('textbox').focus()
  68  |  expect((await state(page)).view).toEqual(cancelled.view)
  69  |  // Loss during a captured animation clears commands without retaining a ghost.
  70  |  await page.getByRole('button', { name: 'Look at', exact: true }).click()
  71  |  await page.getByRole('button', { name: 'Leave range', exact: true }).click()
  72  |  await expect(page.getByRole('status')).toHaveText('No selection')
  73  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toHaveCount(0)
  74  |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toHaveCount(0)
  75  |  await expect.poll(async () => (await state(page)).lookingAt).toBe(false)
  76  |  const lost = await state(page)
  77  |  expect(Math.abs(lost.view.quaternion[1])).toBeCloseTo(1, 8)
  78  |  expect(lost.contacts).toEqual([]); expect(lost.selected).toBeNull()
  79  |  expect(errors).toEqual([])
  80  |  await info.attach('measurements.json', { body: JSON.stringify({ initial, start, looked, turned, focused, cancelled, lost, opaqueControl: 'byte-identical', errors }, (_, v) => typeof v === 'bigint' ? String(v) : v, 2), contentType: 'application/json' })
  81  | })
  82  | test('LOOK-2 shared grounded Scout observes other bodies, Vesper actions remain disabled', async ({ page }, info) => {
  83  |  await boot(page, true)
  84  |  await page.getByRole('button', { name: 'Pulse', exact: true }).click()
  85  |  const initial = await state(page)
  86  |  const planet = initial.contacts.find(b => b.id === 'vesper')!
  87  |  await choose(page, planet.name)
  88  |  await expect(page.getByText(`Current landing planet: ${planet.name}`)).toBeVisible()
  89  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeDisabled()
  90  |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
  91  |  await info.attach('vesper.png', { body: await page.screenshot(), contentType: 'image/png' })
  92  |  const other = initial.contacts.find(b => b.id !== 'vesper')!
  93  |  await choose(page, other.name)
  94  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeEnabled()
  95  |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
  96  |  await page.getByRole('button', { name: 'Look at', exact: true }).click()
  97  |  await expect.poll(async () => (await state(page)).view.quaternion).not.toEqual(initial.view.quaternion)
  98  |  await expect.poll(async () => (await state(page)).lookingAt).toBe(false)
  99  |  const observed = await state(page)
  100 |  expect(observed.ship).toEqual(initial.ship)
  101 |  await info.attach('grounded-observation.png', { body: await page.screenshot(), contentType: 'image/png' })
  102 |  await info.attach('measurements.json', { body: JSON.stringify({ initial, observed }, (_, v) => typeof v === 'bigint' ? String(v) : v, 2), contentType: 'application/json' })
  103 | })
  104 |
```
