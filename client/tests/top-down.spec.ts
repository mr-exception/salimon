import { test, expect, type Page } from '@playwright/test'
import { PNG } from 'pngjs'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { convertAnchor, point2, type FrameId } from '../src/topDownFrames.ts'
import { createTopDownState } from '../src/topDownRuntime.ts'

const scene = (page: Page) => page.getByRole('img', { name: 'Strict overhead Scout and planetary test site' })
const transform = (page: Page, id: string) => page.locator(`[data-entity="${id}"]`).getAttribute('transform')
async function boot(page: Page, view = 'interior') {
  await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') })
  await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  await page.goto(`/?scene=top-down&view=${view}`)
  // Lazy development module can resolve under React's delayed Suspense commit.
  await expect.poll(async () => { await page.clock.runFor(50); return scene(page).count() }).toBe(1)
  await expect(scene(page)).toBeVisible()
  await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
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
    const box = (await scene(page).boundingBox())!
    for (const value of [box.x, box.y, box.width, box.height]) expect(value).toBe(Math.round(value))
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
async function positionIn(page: Page, frame: FrameId) {
  const p = await position(page)
  const current = await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  return convertAnchor({ frame: current, point: point2(p[0], p[1]) }, frame, createTopDownState().frames).point.offsetM.slice(0, 2)
}
async function walkTo(page: Page, target: number[], frame?: FrameId, exactTicks = false) {
  frame ??= await page.locator('[data-entity="person:player"]').getAttribute('data-frame') as FrameId
  for (let i = 0; i < 140; i++) {
    const p = await positionIn(page, frame), dx = target[0] - p[0], dy = target[1] - p[1]
    if (Math.hypot(dx, dy) < .075) return
    const angle = frame === 'ship:scout' ? -.12 : 0, x = Math.cos(angle) * dx - Math.sin(angle) * dy, y = Math.sin(angle) * dx + Math.cos(angle) * dy
    const keys = [...(Math.abs(x) > .035 ? [x > 0 ? 'd' : 'a'] : []), ...(Math.abs(y) > .035 ? [y > 0 ? 's' : 'w'] : [])]
    if (exactTicks) await walkTicks(page, keys, Math.max(1, Math.min(3, Math.floor(Math.hypot(dx, dy) / .04))))
    else await walk(page, keys, Math.max(17, Math.min(65, Math.hypot(dx, dy) / 2.4 * 1000)))
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
  await walkTo(page, [0, .45])
  await walk(page, ['w'], 700)
  expect((await position(page))[1]).toBeCloseTo(.36, 6)
  await scene(page).screenshot({ path: info.outputPath('crate-collision.png') })
  await walkTo(page, [1.35, .36])
  await expect(page.locator('[data-selection="scout:core"]')).toHaveCount(1)
  await expect(page.locator('[data-prompt]')).toContainText('Core')
  await page.keyboard.press('e')
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'interacting')
  await expect(page.getByRole('status')).toContainText('No systems changed')
  await page.screenshot({ path: info.outputPath('core-inspection.png') })
  await page.keyboard.press('Escape')
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  await page.keyboard.press('e')
  await page.evaluate(() => window.dispatchEvent(new Event('blur')))
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  await walkTo(page, [1.35, -1.65])
  await walkTo(page, [0, -1.65])
  await walkTo(page, [0, -3.1])
  await expect(page.locator('[data-selection="scout:chair"]')).toHaveCount(1)
  await page.getByRole('button', { name: 'Inspect fixture' }).click()
  await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:chair')
  await page.getByRole('button', { name: 'Cancel inspection' }).click()
  await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
  await walkTo(page, [1.1, -3.1])
  await walkTo(page, [1.1, -3.9])
  await expect(page.locator('[data-selection="scout:desk"]')).toHaveCount(1)
  await page.keyboard.press('e')
  await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:desk')
  await page.screenshot({ path: info.outputPath('cockpit-reached.png') })
  expect((await position(page))[1]).toBeLessThan(-3)
  await info.attach('journey.json', { body: JSON.stringify({ final: await position(page), input: 'actual Playwright keyboard/mouse; no state setters', targets: ['scout:core', 'scout:desk'] }), contentType: 'application/json' })
})

