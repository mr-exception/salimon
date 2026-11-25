import { test, expect, type Page, type TestInfo } from '@playwright/test'
import { readFile } from 'node:fs/promises'
import { PNG } from 'pngjs'
type State = { ship: { yaw: number; pitch: number; throttle: number; velocityMps: number[]; position: unknown; mode: string }; view: { quaternion: number[]; fov: number }; renderer: string; contacts: string[]; exits: number }
const state = (page: Page) => page.evaluate(() => (window as unknown as { hudSnapshot(): State }).hudSnapshot())
const acquireContacts = (page: Page) => page.evaluate(() => (window as unknown as { hudAdvance(seconds: number): void }).hudAdvance(5))
const scenario = (page: Page, kind: string) => page.evaluate(kind => (window as unknown as { hudScenario(kind: string): void }).hudScenario(kind), kind)
async function boot(page: Page, grounded = false) {
 await page.goto(`/tests/panoramic.html${grounded ? '?grounded' : ''}`)
 await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
}
async function capture(page: Page, name: string, info: TestInfo) {
 const path = info.outputPath(name)
 await page.screenshot({ path })
 await info.attach(name, { path, contentType: 'image/png' })
}
test('HUD-1 full viewport, scanner authority, keyboard browser and priority Escape', async ({ page }, info) => {
 await boot(page)
 await expect(page.getByLabel('Search contacts')).toHaveCount(0)
 await expect(page.getByRole('button', { name: 'Look at', exact: true })).toHaveCount(0)
 const dimensions = await page.locator('canvas').boundingBox()
 expect(dimensions).toMatchObject({ x: 0, y: 0, width: 1280, height: 720 })
 expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(720)
 await expect(page.getByText('Stationary', { exact: false })).toBeVisible()
 // Advance five domain seconds; scanner cadence/range still belongs to the runtime.
 await acquireContacts(page)
 await expect.poll(async () => (await state(page)).contacts.length).toBe(40)
 await capture(page, 'desktop-1280x720.png', info)
 await page.locator('canvas').focus(); await page.keyboard.press('Tab')
 for (let i = 0; i < 45 && !await page.getByRole('button', { name: /Contacts \(C\)/ }).evaluate(el => el === document.activeElement); i++) await page.keyboard.press('Tab')
 await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
 await page.keyboard.press('Enter')
 await expect(page.getByLabel('Search contacts')).toBeFocused()
 const focused = await state(page)
 await page.keyboard.press('KeyD'); await page.keyboard.press('ArrowLeft')
 const consumed = await page.evaluate(() => {
  const event = new WheelEvent('wheel', { deltaY: -500, bubbles: true, cancelable: true })
  document.querySelector('canvas')!.dispatchEvent(event)
  return event.defaultPrevented
 })
 expect(consumed).toBe(true)
 await page.mouse.move(500, 400); await page.mouse.wheel(0, -500)
 expect((await state(page)).view).toEqual(focused.view)
 expect((await state(page)).ship.yaw).toBe(focused.ship.yaw)
 await page.getByLabel('Search contacts').fill('')
 const ids = new Set<string>()
 for (let i = 0; i < 5; i++) {
  for (const name of await page.locator('#scanned-browser li button').allTextContents()) ids.add(name)
  if (i < 4) await page.getByRole('button', { name: 'Next', exact: true }).click()
 }
 expect(ids.size).toBe(40)
 await page.getByLabel('Search contacts').fill('Contact 39')
 await page.getByRole('button', { name: 'Contact 39', exact: true }).focus(); await page.keyboard.press('Enter')
 await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeEnabled()
 await page.keyboard.press('Escape')
 await expect(page.getByLabel('Search contacts')).toHaveCount(0)
 await expect(page.getByRole('button', { name: /Contacts \(C\)/ })).toBeFocused()
 expect((await state(page)).exits).toBe(0)
 await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
 // Open contacts after telemetry: most recently opened utility closes first.
 await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
 await page.keyboard.press('Escape')
 await expect(page.getByLabel('Search contacts')).toHaveCount(0)
 await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toBeVisible()
 await page.getByRole('button', { name: 'Help', exact: true }).click()
 await expect(page.getByRole('dialog')).toBeVisible()
 await expect(page.getByRole('heading', { name: 'Navigation help', exact: true })).toBeFocused()
 await expect(page.getByRole('heading', { name: 'Navigation help', exact: true })).toBeInViewport()
 await capture(page, 'help.png', info)
 await page.keyboard.press('Escape')
 await expect(page.getByRole('dialog')).toHaveCount(0)
 await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toBeVisible()
 await page.keyboard.press('Escape')
 await expect(page.getByRole('region', { name: 'Detailed telemetry' })).toHaveCount(0)
 expect((await state(page)).exits).toBe(0)
 await page.evaluate(() => (document.activeElement as HTMLElement)?.blur())
 await page.keyboard.press('Escape'); expect((await state(page)).exits).toBe(1)
 await info.attach('browser-state.json', { body: JSON.stringify(await state(page), (_, value) => typeof value === 'bigint' ? String(value) : value), contentType: 'application/json' })
})
test('HUD-2 compass render states, ship rotation, zoom and responsive warning priority', async ({ page }, info) => {
 await boot(page)
 await scenario(page, 'rear')
 await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'true')
 await expect(page.locator('[data-compass=velocity]')).toHaveAttribute('data-rear', 'true')
 await capture(page, 'rear.png', info)
 const rear = await page.locator('.orientation-compass').screenshot()
 expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(true)
 await scenario(page, 'overlap')
 await expect(page.locator('[data-compass=heading]')).toHaveAttribute('data-rear', 'false')
 await capture(page, 'overlap.png', info)
 expect((await page.locator('.orientation-compass').screenshot()).equals(rear)).toBe(false)
 await scenario(page, 'stationary')
 await expect(page.locator('[data-compass=velocity]')).toHaveCount(0)
 await page.locator('canvas').focus()
 const before = await state(page)
 await page.keyboard.down('ArrowLeft')
 await expect.poll(async () => (await state(page)).ship.yaw).not.toBe(before.ship.yaw)
 await page.keyboard.up('ArrowLeft')
 expect((await state(page)).view).toEqual(before.view)
 await page.mouse.move(500, 400); await page.mouse.wheel(0, -1000)
 await expect.poll(async () => (await state(page)).view.fov).toBeLessThan(75)
 await page.getByRole('button', { name: 'Reset view', exact: true }).click()
 await expect.poll(async () => (await state(page)).view.fov).toBe(75)
 await page.getByRole('slider', { name: 'Throttle', exact: true }).fill('30')
 await expect.poll(async () => (await state(page)).ship.throttle).toBe(30)
 await page.getByRole('button', { name: 'Zero throttle (Space)', exact: true }).click()
 expect((await state(page)).ship.throttle).toBe(0)
 await page.setViewportSize({ width: 960, height: 600 })
 await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
 await page.getByRole('button', { name: 'Telemetry', exact: true }).click()
 await scenario(page, 'warning')
 await expect(page.getByLabel('Navigation warnings')).not.toBeEmpty()
 for (const name of ['Reset view', 'Exit cockpit', 'Telemetry']) {
  const control = page.getByRole('button', { name, exact: true })
  await expect(control).toBeInViewport()
  expect(await control.evaluate(el => {
   const r = el.getBoundingClientRect()
   return el.contains(document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2))
  }), `ESSENTIAL_CONTROL_OBSTRUCTED ${name}`).toBe(true)
 }
 await capture(page, 'warning-960x600.png', info)
 await scenario(page, 'destroyed')
 await expect(page.getByText(/Journey ended/)).toBeVisible()
 await expect(page.getByRole('button', { name: 'Restart navigation', exact: true })).toBeVisible()
 expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBe(600)
 await capture(page, 'terminal-960x600.png', info)
 await page.getByRole('button', { name: 'Restart navigation', exact: true }).click()
 await expect(page.getByText(/Journey ended/)).toHaveCount(0)
})
test('HUD-3 grounded authority and no sidebar at smaller desktop size', async ({ page }, info) => {
 await page.setViewportSize({ width: 960, height: 600 }); await boot(page, true)
 await expect(page.getByRole('slider', { name: 'Throttle', exact: true })).toBeDisabled()
 await acquireContacts(page)
 await expect.poll(async () => (await state(page)).contacts.length).toBeGreaterThan(0)
 await page.locator('canvas').focus(); await page.keyboard.press('KeyC')
 await page.getByLabel('Search contacts').fill('Vesper')
 await page.locator('#scanned-browser li button').first().click()
 await expect(page.getByRole('button', { name: 'Look at', exact: true })).toBeDisabled()
 await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toBeDisabled()
 await page.keyboard.press('Escape')
 await capture(page, 'grounded-960x600.png', info)
 expect(await page.locator('aside').count()).toBe(0)
 expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, width: 960, height: 600 })
})

