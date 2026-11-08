import { mkdir, writeFile } from 'node:fs/promises'
import { createHash } from 'node:crypto'
import { expectSameResourceState, expectSafeSurvivalReload } from './saveAssertions'
import { test, expect } from '@playwright/test'
import { compareImages, POLICY } from '../scripts/visual-comparison.ts'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { point2 } from '../src/topDownFrames.ts'
import { SCOUT_RESOURCE_CONTAINER_ID as containerId } from '../src/scoutResourceContainer.ts'
import { boot, position, walkTicks, walkTo } from './worldJourney'

const directory = 'test-results/storage-originals'
const saved = (page: import('@playwright/test').Page) => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
const dialog = (page: import('@playwright/test').Page) => page.getByRole('dialog', { name: 'Resource Container', exact: true })

test('STORE-UI-1 built mine, enter, confirm deposit/withdraw, cancel, exit and reload repeats', async ({ browser }, info) => {
  const originals: Record<string, Buffer> = {}, records = []
  for (let replay = 0; replay < 2; replay++) {
    const context = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1, locale: 'en-US', timezoneId: 'UTC', colorScheme: 'dark', reducedMotion: 'reduce' })
    const page = await context.newPage(), errors: string[] = []
    page.on('pageerror', error => errors.push(error.message))
    try {
      await boot(page, '', 'http://127.0.0.1:4175')
      await expect(page.locator('[data-inventory-weight]')).toHaveText('0 / 10 kg')
      await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
      await page.getByRole('button', { name: 'Equip gathering tool' }).click()
      await page.keyboard.press('e'); await walkTicks(page, [], 60); await page.keyboard.press('Escape')
      await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
      for (const p of [[-6, 3], [-2, 3], [-2, 7]]) await walkTo(page, p, 'planet:fixture', true)
      for (const p of [[0, 7], [.5, 4.25]]) await walkTo(page, p, 'ship:scout', true)
      await expect(page.locator('[data-prompt]')).toContainText('Resource Container · 50 kg · Use [E]')
      await page.keyboard.press('e')
      await expect(dialog(page)).toBeVisible()
      await expect(page.locator('[data-gathering]')).toHaveAttribute('data-gathering', '') // E chose storage even with the tool equipped.
      const capture = async (name: string) => {
        await page.clock.runFor(5000)
        const png = await page.screenshot({ path: info.outputPath(`${name}-${replay}.png`) })
        if (!replay) originals[name] = png
        else { const result = compareImages(originals[name]!, png); await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ pass: result.pass, changedPixels: result.changedPixels, rawChangedPixels: result.rawChangedPixels, policy: POLICY }), contentType: 'application/json' }); await info.attach(`${name}-diff.png`, { body: result.diff, contentType: 'image/png' }); expect(result.pass, name).toBe(true) }
      }
      const shipBeforePanel = JSON.parse((await page.locator('main').getAttribute('data-ship'))!)
      await capture('storage-open')
      const shipAfterPanel = JSON.parse((await page.locator('main').getAttribute('data-ship'))!)
      expect(shipAfterPanel.tick).toBeGreaterThan(shipBeforePanel.tick)
      expect(shipAfterPanel.throttle).toBe(shipBeforePanel.throttle)
      expect(shipAfterPanel.velocityMps).toEqual(shipBeforePanel.velocityMps)
      const input = page.getByRole('textbox', { name: 'Amount in grams' })
      await input.fill('333'); await page.getByRole('button', { name: 'Review deposit' }).click()
      const draft = await saved(page)
      expect(draft.stores[containerId].quantities).toEqual({})
      await page.getByRole('button', { name: 'Confirm deposit' }).press('Enter'); await page.keyboard.press('Enter')
      await expect(dialog(page).locator('[data-carried-raw="mineral"]')).toHaveText('0.667')
      await expect(dialog(page).locator('[data-container-raw="mineral"]')).toHaveText('0.333')
      await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Saved: deposited 333 g')
      await input.fill('111'); await page.getByRole('button', { name: 'Review withdrawal' }).click(); await page.getByRole('button', { name: 'Confirm withdraw' }).click()
      await expect(dialog(page).locator('[data-carried-capacity]')).toHaveText('0.778 / 10 kg')
      await expect(dialog(page).locator('[data-container-capacity]')).toHaveText('0.222 / 50 kg')
      await expect(page.getByRole('progressbar', { name: 'Container weight' })).toHaveAttribute('value', '222')
      await capture('storage-transferred')
      const committed = await saved(page)
      await input.fill('9999'); await page.getByRole('button', { name: 'Review deposit' }).click()
      await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Insufficient amount')
      expectSameResourceState(await saved(page), committed)
      await capture('storage-error')
      await input.fill('1'); await page.getByRole('button', { name: 'Review deposit' }).click(); await page.keyboard.press('Escape')
      await expect(dialog(page)).toHaveCount(0); expectSameResourceState(await saved(page), committed)
      await page.keyboard.press('e'); await expect(dialog(page)).toBeVisible()
      await input.fill('1'); await page.getByRole('button', { name: 'Review withdrawal' }).click()
      await page.evaluate(() => window.dispatchEvent(new Event('blur')))
      await expect(dialog(page)).toHaveCount(0); expect((await saved(page)).inventory).toEqual(committed.inventory)
      await page.keyboard.press('e'); await expect(dialog(page)).toBeVisible()
      const beforeTyping = await position(page)
      await input.fill('wasde'); await walkTicks(page, [], 5)
      expect(await position(page)).toEqual(beforeTyping)
      await page.keyboard.press('Escape')
      await walkTicks(page, 's', 5); expect(await position(page)).not.toEqual(beforeTyping)
      for (const p of [[0, 4.5], [0, 7]]) await walkTo(page, p, 'ship:scout', true)
      await walkTo(page, [-2, 7], 'planet:fixture', true)
      await walkTicks(page, [], 60)
      const beforeReload = await saved(page), where = await position(page)
      await page.reload()
      await expect(page.locator('[data-resource="mineral"]')).toHaveText('0.778 kg')
      expect(await position(page)).toEqual(where)
      const afterReload = await saved(page)
      expectSafeSurvivalReload(afterReload, beforeReload)
      expect(afterReload.inventory.quantities.mineral + afterReload.stores[containerId].quantities.mineral + afterReload.sources['source:landing:mineral:01'].remainingGrams).toBe(60000)
      expect(afterReload.stores[containerId].quantities.mineral).toBe(222)
      expect(errors).toEqual([])
      records.push({ replay, shipBeforePanel, shipAfterPanel, committed, beforeReload, afterReload, errors, browser: browser.version(), input: 'Playwright keyboard/mouse; focus-loss event through production listener' })
    } finally { await context.close() }
  }
  const negative = compareImages(originals['storage-open']!, originals['storage-transferred']!)
  expect(negative.pass).toBe(false)
  await info.attach('storage-journeys.json', { body: JSON.stringify({ records, wrongCheckpoint: { pass: negative.pass, changedPixels: negative.changedPixels }, policy: POLICY }, null, 2), contentType: 'application/json' })
})