test('TD-5 explicit built radar entry retains standalone navigation', async ({ page }, info) => {
  await page.goto('http://127.0.0.1:4175/?scene=radar')
  await expect(page.getByRole('img', { name: 'Ship-centered three-dimensional radar', exact: true })).toBeVisible()
  await expect(scene(page)).toHaveCount(0)
  await page.screenshot({ path: info.outputPath('radar-preflight.png') })
  const backend = await page.locator('canvas').evaluate(canvas => {
    const gl = (canvas as HTMLCanvasElement).getContext('webgl2')!
    const ext = gl.getExtension('WEBGL_debug_renderer_info')
    return { renderer: ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER), version: gl.getParameter(gl.VERSION) }
  })
  await info.attach('backend.json', { body: JSON.stringify(backend), contentType: 'application/json' })
})


test('SCOUT-1 wide cargo route, module anchors and exposed exterior cutaway', async ({ page }, info) => {
  test.setTimeout(90000)
  await boot(page)
  const originalNode = await page.locator('[data-entity="ship:scout"]').elementHandle()
  await walkTo(page, [0, 3.9])
  await expect(page.locator('[data-selection="scout:fabricator"]')).toHaveCount(1)
  await page.keyboard.press('e')
  await expect(page.getByRole('status')).toContainText('Fabricator')
  await page.keyboard.press('Escape')
  await walkTo(page, [.65, 4.25])
  await expect(page.locator('[data-selection="scout:storage"]')).toHaveCount(1)
  await page.keyboard.press('e')
  await expect(page.getByRole('status')).toContainText('50 kg / 50 L')
  await page.screenshot({ path: info.outputPath('cargo-modules.png') })
  await page.keyboard.press('Escape')
  await walkTo(page, [0, 4.8])
  await walkTo(page, [0, 5.8])
  expect(await originalNode!.evaluate(node => node.isConnected)).toBe(true)
  await scene(page).screenshot({ path: info.outputPath('entry-cargo.png') })
  // Fresh exterior starting viewpoint, not a teleport or scene switch in play.
  await boot(page, 'exterior')
  await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
  for (const id of ['scout:floor', 'scout:desk', 'scout:chair', 'scout:core', 'scout:fabricator', 'scout:storage', 'scout:entry']) {
    await expect(page.locator(`[data-entity="${id}"]`)).toBeVisible()
  }
  const outside = await scene(page).screenshot({ path: info.outputPath('exterior-cutaway.png') })
  const png = PNG.sync.read(outside)
  let floorPixels = 0
  for (let i = 0; i < png.data.length; i += 4) if (png.data[i] === 223 && png.data[i + 1] === 217 && png.data[i + 2] === 198) floorPixels++
  expect(floorPixels).toBeGreaterThan(1000)
  await info.attach('exterior-cutaway.png', { body: outside, contentType: 'image/png' })
})


// Hold actual keyboard input for an exact observed number of fixed steps.
// Read-only tick observations control time, never gameplay state or rendering.
async function walkTicks(page: Page, key: string | string[], ticks: number) {
  const tick = async () => Number(await page.locator('main').getAttribute('data-tick'))
  const target = await tick() + ticks
  const keys = typeof key === 'string' ? [key] : key
  for (const code of keys) await page.keyboard.down(code)
  await page.clock.runFor(Math.max(0, ticks * 1000 / 60 - 25))
  // A 16 ms RAF interval advances at most one 60 Hz simulation tick.
  for (let i = 0; i < 60 && await tick() < target; i++) await page.clock.runFor(16)
  for (const code of keys) await page.keyboard.up(code)
  expect(await tick()).toBe(target)
}