for (const viewport of [{ width: 960, height: 600 }, { width: 1280, height: 720 }, { width: 1920, height: 1080 }]) {
 test(`HUD-STYLE projected controls, contrast and focused help at ${viewport.width}`, async ({ page }, info) => {
  await page.setViewportSize(viewport)
  await page.clock.install({ time: new Date('2026-11-07T12:00:00Z') })
  await page.clock.pauseAt(new Date('2026-11-07T12:00:01Z'))
  const measurements = []
  for (const grounded of [false, true]) {
   await page.goto(`/tests/panoramic.html${grounded ? '?grounded' : ''}`)
   await expect.poll(async () => { await page.clock.runFor(64); return (await state(page))?.renderer }).toContain('SwiftShader')
   await acquireContacts(page); await page.clock.runFor(64)
   expect(await page.locator('canvas').boundingBox()).toMatchObject({ x: 0, y: 0, ...viewport })
   await page.getByRole('button', { name: /Contacts \(C\)/ }).click(); await page.clock.runFor(64)
   await expect(page.getByLabel('Search contacts')).toBeFocused()
   const context = grounded ? 'grounded-horizon' : 'dark-dense'
   await capture(page, `${context}.png`, info)
   const contrast = await page.locator('.panoramic').evaluate(el => {
    const style = getComputedStyle(el)
    const luminance = (hex: string) => {
     const c = hex.trim().slice(1).match(/../g)!.map(v => parseInt(v, 16) / 255).map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4)
     return c[0] * .2126 + c[1] * .7152 + c[2] * .0722
    }
    const bg = luminance(style.getPropertyValue('--ui-void'))
    return Object.fromEntries(['text', 'muted', 'active', 'mechanical', 'danger'].map(name => [name, (luminance(style.getPropertyValue(`--ui-${name}`)) + .05) / (bg + .05)]))
   })
   for (const ratio of Object.values(contrast)) expect(ratio).toBeGreaterThanOrEqual(4.5)
   // Real computed panel surfaces must be opaque; the bright sky cannot change ink contrast.
   expect(await page.locator('.panoramic-status').evaluate(el => getComputedStyle(el).backgroundColor)).toBe('rgb(9, 20, 28)')
   await page.keyboard.press('Escape'); await page.clock.runFor(64)
   await page.getByRole('button', { name: 'Help', exact: true }).click(); await page.clock.runFor(64)
   await expect(page.getByRole('heading', { name: 'Navigation help' })).toBeFocused()
   const before = await state(page)
   await page.keyboard.press('w'); await page.keyboard.press('ArrowLeft'); await page.clock.runFor(64)
   expect((await state(page)).view).toEqual(before.view)
   expect((await state(page)).ship.yaw).toEqual(before.ship.yaw)
   await capture(page, `focused-help-${context}.png`, info)
   await page.keyboard.press('Escape'); await page.clock.runFor(64)
   await expect(page.getByRole('button', { name: 'Help', exact: true })).toBeFocused()
   expect(await page.evaluate(() => ({ w: document.documentElement.scrollWidth, h: document.documentElement.scrollHeight }))).toEqual({ w: viewport.width, h: viewport.height })
   measurements.push({ context, viewport, contrast, state: await state(page) })
  }
  await info.attach('readability.json', { body: JSON.stringify(measurements, (_, v) => typeof v === 'bigint' ? String(v) : v, 2), contentType: 'application/json' })
 })
}