/** Valid prior expedition fixture, not a production state setter. */
function fullSave() {
  const initial = createTopDownState('exterior', true)
  const world = { ...initial, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(.5, 4.25) } } : e) }
  let raw: string | null = null
  const session = new ResourceSession({ getItem: () => raw, setItem: (_key, value) => { raw = value } }, world)
  for (const [kind, grams] of [['store', 50000], ['player', 1000]] as const) {
    expect(session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: kind === 'store' ? { kind, id: containerId } : { kind }, resourceId: 'mineral', grams })).toBe(true)
  }
  return raw!
}
test('STORE-UI-2 full/overweight, stale-save failure, confirmation cancellation and narrow viewport', async ({ page }, info) => {
  await page.addInitScript(({ key, raw }) => localStorage.setItem(key, raw), { key: RESOURCE_SAVE_KEY, raw: fullSave() })
  await boot(page, '', 'http://127.0.0.1:4175')
  await expect(page.locator('[data-inventory-weight]')).toHaveText('1 / 10 kg')
  await page.keyboard.press('e'); await expect(dialog(page)).toBeVisible()
  await expect(dialog(page).locator('[data-container-capacity]')).toHaveText('50 / 50 kg')
  await page.clock.runFor(5000); await page.screenshot({ path: info.outputPath('storage-full.png') })
  const input = page.getByRole('textbox', { name: 'Amount in grams' }), raw = await saved(page)
  await input.fill('1'); await page.getByRole('button', { name: 'Review deposit' }).click()
  await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Container overweight: only 0 g free')
  expectSameResourceState(await saved(page), raw)
  await page.clock.runFor(5000); await page.screenshot({ path: info.outputPath('storage-full-error.png') })
  await input.fill('9001'); await page.getByRole('button', { name: 'Review withdrawal' }).click()
  await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Player overweight: only 9000 g free')
  expectSameResourceState(await saved(page), raw)
  for (const value of ['0', '-1', '0.5', '1e3']) { await input.fill(value); await page.getByRole('button', { name: 'Review withdrawal' }).click(); await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('positive whole number of grams') }
  await input.fill('333'); await page.getByRole('button', { name: 'Review withdrawal' }).click(); await page.getByRole('button', { name: 'Cancel transfer' }).click()
  expectSameResourceState(await saved(page), raw)
  await input.fill('333'); await page.getByRole('button', { name: 'Review withdrawal' }).click()
  await page.evaluate(key => localStorage.setItem(key, localStorage.getItem(key)! + ' '), RESOURCE_SAVE_KEY)
  const staleBytes = await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)
  await page.getByRole('button', { name: 'Confirm withdraw' }).click()
  await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Saving failed or the save changed elsewhere')
  expect(await page.evaluate(key => localStorage.getItem(key), RESOURCE_SAVE_KEY)).toBe(staleBytes)
  await expect(dialog(page).locator('[data-carried-raw="mineral"]')).toHaveText('1')
  await expect(page.getByRole('button', { name: 'Review withdrawal' })).toBeDisabled()
  await page.screenshot({ path: info.outputPath('storage-save-error.png') })
  await page.setViewportSize({ width: 390, height: 844 })
  await expect(dialog(page)).toHaveCount(0)
  await page.keyboard.press('e'); await expect(dialog(page)).toBeVisible()
  const bounds = await dialog(page).boundingBox()
  expect(bounds!.x).toBeGreaterThanOrEqual(0); expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(390)
  await page.screenshot({ path: info.outputPath('storage-narrow.png') })
  await page.getByRole('button', { name: 'Close Resource Container' }).click()
  await expect(dialog(page)).toHaveCount(0)
})

