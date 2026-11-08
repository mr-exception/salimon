import { test, expect, type Page, type TestInfo } from '@playwright/test'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { execFileSync } from 'node:child_process'
import { isAbsolute, resolve } from 'node:path'
import { PNG } from 'pngjs'
import { compareImages, sha256, POLICY } from '../scripts/visual-comparison.ts'
import { compareUIReference, type UIManifest } from '../scripts/ui-reference.ts'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { resourceInventoryUsage } from '../src/resourceInventory.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { point2 } from '../src/topDownFrames.ts'
import { SCOUT_RESOURCE_CONTAINER_ID } from '../src/scoutResourceContainer.ts'
import { boot, position, walkTicks } from './worldJourney'

const directory = 'tests/expedition-baselines'
const candidate = process.env.SALIMON_UI_CANDIDATES
if (candidate && (process.env.CI || !isAbsolute(candidate) || !resolve(candidate).startsWith(resolve('test-results') + '/'))) {
  throw Error('UI candidates require an absolute ignored client/test-results directory outside CI')
}
const sizes = [{ width: 960, height: 600 }, { width: 1280, height: 720 }, { width: 1920, height: 1080 }]
const epoch = '2026-11-04T00:00:00Z'
const json = (value: unknown) => JSON.stringify(value, (_, v) => typeof v === 'bigint' ? String(v) : v, 2) + '\n'