for (const viewport of [{ width: 960, height: 600 }, { width: 1280, height: 720 }, { width: 1920, height: 1080 }]) {
 test(`HUD-BRIGHT warning summaries survive focused help over a bright body at ${viewport.width}`, async ({ page }, info) => {
  await page.setViewportSize(viewport)
  await page.clock.install({ time: new Date('2026-11-07T12:00:00Z') })
  await page.clock.pauseAt(new Date('2026-11-07T12:00:01Z'))
  await page.goto('/tests/panoramic.html?bright')
  await expect.poll(async () => { await page.clock.runFor(64); return (await state(page))?.renderer }).toContain('SwiftShader')
  await acquireContacts(page); await scenario(page, 'warning'); await page.clock.runFor(64)
  await capture(page, 'bright-body-warning.png', info)
  const png = PNG.sync.read(await readFile(info.outputPath('bright-body-warning.png')))
  const offset = (Math.floor(viewport.height / 2) * viewport.width + Math.floor(viewport.width / 2) + 100) * 4
  const background = [...png.data.subarray(offset, offset + 3)]
  expect(Math.max(...background), 'BRIGHT_BODY_MUST_RENDER').toBeGreaterThan(200)
  await info.attach('bright-body.json', { body: JSON.stringify({ viewport, background, state: await state(page) }, (_, v) => typeof v === 'bigint' ? String(v) : v), contentType: 'application/json' })
  await page.getByRole('button', { name: 'Help', exact: true }).click(); await page.clock.runFor(64)
  const dialog = page.getByRole('dialog')
  await expect(dialog.getByText('Low health · critical condition.', { exact: true })).toBeInViewport()
  await expect(dialog.getByText('Contact 00: dangerous approach', { exact: true })).toBeInViewport()
  await expect(page.getByRole('heading', { name: 'Navigation help' })).toBeFocused()
  await capture(page, 'bright-body-focused-warning.png', info)
  await page.keyboard.press('Escape'); await page.clock.runFor(64)
  await expect(page.getByRole('button', { name: 'Help', exact: true })).toBeFocused()
  await expect(page.getByLabel('Navigation warnings')).not.toBeEmpty()
 })
}