test('PLANET-1 fixed outdoor journey, source footprints and repeat screenshots', async ({ browser }, info) => {
  test.setTimeout(120000)
  const captures: Record<string, Buffer> = {}
  const sourceSnapshots: string[][] = []
  for (let replay = 0; replay < 2; replay++) {
    const context = await browser.newContext({ baseURL: 'http://127.0.0.1:5175', viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
    const page = await context.newPage()
    try {
      // Empty view exercises the default development launch, not a test-only spawn.
      await boot(page, '')
      await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
      expect(await position(page)).toEqual([-6, 3])
      const originalNode = await page.locator('[data-entity="ship:scout"]').elementHandle()
      const sources = await page.locator('[data-source-kind]').evaluateAll(nodes => nodes.map(n => `${n.getAttribute('data-entity')}/${n.getAttribute('data-source-kind')}/${n.getAttribute('data-frame')}/${n.getAttribute('data-position')}/${n.getAttribute('data-source-site')}`))
      sourceSnapshots.push(sources)
      expect(sources).toHaveLength(2)
      const capture = async (name: string) => {
        await page.clock.runFor(5000)
        const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
        await info.attach(`${name}-${replay}.json`, { body: JSON.stringify({ position: await position(page), tick: await page.locator('main').getAttribute('data-tick'), ground: await transform(page, 'terrain:site'), player: await transform(page, 'person:player'), viewport: page.viewportSize(), sceneBounds: await scene(page).boundingBox() }), contentType: 'application/json' })
        if (replay === 0) captures[name] = png
        else {
          const comparison = compareImages(captures[name]!, png)
          await info.attach(`${name}-comparison.json`, { body: JSON.stringify({ policy: POLICY, changedPixels: comparison.changedPixels, rawChangedPixels: comparison.rawChangedPixels, totalPixels: comparison.totalPixels }), contentType: 'application/json' })
          await info.attach(`${name}-diff.png`, { body: comparison.diff, contentType: 'image/png' })
          expect(comparison.pass, `${name} uses unchanged repository visual policy`).toBe(true)
        }
      }
      await capture('planet-spawn')
      await page.keyboard.press('e')
      await expect(page.getByRole('status')).toContainText('No available object in reach')
      await walkTicks(page, 'w', 75)
      await walkTicks(page, 'a', 100)
      expect((await position(page))[0]).toBeCloseTo(-7.34, 6)
      await expect(page.locator('[data-selection="source:landing:mineral:01"]')).toHaveCount(1)
      await page.keyboard.press('e')
      await expect(page.getByRole('status')).toContainText('Banded mineral outcrop')
      await capture('planet-near-rock')
      await page.keyboard.press('e')
      await page.keyboard.press('Escape')
      await walkTicks(page, 'd', 34)
      await walkTicks(page, 's', 110)
      await walkTicks(page, 'd', 25)
      await expect(page.locator('[data-selection="source:landing:biology:01"]')).toHaveCount(1)
      await page.keyboard.press('e')
      await expect(page.getByRole('status')).toContainText('Alien plate colony')
      await walkTicks(page, 's', 50)
      expect((await position(page))[1]).toBeCloseTo(4.69, 6)
      await capture('planet-near-colony')
      await walkTicks(page, 'd', 38)
      await walkTicks(page, 'w', 117)
      await walkTicks(page, 'd', 24)
      await capture('planet-near-scout')
      expect(await originalNode!.evaluate(node => node.isConnected)).toBe(true)
      expect(await page.locator('[data-source-kind]').evaluateAll(nodes => nodes.map(n => `${n.getAttribute('data-entity')}/${n.getAttribute('data-source-kind')}/${n.getAttribute('data-frame')}/${n.getAttribute('data-position')}/${n.getAttribute('data-source-site')}`))).toEqual(sources)
    } finally { await context.close() }
  }
  expect(sourceSnapshots[1]).toEqual(sourceSnapshots[0])
  const wrongView = compareImages(captures['planet-spawn']!, captures['planet-near-rock']!)
  expect(wrongView.pass).toBe(false)
  expect(wrongView.changedPixels).toBeGreaterThan(100)
  await info.attach('wrong-checkpoint-negative.json', { body: JSON.stringify({ policy: POLICY, pass: wrongView.pass, changedPixels: wrongView.changedPixels, rawChangedPixels: wrongView.rawChangedPixels }), contentType: 'application/json' })
  await info.attach('planet-sources.json', { body: JSON.stringify({ sources: sourceSnapshots, input: 'Playwright keyboard; no state setters', site: 'fixed fictional living-world sample v1', comparisons: Object.keys(captures), collection: 'not implemented; inspections only' }, null, 2), contentType: 'application/json' })
})

test('TRAVERSE-1 outdoor spawn to cockpit and back preserves frames, camera and one visible world', async ({ browser }, info) => {
  test.setTimeout(180000)
  const captures: Record<string, Buffer> = {}, journeys = []
  for (let replay = 0; replay < 2; replay++) {
    const context = await browser.newContext({ baseURL: 'http://127.0.0.1:5175', viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
    const page = await context.newPage()
    const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
    try {
      await boot(page, '')
      const go = (target: number[], frame: FrameId) => walkTo(page, target, frame, true)
      const world = await scene(page).elementHandle()
      const ids = await scene(page).locator('[data-entity]').evaluateAll(nodes => nodes.map(n => n.getAttribute('data-entity')))
      const capture = async (name: string) => {
        await page.clock.runFor(5000)
        for (const id of ['scout:floor', 'scout:desk', 'scout:core', 'source:landing:biology:01']) {
          const box = await page.locator(`[data-entity="${id}"]`).boundingBox(), bounds = (await scene(page).boundingBox())!
          expect(box).not.toBeNull()
          expect(box!.x + box!.width).toBeGreaterThan(bounds.x); expect(box!.x).toBeLessThan(bounds.x + bounds.width)
          expect(box!.y + box!.height).toBeGreaterThan(bounds.y); expect(box!.y).toBeLessThan(bounds.y + bounds.height)
        }
        await info.attach(`${name}-${replay}-state.json`, { body: JSON.stringify({ position: await position(page), planet: await positionIn(page, 'planet:fixture'), frame: await page.locator('[data-entity="person:player"]').getAttribute('data-frame'), transform: await transform(page, 'terrain:site') }), contentType: 'application/json' })
        const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
        if (replay === 0) captures[name] = png
        else {
          const result = compareImages(captures[name]!, png)
          await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: result.pass, changedPixels: result.changedPixels, rawChangedPixels: result.rawChangedPixels, policy: POLICY }), contentType: 'application/json' })
          await info.attach(`${name}-diff.png`, { body: result.diff, contentType: 'image/png' })
          expect(result.pass).toBe(true)
        }
      }
      const frame = () => page.locator('[data-entity="person:player"]').getAttribute('data-frame')
      const transitions: unknown[][] = []
      const sampleCrossing = async (key: string, expected: string) => {
        const samples = []
        for (let i = 0; i < 14; i++) {
          const before = await positionIn(page, 'planet:fixture'), previous = await frame()
          const oldBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
          await walkTicks(page, key, 1)
          const after = await positionIn(page, 'planet:fixture'), current = await frame()
          const newBox = (await page.locator('[data-entity="person:player"]').boundingBox())!
          expect(Math.hypot(after[0] - before[0], after[1] - before[1])).toBeCloseTo(.04, 7)
          expect(Math.hypot(newBox.x - oldBox.x, newBox.y - oldBox.y)).toBeLessThan(3)
          samples.push({ before, after, previous, current, oldBox, newBox })
        }
        expect(await frame()).toBe(expected)
        expect(samples.filter(s => s.previous !== s.current)).toHaveLength(1)
        transitions.push(samples)
      }
      await go([-2, 3], 'planet:fixture')
      await go([-2, 7], 'planet:fixture')
      await go([0, 7], 'ship:scout')
      // Walk into an aft jamb. The real hull sweep blocks entry.
      await go([2, 7], 'ship:scout')
      await walkTicks(page, 'w', 42)
      expect(await frame()).toBe('planet:fixture')
      expect((await positionIn(page, 'ship:scout'))[1]).toBeGreaterThan(6)
      await go([0, 7], 'ship:scout')
      await go([0, 5.65], 'ship:scout')
      await capture('door-outside')
      await sampleCrossing('w', 'ship:scout')
      await capture('door-inside')
      await go([0, 2], 'ship:scout')
      for (const target of [[0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await go(target, 'ship:scout')
      await expect(page.locator('[data-selection="scout:chair"]')).toHaveCount(1)
      await page.keyboard.press('e')
      await expect(page.locator('main')).toHaveAttribute('data-target', 'scout:chair')
      await page.keyboard.press('Escape')
      await capture('cockpit-round-trip')
      for (const target of [[0, -1.65], [1.35, -1.65], [1.35, .45], [0, .45], [0, 2], [0, 5.3]]) await go(target, 'ship:scout')
      await sampleCrossing('s', 'planet:fixture')
      await go([0, 7], 'ship:scout')
      await capture('returned-outdoors')
      expect(await world!.evaluate(node => node.isConnected)).toBe(true)
      expect(await scene(page).locator('[data-entity]').evaluateAll(nodes => nodes.map(n => n.getAttribute('data-entity')))).toEqual(ids)
      expect(errors).toEqual([])
      journeys.push({ replay, transitions, final: await position(page), frame: await frame(), ids, errors, browser: context.browser()!.version(), input: 'Playwright keyboard through production adapter', renderer: 'Chromium SVG CPU rasterization', viewport: page.viewportSize(), dpr: await page.evaluate(() => devicePixelRatio) })
    } finally { await context.close() }
  }
  const negative = compareImages(captures['door-outside']!, captures['cockpit-round-trip']!)
  expect(negative.pass).toBe(false)
  await info.attach('traversal-journeys.json', { body: JSON.stringify({ journeys, wrongCheckpoint: { pass: negative.pass, changedPixels: negative.changedPixels, rawChangedPixels: negative.rawChangedPixels }, clock: '2026-11-04T00:00:01Z fixed steps', baseline: 'repeat evidence only; no approved references changed' }, null, 2), contentType: 'application/json' })
})
