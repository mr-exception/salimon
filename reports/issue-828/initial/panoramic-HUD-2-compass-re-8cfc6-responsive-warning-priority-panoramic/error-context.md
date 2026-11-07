# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: panoramic.spec.ts >> HUD-2 compass render states, ship rotation, zoom and responsive warning priority
- Location: tests/panoramic.spec.ts:71:1

# Error details

```
Error: locator.screenshot: Element is not attached to the DOM
Call log:
  - taking element screenshot
  - waiting for fonts to load...
  - fonts loaded
  - attempting scroll into view action
    - waiting for element to be stable

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
  11  |  const path = info.outputPath(name)
  12  |  await page.screenshot({ path })
  13  |  await info.attach(name, { path, contentType: 'image/png' })
  14  | }
  15  | test('HUD-1 full viewport, scanner authority, keyboard browser and priority Escape', async ({ page }, info) => {
  16  |  await boot(page)
  17  |  await expect(page.getByLabel('Search contacts')).toHaveCount(0)
  18  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toHaveCount(0)
  19  |  const dimensions = await page.locator('canvas').boundingBox()
  20  |  expect(dimensions).toMatchObject({ x: 0, y: 0, width: 1280, height: 720 })
  21  |  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(720)
  22  |  await expect(page.getByText('Stationary', { exact: false })).toBeVisible()
  23  |  // Advance five domain seconds; scanner cadence/range still belongs to the runtime.
  24  |  await acquireContacts(page)
  25  |  await expect.poll(async () => (await state(page)).contacts.length).toBe(40)
  26  |  await capture(page, 'desktop-1280x720.png', info)
  27  |  await page.locator('canvas').focus(); await page.keyboard.press('Tab')
  28  |  for (let i = 0; i < 45 && !await page.getByRole('button', { name: /Contacts \(C\)/ }).evaluate(el => el === document.activeElement); i++) await page.keyboard.press('Tab')
  29  |  await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
  30  |  await page.keyboard.press('Enter')
  31  |  await expect(page.getByLabel('Search contacts')).toBeFocused()
  32  |  const focused = await state(page)
  33  |  await page.keyboard.press('KeyD'); await page.keyboard.press('ArrowLeft')
  34  |  const consumed = await page.evaluate(() => {
  35  |   const event = new WheelEvent('wheel', { deltaY: -500, bubbles: true, cancelable: true })
  36  |   document.querySelector('canvas')!.dispatchEvent(event)
  37  |   return event.defaultPrevented
  38  |  })
  39  |  expect(consumed).toBe(true)
  40  |  await page.mouse.move(500, 400); await page.mouse.wheel(0, -500)
  41  |  expect((await state(page)).view).toEqual(focused.view)
  42  |  expect((await state(page)).ship.yaw).toBe(focused.ship.yaw)
  43  |  await page.getByLabel('Search contacts').fill('')
  44  |  const ids = new Set<string>()
  45  |  for (let i = 0; i < 5; i++) {
  46  |   for (const name of await page.locator('#scanned-browser li button').allTextContents()) ids.add(name)
  47  |   if (i < 4) await page.getByRole('button', { name: 'Next', exact: true }).click()
  48  |  }
  49  |  expect(ids.size).toBe(40)
  50  |  await page.getByLabel('Search contacts').fill('Contact 39')
  51  |  await page.getByRole('button', { name: 'Contact 39', exact: true }).focus(); await page.keyboard.press('Enter')
  52  |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeEnabled()
  53  |  await page.keyboard.press('Escape')
  54  |  await expect(page.getByLabel('Search contacts')).toHaveCount(0)
  55  |  await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
  56  |  expect((await state(page)).exits).toBe(0)
  57  |  await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
  58  |  await page.getByRole('button', { name: 'Help', exact: true }).click()
  59  |  await expect(page.getByRole('dialog')).toBeVisible()
  60  |  await expect(page.getByRole('heading', { name: 'Navigation help', exact: true })).toBeFocused()
  61  |  await expect(page.getByRole('heading', { name: 'Navigation help', exact: true })).toBeInViewport()
  62  |  await capture(page, 'help.png', info)
  63  |  await page.keyboard.press('Escape')
  64  |  await expect(page.getByRole('dialog')).toHaveCount(0)
  65  |  await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toBeVisible()
  66  |  await page.keyboard.press('Escape')
  67  |  await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toHaveCount(0)
  68  |  expect((await state(page)).exits).toBe(0)
  69  |  await page.evaluate(() => (document.activeElement as HTMLElement)?.blur())
  70  |  await page.keyboard.press('Escape'); expect((await state(page)).exits).toBe(1)
  71  |  await info.attach('browser-state.json', { body: JSON.stringify(await state(page), (_, value) => typeof value === 'bigint' ? String(value) : value), contentType: 'application/json' })
  72  | })
  73  | test('HUD-2 compass render states, ship rotation, zoom and responsive warning priority', async ({ page }, info) => {
  74  |  await boot(page)
  75  |  await scenario(page, 'rear')
  76  |  await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'true')
> 77  |  await expect(page.locator('[data-compass=velocity]')).toHaveAttribute('data-rear', 'true')
      |                                                          ^ Error: locator.screenshot: Element is not attached to the DOM
  78  |  await capture(page, 'rear.png', info)
  79  |  const rear = await page.locator('.orientation-compass').screenshot()
  80  |  expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(true)
  81  |  await scenario(page, 'overlap')
  82  |  await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'false')
  83  |  await capture(page, 'overlap.png', info)
  84  |  expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(false)
  85  |  await scenario(page, 'stationary')
  86  |  await expect(page.locator('[data-compass=velocity]')).toHaveCount(0)
  87  |  await page.locator('canvas').focus()
  88  |  const before = await state(page)
  89  |  await page.keyboard.down('ArrowLeft')
  90  |  await expect.poll(async () => (await state(page)).ship.yaw).not.toBe(before.ship.yaw)
  91  |  await page.keyboard.up('ArrowLeft')
  92  |  expect((await state(page)).view).toEqual(before.view)
  93  |  await page.mouse.move(500, 400); await page.mouse.wheel(0, -1000)
  94  |  await expect.poll(async () => (await state(page)).view.fov).toBeLessThan(75)
  95  |  await page.getByRole('button', { name: 'Reset view', exact: true }).click()
  96  |  await expect.poll(async () => (await state(page)).view.fov).toBe(75)
  97  |  await page.getByRole('slider', { name: 'Throttle', exact: true }).fill('30')
  98  |  await expect.poll(async () => (await state(page)).ship.throttle).toBe(30)
  99  |  await page.getByRole('button', { name: 'Zero throttle (Space)', exact: true }).click()
  100 |  expect((await state(page)).ship.throttle).toBe(0)
  101 |  await page.setViewportSize({ width: 960, height: 600 })
  102 |  await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
  103 |  await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
  104 |  await scenario(page, 'warning')
  105 |  await expect(page.getByLabel('Navigation warnings')).not.toBeEmpty()
  106 |  for (const name of ['Reset view', 'Exit cockpit', 'Telemetry']) {
  107 |   const control = page.getByRole('button', { name, exact: true })
  108 |   await expect(control).toBeInViewport()
  109 |   expect(await control.evaluate(el => {
  110 |    const r = el.getBoundingClientRect()
  111 |    return el.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2))
  112 |   }), `ESSENTIAL_CONTROL_OBSTRUCTED ${name}`).toBe(true)
  113 |  }
  114 |  await capture(page, 'warning-960x600.png', info)
  115 |  await scenario(page, 'destroyed')
  116 |  await expect(page.getByText(/Journey ended/)).toBeVisible()
  117 |  await expect(page.getByRole('button', { name: 'Restart navigation', exact: true })).toBeVisible()
  118 |  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(600)
  119 |  await capture(page, 'terminal-960x600.png', info)
  120 |  await page.getByRole('button', { name: 'Restart navigation', exact: true }).click()
  121 |  await expect(page.getByText(/Journey ended/)).toHaveCount(0)
  122 | })
  123 | test('HUD-3 grounded authority and no sidebar at smaller desktop size', async ({ page }, info) => {
  124 |  await page.setViewportSize({ width: 960, height: 600 }); await boot(page, true)
  125 |  await expect(page.getByRole('slider', { name: 'Throttle', exact: true })).toBeDisabled()
  126 |  await acquireContacts(page)
  127 |  await expect.poll(async () => (await state(page)).contacts.length).toBeGreaterThan(0)
  128 |  await page.locator('canvas').focus(); await page.keyboard.press('KeyC')
  129 |  await page.getByLabel('Search contacts').fill('Vesper')
  130 |  await page.locator('#scanned-browser li button').first().click()
  131 |  await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeDisabled()
  132 |  await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
  133 |  await page.keyboard.press('Escape')
  134 |  await capture(page, 'grounded-960x600.png', info)
  135 |  expect(await page.locator('aside').count()).toBe(0)
  136 |  expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, width: 960, height: 600 })
  137 | })
  138 | 
  139 | for (const viewport of [{ width: 960, height: 600 }, { width: 1280, height: 720 }, { width: 1920, height: 1080 }]) {
  140 |  test(`HUD-STYLE projected controls, contrast and focused help at ${viewport.width}`, async ({ page }, info) => {
  141 |   await page.setViewportSize(viewport)
  142 |   await page.clock.install({ time: new Date('2026-11-07T12:00:00Z') })
  143 |   await page.clock.pauseAt(new Date('2026-11-07T12:00:01Z'))
  144 |   const measurements = []
  145 |   for (const grounded of [false, true]) {
  146 |    await page.goto(`/tests/panoramic.html${grounded ? '?grounded' : ''}`)
  147 |    await expect.poll(async () => { await page.clock.runFor(64); return (await state(page))?.renderer }).toContain('SwiftShader')
  148 |    await acquireContacts(page); await page.clock.runFor(64)
  149 |    expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, ...viewport })
  150 |    await page.getByRole('button', { name: /Contacts \(C\)/ }).click(); await page.clock.runFor(64)
  151 |    await expect(page.getByLabel('Search contacts')).toBeFocused()
  152 |    const context = grounded ? 'bright-horizon' : 'dark-dense'
  153 |    await capture(page, `${context}.png`, info)
  154 |    const contrast = await page.locator('.panoramic').evaluate(el => {
  155 |     const style = getComputedStyle(el)
  156 |     const luminance = (hex: string) => {
  157 |      const c = hex.trim().slice(1).match(/../g)!.map(v => parseInt(v, 16) / 255).map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4)
  158 |      return c[0] * .2126 + c[1] * .7152 + c[2] * .0722
  159 |     }
  160 |     const bg = luminance(style.getPropertyValue('--ui-void'))
  161 |     return Object.fromEntries(['text', 'muted', 'active', 'mechanical', 'danger'].map(name => [name, (luminance(style.getPropertyValue(`--ui-${name}`)) + .05) / (bg + .05)]))
  162 |    })
  163 |    for (const ratio of Object.values(contrast)) expect(ratio).toBeGreaterThanOrEqual(4.5)
  164 |    // Real computed panel surfaces must be opaque; the bright sky cannot change ink contrast.
  165 |    expect(await page.locator('.panoramic-status').evaluate(el => getComputedStyle(el).backgroundColor)).toBe('rgb(9, 20, 28)')
  166 |    await page.keyboard.press('Escape'); await page.clock.runFor(64)
  167 |    await page.getByRole('button', { name: 'Help', exact: true }).click(); await page.clock.runFor(64)
  168 |    await expect(page.getByRole('heading', { name: 'Navigation help' })).toBeFocused()
  169 |    const before = await state(page)
  170 |    await page.keyboard.press('w'); await page.keyboard.press('ArrowLeft'); await page.clock.runFor(64)
  171 |    expect((await state(page)).view).toEqual(before.view)
  172 |    expect((await state(page)).ship.yaw).toEqual(before.ship.yaw)
  173 |    await capture(page, `focused-help-${context}.png`, info)
  174 |    await page.keyboard.press('Escape'); await page.clock.runFor(64)
  175 |    await expect(page.getByRole('button', { name: 'Help', exact: true })).toBeFocused()
  176 |    expect(await page.evaluate(() => ({ w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight }))).toEqual({ w: viewport.width, h: viewport.height })
  177 |    measurements.push({ context, viewport, contrast, state: await state(page) })
```