test('FEEDBACK scanner pulse is quiet; lost contact removes authoritative actions', async ({ page }, info) => {
 await boot(page)
 await acquireContacts(page)
 await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
 await page.getByRole('button', { name: 'Contact 00', exact:true }).click()
 await expect(page.getByRole('button', { name: 'Look at', exact:true })).toBeVisible()
 const status = page.locator('.panoramic-status [role=status]')
 await expect(status).toHaveText('Scanner active · 40 contacts')
 const text = await status.textContent()
 await acquireContacts(page)
 await expect(status).toHaveText(text!)
 await scenario(page, 'lost-contact'); await acquireContacts(page)
 await expect(page.locator('.selected-contact')).toContainText('Scanner contact lost')
 await expect(page.getByRole('button', { name: 'Look at', exact:true })).toHaveCount(0)
 await expect(page.getByRole('button', { name: 'Set direction', exact:true })).toHaveCount(0)
 await capture(page, 'scanner-contact-lost.png', info)
})

test('ENV-905 seeded scanner space, angular grid, translation and marker contrast across FOV', async ({ page }, info) => {
 test.setTimeout(240000)
 const errors: string[] = []
 page.on('pageerror', error => errors.push(error.message))
 page.on('console', message => { if (message.type() === 'error') errors.push(message.text()) })
 await page.goto('/tests/panoramic.html?environment')
 await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
 const setView = async (fov: number, yaw = 0, pitch = 0) => {
  await page.evaluate(({ fov, yaw, pitch }) => (window as unknown as { hudView(f: number, y: number, p: number): void }).hudView(fov, yaw, pitch), { fov, yaw, pitch })
  await expect.poll(async () => (await state(page)).view.fov).toBe(fov)
  await page.waitForTimeout(200)
 }
 const sky = () => page.locator('canvas').screenshot({ style: '.panoramic > :not(:first-child) { visibility: hidden !important }' })
 const grid = page.getByRole('button', { name: 'Orientation grid', exact: true })
 await expect(grid).toHaveAttribute('aria-pressed', 'false')
 await setView(75)
 const original = await state(page), wide = await sky(), png = PNG.sync.read(wide)
 let bright = 0, black = 0
 for (let i = 0; i < png.data.length; i += 4) {
  if (Math.max(...png.data.subarray(i, i + 3)) > 45) bright++
  if (png.data[i] + png.data[i + 1] + png.data[i + 2] === 0) black++
 }
 expect(black).toBe(0)
 expect(bright).toBeGreaterThan(10); expect(bright).toBeLessThan(png.width * png.height * .005)
 expect(png.data[100 * png.width * 4 + 100 * 4 + 2]).not.toBe(png.data[600 * png.width * 4 + 1100 * 4 + 2])
 await info.attach('wide-canvas.png', { body: wide, contentType: 'image/png' })
 expect((await sky()).equals(wide), 'static repeat').toBe(true)
 await setView(75, 1.1, .5)
 const rotated = await sky()
 expect(rotated.equals(wide), 'orientation must change environment').toBe(false)
 await info.attach('rotated-canvas.png', { body: rotated, contentType: 'image/png' })
 await setView(75)
 expect((await sky()).equals(wide), 'restored orientation').toBe(true)
 await scenario(page, 'lost-contact'); await page.waitForTimeout(200)
 expect((await sky()).equals(wide), 'origin must not translate environment').toBe(true)
 await page.reload()
 await expect.poll(async () => (await state(page))?.renderer).toContain('SwiftShader')
 await setView(75)
 expect((await sky()).equals(wide), 'seeded reload').toBe(true)
 await grid.click(); await page.waitForTimeout(200)
 await expect(grid).toHaveAttribute('aria-pressed', 'true')
 const gridded = await sky()
 expect(gridded.equals(wide), 'grid negative control').toBe(false)
 await info.attach('grid-canvas.png', { body: gridded, contentType: 'image/png' })
 await grid.click(); await page.waitForTimeout(200)
 expect((await sky()).equals(wide), 'grid off restores pixels').toBe(true)
 await acquireContacts(page)
 const linear = (value: number) => { const c = value / 255; return c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4 }
 const luminance = (rgb: number[]) => .2126 * linear(rgb[0]) + .7152 * linear(rgb[1]) + .0722 * linear(rgb[2])
 const measurements = []
 for (const fov of [75, 30, 5, .1]) {
  await setView(fov, 0, Math.atan2(5000, 100000))
  const marker = page.locator('[data-sensor-marker]').first()
  await expect(marker).toBeInViewport()
  const colors = await marker.evaluate(el => { const s = getComputedStyle(el); return [s.color, s.backgroundColor] })
  const channels = colors.map(color => color.match(/[\d.]+/g)!.slice(0, 3).map(Number))
  const contrast = (luminance(channels[0]) + .05) / (luminance(channels[1]) + .05)
  expect(contrast).toBeGreaterThan(4.5)
  const snapshot = await state(page) as State & { calls: number; triangles: number; geometries: number }
  for (const key of ['position', 'velocityMps', 'yaw', 'pitch', 'throttle', 'mode'] as const) expect(snapshot.ship[key], key).toEqual(original.ship[key])
  expect(snapshot.contacts.length).toBe(40)
  expect(snapshot.calls).toBe(1); expect(snapshot.triangles).toBe(2); expect(snapshot.geometries).toBe(1)
  measurements.push({ fov, contrast, snapshot })
  if (fov === 75 || fov === .1) await capture(page, `scanner-${fov}.png`, info)
 }
 const telescope = await sky()
 await grid.click(); await page.waitForTimeout(200)
 expect((await sky()).equals(telescope), 'grid fades out in telescope').toBe(true)
 expect(errors).toEqual([])
 await info.attach('environment-measurements.json', { body: JSON.stringify({ bright, black, original, measurements, errors, comparisons: ['repeat', 'rotation negative', 'restored', 'translation', 'reload', 'grid negative', 'grid restoration', 'telescope fade'] }, (_, v) => typeof v === 'bigint' ? String(v) : v, 2), contentType: 'application/json' })
})

