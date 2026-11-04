import { test, expect, type Page } from '@playwright/test'
import { PNG } from 'pngjs'

const scene = (page: Page) => page.getByRole('img', { name: 'Strict overhead Scout and planetary test site' })
const transform = (page: Page, id: string) => page.locator(`[data-entity="${id}"]`).getAttribute('transform')
async function boot(page: Page) {
  await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') })
  await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  await page.goto('/?scene=top-down')
  // Lazy development module can resolve under React's delayed Suspense commit.
  await expect.poll(async () => { await page.clock.runFor(50); return scene(page).count() }).toBe(1)
  await expect(scene(page)).toBeVisible()
  await page.evaluate(() => document.fonts.ready)
  await page.clock.runFor(32)
}

test('TD-1/4 one scene, stable layers, visible interior and identical fresh-run baseline captures', async ({ page }, info) => {
  const errors: string[] = []; page.on('pageerror', error => errors.push(error.message))
  const captures: Buffer[] = [], observations = []
  for (let replay = 0; replay < 2; replay++) {
    await boot(page)
    await expect(page.locator('canvas')).toHaveCount(0)
    const layers = await scene(page).locator(':scope > g').evaluateAll(nodes => nodes.map(n => n.getAttribute('data-layer')))
    expect(layers).toEqual(['terrain', 'hull', 'interior', 'people', 'props', 'cues'])
    await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
    await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'ship:scout')
    expect(await scene(page).locator('[data-entity]').count()).toBeGreaterThan(10)
    const original = await scene(page).screenshot({ path: info.outputPath(`top-down-baseline-${replay}.png`), animations: 'allow' })
    captures.push(original)
    const png = PNG.sync.read(original), counts = { floor: 0, player: 0, rocks: 0 }
    for (let i = 0; i < png.data.length; i += 4) {
      const [r, g, b] = png.data.subarray(i, i + 3)
      if (r === 223 && g === 217 && b === 198) counts.floor++
      if (r === 226 && g === 216 && b === 185) counts.player++
      if (r === 130 && g === 127 && b === 115) counts.rocks++
    }
    expect(counts.floor).toBeGreaterThan(1000); expect(counts.player).toBeGreaterThan(50); expect(counts.rocks).toBeGreaterThan(500)
    observations.push({ layers, counts, player: await transform(page, 'person:player'), viewport: page.viewportSize(), browser: page.context().browser()!.version(), dpr: await page.evaluate(() => devicePixelRatio) })
  }
  expect(captures[1]).toEqual(captures[0])
  expect(errors).toEqual([])
  await info.attach('top-down-baseline.png', { body: captures[0], contentType: 'image/png' })
  await info.attach('top-down-observations.json', { body: JSON.stringify({ observations, errors, clock: '2026-11-04T00:00:01Z; 32ms', input: 'Playwright synthetic', renderer: 'Chromium SVG', status: 'new deterministic capture; no approved pixel reference promoted' }, null, 2), contentType: 'application/json' })
})