for (const [width, height] of [[960, 600], [1280, 720], [1920, 1080]]) {
  test(`STORE-UI-3 industrial equipment states, focus and local scroll ${width}x${height}`, async ({ page }, info) => {
    await page.setViewportSize({ width, height })
    await page.addInitScript(({ key, raw }) => localStorage.setItem(key, raw), { key: RESOURCE_SAVE_KEY, raw: fullSave() })
    await boot(page, '', 'http://127.0.0.1:4175')
    await page.keyboard.press('e')
    const panel = dialog(page), input = page.getByRole('textbox', { name: 'Amount in grams' })
    await expect(panel).toBeFocused()
    await expect(panel).toHaveAccessibleName('Resource Container')
    await expect(input).toHaveAccessibleDescription(/positive whole gram amount/)
    const close = panel.getByRole('button', { name: 'Close Resource Container' })
    await page.keyboard.press('Shift+Tab'); await expect(close).toBeFocused()
    await page.keyboard.press('Tab'); await expect(panel.getByRole('combobox')).toBeFocused()
    const before = await position(page)
    await page.keyboard.press('w'); await page.keyboard.press('e'); await page.keyboard.press('p')
    await walkTicks(page, [], 5); expect(await position(page)).toEqual(before)
    await input.focus()
    const measurements = await panel.evaluate(el => {
      const box = el.getBoundingClientRect(), style = getComputedStyle(el)
      const input = el.querySelector('input')!
      // WCAG relative luminance, using opaque shared token inks/surfaces.
      const luminance = (rgb: string) => rgb.match(/[\d.]+/g)!.slice(0, 3).map(Number).map(n => n / 255).map(n => n <= .04045 ? n / 12.92 : ((n + .055) / 1.055) ** 2.4).reduce((sum, n, i) => sum + n * [.2126, .7152, .0722][i]!, 0)
      const contrast: Record<string, number> = {}
      for (const token of ['--ui-text', '--ui-muted', '--ui-active', '--ui-mechanical', '--ui-danger']) {
        const probe = document.createElement('span'); probe.style.color = `var(${token})`; el.append(probe)
        const ink = luminance(getComputedStyle(probe).color); probe.remove()
        const background = luminance(style.backgroundColor)
        contrast[token] = (Math.max(ink, background) + .05) / (Math.min(ink, background) + .05)
      }
      return { box: { x:box.x, y:box.y, width:box.width, height:box.height }, scrollHeight:el.scrollHeight, clientHeight:el.clientHeight,
        overflow:style.overflowY, animation:style.animationDuration, focus:getComputedStyle(input).outlineWidth,
        bodyScroll:document.documentElement.scrollHeight > innerHeight, horizontalOverflow:el.scrollWidth > el.clientWidth, contrast }
    })
    expect(measurements.box.x).toBeGreaterThanOrEqual(0); expect(measurements.box.y).toBeGreaterThanOrEqual(0)
    expect(measurements.box.x + measurements.box.width).toBeLessThanOrEqual(width)
    expect(measurements.box.y + measurements.box.height).toBeLessThanOrEqual(height)
    expect(measurements.bodyScroll).toBe(false); expect(measurements.horizontalOverflow).toBe(false)
    expect(measurements.overflow).toBe('auto'); expect(measurements.animation).toBe('0s'); expect(measurements.focus).toBe('2px')
    for (const ratio of Object.values(measurements.contrast)) expect(ratio).toBeGreaterThanOrEqual(4.5)
    const captures: Record<string, string> = {}
    const capture = async (state: string) => {
      await panel.focus() // Keep the native blinking text caret out of repeat captures.
      await page.mouse.move(0, 0)
      await page.clock.runFor(250)
      await panel.evaluate(el => { el.scrollTop = 0 })
      const first = await panel.screenshot({ animations:'disabled' })
      const repeat = await panel.screenshot({ animations:'disabled' })
      await info.attach(`${state}-original`, { body:first, contentType:'image/png' })
      await info.attach(`${state}-repeat`, { body:repeat, contentType:'image/png' })
      expect(repeat.equals(first)).toBe(true)
      const name = `industrial-${width}-${state}.png`
      await writeFile(`${directory}/${name}`, first)
      captures[name] = createHash('sha256').update(first).digest('hex')
      if ((state === 'pending' || state === 'rejected') && await panel.evaluate(el => el.scrollHeight > el.clientHeight)) {
        await panel.evaluate(el => { el.scrollTop = el.scrollHeight })
        const lower = await panel.screenshot({ animations:'disabled' })
        expect((await panel.screenshot({ animations:'disabled' })).equals(lower)).toBe(true)
        const lowerName = `industrial-${width}-${state}-controls.png`
        await writeFile(`${directory}/${lowerName}`, lower)
        captures[lowerName] = createHash('sha256').update(lower).digest('hex')
      }
      return first
    }
    await mkdir(directory, { recursive:true })
    const idle = await capture('ready')
    await input.fill('333'); await panel.getByRole('button', { name:'Review withdrawal' }).click()
    await expect(panel.locator('.container-state')).toHaveText('Pending review · not committed')
    await expect(panel.locator('[data-container-capacity]')).toHaveText('50 / 50 kg')
    const pending = await capture('pending')
    expect(pending.equals(idle)).toBe(false)
    await panel.getByRole('button', { name:'Confirm withdraw' }).click()
    await expect(panel.locator('.container-state')).toHaveText('Transfer committed')
    await expect(panel.locator('[data-container-capacity]')).toHaveText('49.667 / 50 kg')
    await capture('committed')
    await input.fill('0'); await panel.getByRole('button', { name:'Review deposit' }).click()
    await expect(panel.locator('.container-state')).toHaveText('Transfer rejected · nothing moved')
    await expect(panel.getByRole('status', { name:'Container feedback' })).toContainText('positive whole number of grams')
    await capture('rejected')
    await close.focus(); await expect(close).toBeInViewport()
    await writeFile(`${directory}/measurements-${width}.json`, JSON.stringify({ viewport:{width,height}, dpr:1, measurements, captures, negativeCheckpoint:'ready versus pending rejected', browser:page.context().browser()!.version() }, null, 2) + '\n')
    await page.keyboard.press('Escape'); await expect(panel).toHaveCount(0)
    await expect(page.locator('[data-inventory-weight]')).toHaveText('1.333 / 10 kg')
    await page.emulateMedia({ reducedMotion:'no-preference' })
    await page.keyboard.press('e'); await expect(dialog(page)).toBeVisible()
    expect(await dialog(page).evaluate(el => getComputedStyle(el).animationDuration)).toBe('0.18s')
    await page.keyboard.press('Escape'); await expect(dialog(page)).toHaveCount(0)
    await info.attach('measurements', { path:`${directory}/measurements-${width}.json`, contentType:'application/json' })
  })
}