async function recorder(page: Page, info: TestInfo) {
  const source = execFileSync('git', ['rev-parse', 'HEAD'], { encoding: 'utf8' }).trim()
  const inputs = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', 'src', 'tests', 'scripts', 'package.json', 'package-lock.json', 'playwright.config.ts'], { encoding: 'utf8' }).trim().split('\n').filter(p => !p.startsWith('tests/expedition-baselines/'))
  const sourceInputs = sha256((await Promise.all(inputs.map(async p => `${p}:${sha256(await readFile(p))}`))).join('\n'))
  const environment = { browser: page.context().browser()!.version(), playwright: JSON.parse(await readFile('node_modules/@playwright/test/package.json', 'utf8')).version,
    platform: process.platform, arch: process.arch, viewport: page.viewportSize(), dpr: 1, locale: 'en-US', timezone: 'UTC', reducedMotion: 'reduce', epoch,
    fonts: Object.fromEntries(await Promise.all(['DejaVuSans.ttf', 'DejaVuSans-Bold.ttf', 'fonts.conf', 'fonts.css'].map(async f => [f, sha256(await readFile(`tests/assets/${f}`))]))), policy: POLICY }
  const manifestPath = `${directory}/${page.viewportSize()!.width}.json`
  const manifest: UIManifest = candidate ? { schema: 1, review: 'PENDING_OWNER_REVIEW', environment, images: {} } : JSON.parse(await readFile(manifestPath, 'utf8'))
  expect(manifest.schema).toBe(1)
  expect(environment, 'UI_ENVIRONMENT_DRIFT: propose reviewed references separately').toEqual(manifest.environment)
  const output = info.outputPath('comparisons'); await mkdir(output, { recursive: true })
  const records: unknown[] = []
  const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
  return {
    async capture(id: string, preservePointer = false) {
      await page.evaluate(async () => {
        await document.fonts.ready
        await Promise.all([...document.querySelectorAll('svg image')].map(async el => {
          const image = new Image(); image.src = el.getAttribute('href')!; await image.decode()
        }))
      })
      if (!preservePointer) await page.mouse.move(0, 0)
      const name = `${id}-${page.viewportSize()!.width}.png`
      if (process.env.SALIMON_UI_MUTATION === 'visual' && id === 'gallery-planet-default') {
        await page.addStyleTag({ content: '.ui-panel { background: #ff0000 !important }' })
      }
      const actual = await page.screenshot({ caret: 'hide', animations: 'disabled' })
      const repeated = await page.screenshot({ caret: 'hide', animations: 'disabled' })
      const repeat = compareImages(actual, repeated)
      await writeFile(`${output}/${name}`, actual)
      await writeFile(`${output}/${id}-repeat.png`, repeated)
      await writeFile(`${output}/${id}-repeat-diff.png`, repeat.diff)
      expect(repeat.pass, `UI_REPEAT ${id}`).toBe(true)
      const snapshot = await page.evaluate(() => ({
        scroll: { width: document.documentElement.scrollWidth, height: document.documentElement.scrollHeight }, viewport: { width: innerWidth, height: innerHeight }, dpr: devicePixelRatio,
        meters: [...document.querySelectorAll('meter,progress')].map(el => ({ label: el.getAttribute('aria-label'), value: el.getAttribute('value'), max: el.getAttribute('max'), text: el.getAttribute('aria-valuetext') })),
        world: (() => { const main = document.querySelector('main[data-ship]'); return main ? { ship: JSON.parse(main.getAttribute('data-ship')!), mode: main.getAttribute('data-player-mode'), tick: main.getAttribute('data-tick') } : null })(),
        geometry: [...document.querySelectorAll('.survival-hud,.resource-hud,.top-down-bottom,.panoramic-status,.panoramic-controls,.panoramic-instruments,.ui-dialog[open],.panoramic-contacts')].filter(el => el.getClientRects().length).map(el => {
          const r = el.getBoundingClientRect(); return { name: el.className, x: r.x, y: r.y, width: r.width, height: r.height, overflowX: el.scrollWidth > el.clientWidth }
        }),
        worldHit: (() => { const el = document.querySelector('[data-entity="person:player"]'); if (!el?.getClientRects().length || document.querySelector('dialog[open]')) return null; const r = el.getBoundingClientRect(); return !!document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2)?.closest('.top-down-scene') })(),
        focus: document.activeElement?.outerHTML.slice(0, 500),
        backend: (() => { const canvas = document.querySelector('canvas'); if (!canvas) return 'Chromium SVG/DOM'; const gl = canvas.getContext('webgl2')!, ext = gl.getExtension('WEBGL_debug_renderer_info')!; return gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) })(),
      }))
      const bright = id === 'cockpit-bright-warning' ? (() => { const png = PNG.sync.read(actual), offset = (Math.floor(png.height / 2) * png.width + Math.floor(png.width / 2) + 100) * 4; return [...png.data.subarray(offset, offset + 3)] })() : null
      const result = { id, source, sourceInputs, environment, snapshot, bright, errors, repeat: { ...repeat, diff: undefined }, actual: sha256(actual), candidate: !!candidate }
      await writeFile(`${output}/${id}.json`, json(result))
      expect(snapshot.scroll, `UI_BODY_SCROLL ${id}`).toEqual(snapshot.viewport)
      expect(snapshot.dpr).toBe(1)
      if (bright) expect(Math.max(...bright), 'UI_BRIGHT_BODY_RENDERED').toBeGreaterThan(200)
      if (snapshot.backend !== 'Chromium SVG/DOM') expect(snapshot.backend).toContain('SwiftShader')
      for (const box of snapshot.geometry) {
        expect(box.x, `UI_CLIP ${id} ${box.name}`).toBeGreaterThanOrEqual(0)
        expect(box.y, `UI_CLIP ${id} ${box.name}`).toBeGreaterThanOrEqual(0)
        expect(box.x + box.width).toBeLessThanOrEqual(snapshot.viewport.width)
        expect(box.y + box.height).toBeLessThanOrEqual(snapshot.viewport.height)
        expect(box.overflowX, `UI_HORIZONTAL_OVERFLOW ${id} ${box.name}`).toBe(false)
      }
      if (snapshot.worldHit !== null) expect(snapshot.worldHit, `UI_WORLD_INTERCEPT ${id}`).toBe(true)
      const hud = snapshot.geometry.filter(b => !b.name.includes('ui-dialog'))
      for (let i = 0; i < hud.length; i++) for (const b of hud.slice(i + 1)) {
        const a = hud[i]
        expect(a.x + a.width <= b.x || b.x + b.width <= a.x || a.y + a.height <= b.y || b.y + b.height <= a.y, `UI_OVERLAP ${id} ${a.name}/${b.name}`).toBe(true)
      }
      expect(errors).toEqual([])
      await info.attach(name, { body: actual, contentType: 'image/png' })
      records.push({ id, actual: sha256(actual), metadata: `${id}.json` })
      // Write canonical state before a comparison can fail, preserving failure evidence.
      await writeFile(`${output}/summary.json`, json(records))
      if (candidate) {
        await mkdir(candidate, { recursive: true })
        await writeFile(`${candidate}/${name}`, actual)
        manifest.images[name] = sha256(actual)
      } else {
        await compareUIReference(directory, manifest, name, actual, output)
      }
    },
    async finish() {
      if (candidate) {
        // Gallery / production / scanner tests append distinct keys at one width.
        const path = `${candidate}/${page.viewportSize()!.width}.json`
        const previous = JSON.parse(await readFile(path, 'utf8').catch(() => '{}'))
        await writeFile(path, json({ ...manifest, images: { ...previous.images, ...manifest.images } }))
      }
      await info.attach('ui-state-summary', { path: `${output}/summary.json`, contentType: 'application/json' })
    },
  }
}