for (const viewport of [{ width: 960, height: 600 }, { width: 1280, height: 720 }, { width: 1920, height: 1080 }]) {
 test(`HUD-907 edge panels reserve space at ${viewport.width}`, async ({ page }, info) => {
  await page.setViewportSize(viewport)
  await page.clock.install({ time: new Date('2026-11-25T12:00:00Z') })
  await page.clock.pauseAt(new Date('2026-11-25T12:00:01Z'))
  await page.goto('/tests/panoramic.html')
  await expect.poll(async () => { await page.clock.runFor(64); return (await state(page))?.renderer }).toContain('SwiftShader')
  await expect(page.locator('.selected-contact')).toHaveCount(0)
  await expect(page.locator('.scanner-readings')).toContainText('Range 100,000 ls · FOV 75.0°')
  await capture(page, 'default.png', info)
  await acquireContacts(page); await page.clock.runFor(64)
  // Full-viewport projection still supports the existing mouse selection path.
  await page.locator('[data-sensor-marker]').first().click(); await page.clock.runFor(64)
  if (await page.getByLabel('Search contacts').count()) {
   await page.getByLabel('Search contacts').fill('Contact 00')
   await page.getByRole('button', { name: 'Contact 00', exact:true }).click(); await page.clock.runFor(64)
   await page.keyboard.press('Escape'); await page.clock.runFor(64)
  }
  await expect(page.locator('.selected-contact')).toContainText('Selected:')
  await expect(page.getByRole('button', { name: 'Look at', exact:true })).toBeEnabled()
  await capture(page, 'selected.png', info)
  expect((await page.screenshot()).equals(await page.screenshot()), 'HUD_REPEAT_CAPTURE').toBe(true)
  await page.getByRole('button', { name: 'Telemetry', exact:true }).click(); await page.clock.runFor(64)
  await scenario(page, 'warning'); await page.clock.runFor(64)
  const panels = '.panoramic-status, .panoramic-utilities, .panoramic-contacts, .panoramic-instruments, .panoramic-propulsion, .panoramic-systems, .panoramic-telemetry, .panoramic-priority'
  const measure = () => page.locator(panels).evaluateAll(elements => {
   const rects = elements.filter(el => el.getBoundingClientRect().height > 0).map(el => ({ name:el.className, ...Object.fromEntries(['x','y','width','height','right','bottom'].map(k => [k, el.getBoundingClientRect()[k as keyof DOMRect]])) })) as {name:string;x:number;y:number;width:number;height:number;right:number;bottom:number}[]
   const overlaps = rects.flatMap((a,i) => rects.slice(i+1).filter(b => Math.min(a.right,b.right)-Math.max(a.x,b.x)>1 && Math.min(a.bottom,b.bottom)-Math.max(a.y,b.y)>1).map(b => `${a.name} / ${b.name}`))
   return {rects, overlaps}
  })
  const measurements = await measure()
  expect(measurements.overlaps).toEqual([])
  for (const r of measurements.rects) { expect(r.x).toBeGreaterThanOrEqual(0); expect(r.y).toBeGreaterThanOrEqual(0); expect(r.right).toBeLessThanOrEqual(viewport.width); expect(r.bottom).toBeLessThanOrEqual(viewport.height) }
  // Check actual hit targets and shared focus ink, not just panel rectangles.
  for (const name of ['Help', 'Exit cockpit', 'Reset view', 'Telemetry', 'Look at', 'Zero throttle (Space)']) {
   const button = page.getByRole('button', {name, exact:true})
   expect(await button.evaluate(el => { const r=el.getBoundingClientRect(); return el.contains(document.elementFromPoint(r.x+r.width/2,r.y+r.height/2)) }), name).toBe(true)
  }
  await page.getByRole('button', { name:'Exit cockpit', exact:true }).focus()
  await page.keyboard.press('Shift+Tab')
  await expect(page.getByRole('button', { name:'Help', exact:true })).toBeFocused()
  expect(await page.getByRole('button', { name:'Help', exact:true }).evaluate(el => getComputedStyle(el).outlineStyle)).toBe('solid')
  await info.attach('layout.json', {body:JSON.stringify({viewport, measurements, state:await state(page)},(_,v)=>typeof v==='bigint'?String(v):v), contentType:'application/json'})
  await capture(page, 'warning-telemetry.png', info)
  // Negative: placing the compass over status must be detected by this oracle.
  await page.locator('.panoramic-instruments').evaluate(el => { (el as HTMLElement).style.cssText='position:absolute;grid-area:auto;left:12px;top:12px' })
  expect((await measure()).overlaps.length, 'OVERLAP_MUTATION_NOT_DETECTED').toBeGreaterThan(0)
  await page.locator('.panoramic-instruments').evaluate(el => el.removeAttribute('style'))
  expect((await measure()).overlaps).toEqual([])
  expect(await page.getByRole('button', {name:/^(Zoom in|Zoom out|Left|Right|Up|Down)$/}).count()).toBe(0)
 })
}