test('FEEDBACK persistent hazard survives rejection, draft and modal scroll without repeated announcements', async ({ page }, info) => {
  const raw = JSON.parse(fullSave()); raw.survival.health = 20
  await page.addInitScript(({ key, raw }) => localStorage.setItem(key, JSON.stringify(raw)), { key: RESOURCE_SAVE_KEY, raw })
  await page.setViewportSize({ width: 960, height: 600 })
  await boot(page, '', 'http://127.0.0.1:4175')
  await page.keyboard.press('e')
  const panel = dialog(page), warnings = panel.getByRole('status', { name: 'Persistent warnings' })
  await expect(warnings).toContainText('Low health')
  await expect(warnings).toHaveAttribute('aria-live', 'assertive')
  const input = panel.getByRole('textbox', { name: 'Amount in grams' })
  await input.fill('1'); await page.getByRole('button', { name: 'Review deposit' }).click()
  await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Container overweight')
  await expect(warnings).toContainText('Low health')
  await input.fill('333'); await page.getByRole('button', { name: 'Review withdrawal' }).click()
  await expect(panel).toContainText('Pending draft · nothing has moved')
  await expect(warnings).toContainText('Low health')
  await page.getByRole('button', { name: 'Confirm withdraw' }).click()
  await expect(page.getByRole('status', { name: 'Container feedback' })).toContainText('Saved: withdrew 333 g')
  await expect(warnings).toContainText('Low health')
  await warnings.evaluate(el => {
    Object.assign(window, { warningMutations: 0 })
    new MutationObserver(records => { (window as unknown as { warningMutations: number }).warningMutations += records.length }).observe(el, { subtree:true, childList:true, characterData:true })
  })
  await page.clock.runFor(1000)
  expect(await page.evaluate(() => (window as unknown as {warningMutations:number}).warningMutations)).toBe(0)
  await panel.evaluate(el => { el.scrollTop = el.scrollHeight })
  await expect(warnings).toBeInViewport()
  expect(await panel.evaluate(el => getComputedStyle(el).animationDuration)).toBe('0s')
  await page.clock.runFor(50)
  await page.screenshot({ path: info.outputPath('persistent-hazard-committed.png') })
  const committed = await saved(page)
  await input.fill('1'); await page.getByRole('button', { name: 'Review withdrawal' }).click()
  await page.keyboard.press('Escape')
  await expect(panel).toHaveCount(0)
  expectSameResourceState(await saved(page), committed)
  await expect(page.getByLabel('Survival condition')).toContainText('Low health')
  await info.attach('committed-state.json', { body:JSON.stringify(committed), contentType:'application/json' })
})