const position = async (page: Page) => (await page.locator('[data-entity="person:player"]').getAttribute('data-position'))!.split(',').map(Number)
async function walk(page: Page, keys: string[], ms: number) {
  for (const key of keys) await page.keyboard.down(key)
  await page.clock.runFor(ms)
  for (const key of keys) await page.keyboard.up(key)
}
// Steer through the real keyboard adapter; DOM attributes are observations only.
async function walkTo(page: Page, target: number[]) {
  for (let i = 0; i < 140; i++) {
    const p = await position(page), dx = target[0] - p[0], dy = target[1] - p[1]
    if (Math.hypot(dx, dy) < .075) return
    const angle = -.12, x = Math.cos(angle) * dx - Math.sin(angle) * dy, y = Math.sin(angle) * dx + Math.cos(angle) * dy
    const keys = [...(Math.abs(x) > .035 ? [x > 0 ? 'd' : 'a'] : []), ...(Math.abs(y) > .035 ? [y > 0 ? 's' : 'w'] : [])]
    await walk(page, keys, Math.max(17, Math.min(65, Math.hypot(dx, dy) / 2.4 * 1000)))
  }
  throw new Error(`Could not walk to ${target}; stopped at ${await position(page)}`)
}
test('TD-2 release/blur/hidden/resize/focus clear movement while camera settles smoothly', async ({ page }, info) => {
  await boot(page)
  const before = await position(page), player = await transform(page, 'person:player')
  await walk(page, ['w'], 160)
  const walked = await position(page)
  expect(walked[1]).toBeLessThan(before[1]); expect(await transform(page, 'person:player')).not.toBe(player)
  const facing = await page.locator('[data-entity="person:player"]').getAttribute('data-facing')
  await page.clock.runFor(160); expect(await position(page)).toEqual(walked)
  expect(await page.locator('[data-entity="person:player"]').getAttribute('data-facing')).toBe(facing)
  for (const event of ['blur', 'resize', 'visibilitychange', 'focusin']) {
    await page.keyboard.down('w'); await page.clock.runFor(48)
    await page.evaluate(name => (name === 'visibilitychange' ? document : window).dispatchEvent(new Event(name)), event)
    const stopped = await position(page)
    await page.clock.runFor(160); expect(await position(page), event).toEqual(stopped)
    await page.keyboard.up('w')
  }
  await page.evaluate(() => { const input = document.createElement('input'); document.body.append(input); input.focus() })
  const focused = await position(page)
  await walk(page, ['d'], 160)
  await page.keyboard.press('e')
  expect(await position(page)).toEqual(focused)
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  await info.attach('walk.json', { body: JSON.stringify({ before, walked, facing }), contentType: 'application/json' })
})

test('TD-3 collisions, keyboard detour, selection and request/cancel use the continuous scene', async ({ page }, info) => {
  test.setTimeout(90000)
  await boot(page)
  await page.keyboard.press('e')
  await expect(page.getByRole('status')).toContainText('No available object in reach')
  await walk(page, ['s'], 800)
  expect((await position(page))[1]).toBeCloseTo(2.12, 6)
  await walkTo(page, [0, .45])
  await walk(page, ['w'], 700)
  expect((await position(page))[1]).toBeCloseTo(.36, 6)
  await scene(page).screenshot({ path: info.outputPath('crate-collision.png') })
  await walkTo(page, [1.35, .36])
  await expect(page.locator('[data-selection="scout:core"]')).toHaveCount(1)
  await expect(page.locator('[data-prompt]')).toContainText('Core test console')
  await page.keyboard.press('e')
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'interacting')
  await expect(page.getByRole('status')).toContainText('No systems changed')
  await page.screenshot({ path: info.outputPath('core-inspection.png') })
  await page.keyboard.press('Escape')
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  await page.keyboard.press('e')
  await page.evaluate(() => window.dispatchEvent(new Event('blur')))
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  await walkTo(page, [1.35, -2])
  await walkTo(page, [0, -2])
  await walkTo(page, [0, -3.25])
  await expect(page.locator('[data-selection="scout:desk"]')).toHaveCount(1)
  await page.getByRole('button', { name: 'Inspect console' }).click()
  await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:desk')
  await page.getByRole('button', { name: 'Cancel inspection' }).click()
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  await page.screenshot({ path: info.outputPath('cockpit-reached.png') })
  expect((await position(page))[1]).toBeLessThan(-3)
  await info.attach('journey.json', { body: JSON.stringify({ final: await position(page), input: 'actual Playwright keyboard/mouse; no state setters', targets: ['scout:core', 'scout:desk'] }), contentType: 'application/json' })
})

test('TD-5 built entry retains navigation even with development scene query', async ({ page }) => {
  await page.goto('http://127.0.0.1:4175/?scene=top-down')
  await expect(page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })).toBeVisible()
  await expect(scene(page)).toHaveCount(0)
})