for (const viewport of sizes) {
  test(`UI-GALLERY production primitives and keyboard states ${viewport.width}`, async ({ page }, info) => {
    await page.setViewportSize(viewport)
    const capture = await recorder(page, info)
    for (const context of ['planet', 'space']) {
      await page.goto(`/tests/ui.html?context=${context}`)
      await capture.capture(`gallery-${context}-default`)
      const opener = page.getByRole('button', { name: 'Inspect equipment' })
      await opener.hover()
      await capture.capture(`gallery-${context}-hover`, true)
      await page.keyboard.press('Tab'); await opener.focus()
      if (process.env.SALIMON_UI_MUTATION === 'focus') await page.addStyleTag({ content: '*:focus-visible { outline: none !important }' })
      expect(await opener.evaluate(el => getComputedStyle(el).outlineWidth), 'UI_FOCUS_REGRESSION').toBe('2px')
      await capture.capture(`gallery-${context}-focus`)
      await opener.press('Enter')
      const modal = page.getByRole('dialog', { name: 'Equipment review' })
      await expect(modal).toBeFocused()
      await page.keyboard.press('Shift+Tab')
      await expect(modal.getByRole('button', { name: 'Close Equipment review' })).toBeFocused()
      await capture.capture(`gallery-${context}-confirmation`)
      await page.keyboard.press('Escape'); await expect(opener).toBeFocused()
    }
    await capture.finish()
  })

  test(`UI-WORLD built surface, cabin, storage and cockpit states ${viewport.width}`, async ({ page }, info) => {
    test.setTimeout(240000)
    await page.setViewportSize(viewport)
    const capture = await recorder(page, info)
    // Seed a valid earlier expedition through domain transactions; no shipping setters.
    let seed = save('surface')
    await page.addInitScript(({ key }) => { const raw = sessionStorage.getItem('ui-seed'); if (raw) localStorage.setItem(key, raw) }, { key: RESOURCE_SAVE_KEY })
    await boot(page, '', 'http://127.0.0.1:4175')
    const load = async (raw: string) => {
      seed = raw
      await page.evaluate(raw => sessionStorage.setItem('ui-seed', raw), raw)
      await page.reload(); await page.clock.runFor(5000)
      await expect(page.locator('.top-down-scene')).toHaveAttribute('data-render-ready', 'true')
      await expect(page.getByRole('meter', { name: 'Health', exact: true })).toHaveAttribute('value', String(JSON.parse(raw).survival.health))
    }
    await load(seed)
    await capture.capture('surface-context')
    await page.keyboard.press('i')
    await expect(page.locator('#field-inventory')).toBeFocused()
    const before = await position(page)
    await page.keyboard.press('w'); await page.keyboard.press('e'); await page.clock.runFor(100)
    expect(await position(page)).toEqual(before)
    await capture.capture('inventory-empty')
    await page.keyboard.press('Escape')
    await expect(page.getByRole('button', { name: 'Field inventory' })).toBeFocused()
    await page.getByRole('button', { name: 'Equip gathering tool' }).click()
    await page.keyboard.press('e'); await walkTicks(page, [], 30)
    await expect(page.getByRole('progressbar', { name: 'Next kilogram extraction interval' })).toHaveAttribute('value', '30')
    await capture.capture('surface-gathering')
    await load(save('full'))
    await page.keyboard.press('i')
    const parsed = JSON.parse(seed), usage = resourceInventoryUsage(parsed.inventory, { mineral: {} }, {})
    await expect(page.locator('[data-inventory-weight]')).toHaveText(`${usage.grams / 1000} / 10 kg`)
    await expect(page.locator('[data-inventory-slots]')).toHaveText(`${usage.slots} / 12 slots`)
    await capture.capture('inventory-full')
    await load(save('critical'))
    await expect(page.getByLabel('Survival condition')).toContainText('Low health')
    await page.getByRole('button', { name: 'Help', exact: true }).click()
    await expect(page.getByRole('status', { name: 'Persistent warnings' })).toContainText('Low health')
    await capture.capture('critical-help')
    await load(save('terminal'))
    await expect(page.getByRole('button', { name: /Start fresh character/ })).toBeVisible()
    await capture.capture('terminal')
    await load(save('storage'))
    await capture.capture('cabin-context')
    await page.keyboard.press('e')
    const panel = page.getByRole('dialog', { name: 'Resource Container', exact: true })
    await expect(panel).toBeFocused()
    const input = panel.getByRole('textbox', { name: 'Amount in grams' })
    await input.fill('333'); await page.getByRole('button', { name: 'Review deposit' }).press('Enter')
    await expect(panel).toContainText('Pending draft · nothing has moved')
    const pending = await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
    expect(pending.stores[SCOUT_RESOURCE_CONTAINER_ID].quantities).toEqual({})
    await panel.focus(); await panel.evaluate(el => { el.scrollTop = 0 })
    await capture.capture('storage-review')
    await panel.evaluate(el => { el.scrollTop = el.scrollHeight })
    await capture.capture('storage-review-controls')
    await page.getByRole('button', { name: 'Confirm deposit' }).press('Enter')
    await expect(panel.locator('[data-carried-capacity]')).toHaveText('0.667 / 10 kg')
    await expect(panel.locator('[data-container-capacity]')).toHaveText('0.333 / 50 kg')
    await input.fill('99999'); await page.getByRole('button', { name: 'Review deposit' }).press('Enter')
    await expect(panel).toContainText('Transfer rejected · nothing moved')
    await panel.focus(); await panel.evaluate(el => { el.scrollTop = 0 })
    await capture.capture('storage-error')
    await panel.evaluate(el => { el.scrollTop = el.scrollHeight })
    await capture.capture('storage-error-controls')
    await page.keyboard.press('Escape'); await expect(panel).toBeHidden()
    await load(save('cockpit'))
    await page.keyboard.press('e')
    await expect.poll(async () => { await page.clock.runFor(32); return page.locator('canvas').count() }).toBe(1)
    await page.clock.runFor(64)
    await expect(page.getByLabel('Throttle', { exact: true })).toBeDisabled()
    const ship = async () => JSON.parse((await page.locator('main').getAttribute('data-ship'))!)
    const initialShip = await ship()
    await capture.capture('cockpit-horizon')
    await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
    await page.getByRole('button', { name: 'Vesper Moon', exact: true }).click()
    await page.getByRole('button', { name: 'Look at', exact: true }).click()
    for (let n = 0; n < 25; n++) await page.clock.fastForward(50)
    await capture.capture('cockpit-selected-dark')
    await page.keyboard.press('Escape')
    await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'seated')
    await page.getByLabel('Survival condition').getByRole('button', { name: 'Help', exact: true }).press('Enter')
    const help = page.getByRole('dialog', { name: 'Surface controls' })
    await expect(help).toBeFocused()
    const protectedView = await page.locator('.panoramic').getAttribute('data-observation')
    await page.keyboard.press('e'); await page.keyboard.press('ArrowLeft'); await page.keyboard.press('w')
    await page.clock.runFor(128)
    expect(await page.locator('.panoramic').getAttribute('data-observation')).toBe(protectedView)
    expect((await ship()).position).toEqual(initialShip.position)
    expect((await ship()).mode).toBe('grounded')
    expect((await ship()).yaw).toBe(initialShip.yaw)
    expect((await ship()).throttle).toBe(0)
    expect((await ship()).velocityMps).toEqual(initialShip.velocityMps)
    await page.keyboard.press('Escape'); await expect(help).toBeHidden()
    await page.keyboard.press('Escape')
    await expect(page.locator('main')).toHaveAttribute('data-player-mode', 'standing')
    await page.clock.runFor(5000)
    await capture.capture('surface-return')
    await capture.finish()
  })

  test(`UI-CONTACT scanner selected, occluded and lost states ${viewport.width}`, async ({ page }, info) => {
    await page.setViewportSize(viewport)
    await page.clock.install({ time: new Date(epoch) }); await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
    const capture = await recorder(page, info)
    await page.goto('/tests/perspective.html')
    const settle = async () => { await page.clock.runFor(64) }
    await expect.poll(async () => { await settle(); return page.locator('canvas').evaluate(c => !!(c as HTMLCanvasElement).getContext('webgl2')) }).toBe(true)
    await settle()
    const snapshot = () => page.evaluate(() => (window as unknown as { sensorSnapshot(): { selected: string | null; actions: number; markers: { ids: string[] }[] } }).sensorSnapshot())
    await page.locator('canvas').click({ position: { x: viewport.width / 2, y: viewport.height / 2 } }); await settle()
    expect((await snapshot()).selected).toBeNull()
    await expect(page.getByRole('button', { name: /Sensor contact/ })).toHaveCount(0)
    await capture.capture('contacts-unscanned')
    await page.getByRole('button', { name: 'Pulse', exact: true }).click(); await settle()
    expect((await snapshot()).markers.flatMap(m => m.ids)).not.toContain('hidden')
    await page.getByRole('button', { name: 'Sensor contact Detected planet', exact: true }).click(); await settle()
    expect((await snapshot()).selected).toBe('planet')
    await capture.capture('contacts-selected-occluded')
    await page.getByRole('button', { name: 'Leave range', exact: true }).click(); await settle()
    expect((await snapshot()).selected).toBeNull()
    expect((await snapshot()).actions).toBe(0)
    await expect(page.getByRole('button', { name: 'Set direction', exact: true })).toHaveCount(0)
    await capture.capture('contacts-lost')
    await page.goto('/tests/panoramic.html?bright')
    await expect.poll(async () => { await settle(); return page.locator('canvas').count() }).toBe(1)
    await page.evaluate(() => { const fixture = window as unknown as { hudAdvance(n: number): void; hudScenario(s: string): void }; fixture.hudAdvance(5); fixture.hudScenario('warning') })
    await settle()
    await expect(page.getByLabel('Navigation warnings')).toContainText('dangerous approach')
    await capture.capture('cockpit-bright-warning')
    await page.getByRole('button', { name: 'Help', exact: true }).press('Enter')
    await expect(page.getByRole('status', { name: 'Persistent warnings' })).toContainText('Low health')
    await capture.capture('cockpit-bright-focused-warning')
    await capture.finish()
  })
}

function save(kind: 'surface' | 'full' | 'critical' | 'terminal' | 'storage' | 'cockpit') {
  const initial = createTopDownState('exterior', true)
  const frame = kind === 'storage' || kind === 'cockpit' ? 'ship:scout' as const : 'planet:fixture' as const
  const xy = kind === 'storage' ? [.5, 4.25] : kind === 'cockpit' ? [0, -3.1] : [-7.1, 0]
  const anchor = { frame, point: point2(xy[0], xy[1]) }
  const world = { ...initial, camera: { ...initial.camera, anchor }, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor } : e) }
  let raw: string | null = null
  const session = new ResourceSession({ getItem: () => raw, setItem: (_key, value) => { raw = value } }, world, undefined, undefined, () => new Date(epoch).getTime())
  if (kind === 'full' || kind === 'storage') {
    expect(session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: kind === 'full' ? 10000 : 1000 })).toBe(true)
  }
  const value = { ...session.snapshot, survival: { ...session.snapshot.survival, health: kind === 'critical' || kind === 'cockpit' ? 20 : kind === 'terminal' ? 0 : 200 } }
  return json(value)
}
