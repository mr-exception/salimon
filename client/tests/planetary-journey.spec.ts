import { test, expect } from '@playwright/test'
import { RESOURCE_SAVE_KEY, ResourceSession } from '../src/resourceSession.ts'
import { createTopDownState } from '../src/topDownRuntime.ts'
import { localToSurface, surfaceRegion } from '../src/surfaceCoordinates.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
import { boot, walkTicks, walkTo } from './worldJourney'

test.use({ trace: { mode: 'on', snapshots: false, screenshots: true, sources: true } })

const saved = async (page: import('@playwright/test').Page) => {
  await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
  return page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
}
const physical = (s: Awaited<ReturnType<typeof saved>>) => localToSurface(s.location.anchor.chart ?? surfaceRegion(s.scout.surface), s.location.anchor.point.offsetM.slice(0, 2))

test('PLANET-854 restored Vesper walk, four regions, turn, mine, mark, Scout storage and reload', async ({ page }, info) => {
  test.setTimeout(600000)
  const initial = new ResourceSession({ getItem: () => null, setItem: () => {} }, createTopDownState('exterior', true), undefined, undefined, () => Date.parse('2026-11-04T00:00:00Z'))
  const raw = JSON.stringify(initial.snapshot, (_, v) => typeof v === 'bigint' ? String(v) : v)
  await page.addInitScript(({key,raw}) => { if (!localStorage.getItem(key)) localStorage.setItem(key,raw) }, { key: RESOURCE_SAVE_KEY, raw })
  const errors: string[] = [], routes: unknown[] = []
  page.on('pageerror', error => errors.push(error.message))
  await boot(page, '', 'http://127.0.0.1:4175')
  const start = await saved(page)
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
  await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
  await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  await page.keyboard.press('e'); await walkTicks(page, [], 30); await page.keyboard.press('Escape')
  const interrupted = await saved(page)
  expect(interrupted.inventory.quantities.mineral).toBeUndefined()
  expect(interrupted.miningWork['source:landing:mineral:01']).toBe(480)
  await page.reload(); await page.clock.runFor(200)
  await page.getByRole('button', { name: 'Equip gathering tool' }).click()
  await page.keyboard.press('e'); await walkTicks(page, [], 30); await page.keyboard.press('Escape')
  expect((await saved(page)).inventory.quantities.mineral).toBe(1000)
  await page.keyboard.press('m')
  const notebook = page.getByRole('dialog', { name: 'Field notebook' })
  await notebook.getByLabel('Name (optional)', { exact: true }).fill('Worked Vesper outcrop')
  await notebook.getByRole('button', { name: 'Inspect west boulder' }).click()
  await page.keyboard.press('Escape')
  await page.locator('main').click({position:{x:640,y:350}})
  const marked = await saved(page)
  expect(marked.journal.marks).toHaveLength(1)
  // The authored west corridor connects to canonical geological regions. Every
  // movement step goes through production collision and the sole 60 Hz clock.
  for (const p of [[-6,3],[-20,3],[-20,0]]) await walkTo(page,p,'planet:fixture',true)
  const near = physical(await saved(page))
  const move = async (key: string, ticks: number) => {
    const tick = async () => Number(await page.locator('main').getAttribute('data-tick'))
    const target = await tick() + ticks
    await page.keyboard.down(key)
    while (target - await tick() > 16) await page.clock.fastForward(200)
    await page.keyboard.up(key)
    const rest = target - await tick(); if (rest > 0) await walkTicks(page, key, rest)
    expect(await tick()).toBe(target)
  }
  for (let i = 0; i < 5; i++) {
    await move('a', 1600)
    const s = await saved(page), ground = page.locator('[data-generated-ground]')
    const stats = JSON.parse((await ground.getAttribute('data-stream'))!)
    expect(stats.regions).toBeLessThanOrEqual(32)
    expect(stats.samples).toBeLessThanOrEqual(32 * 2048)
    expect(stats.frameGenerated).toBeLessThanOrEqual(2048)
    expect(s.scout.ship.position).toEqual(start.scout.ship.position)
    routes.push({ location: s.location, region: surfaceRegion(physical(s)).id, stats })
  }
  const far = await saved(page)
  expect(far.journal.visits.length).toBeGreaterThanOrEqual(5)
  expect(new Set(routes.map(r => (r as {region:string}).region)).size).toBeGreaterThanOrEqual(4)
  expect(Math.hypot(...physical(far).radialM.map((n,i) => n-near.radialM[i]!))).toBeGreaterThan(319)
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(0)
  await walkTicks(page, [], 180)
  const original = await page.screenshot({ path: info.outputPath('geological-far.png') })
  expect(compareImages(original, await page.screenshot({ path: info.outputPath('geological-far-repeat.png') })).pass).toBe(true)
  await page.reload(); await page.clock.runFor(200)
  expect(physical(await saved(page))).toEqual(physical(far))
  for (let i = 0; i < 5; i++) await move('d', 1600)
  expect(Math.hypot(...physical(await saved(page)).radialM.map((n,i) => n-near.radialM[i]!))).toBeLessThan(.001)
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
  await expect(page.locator('[data-field-mark]')).toHaveAttribute('data-field-mark', marked.journal.marks[0].id)
  await expect(page.locator('[data-entity="source:landing:mineral:01"]')).toHaveAttribute('data-remaining-grams','59000')
  // Short waypoints keep diagonal keyboard steering clear of the west stones.
  for (const p of [[-20,2],[-16,2],[-12,2],[-6,3],[-2,3],[-2,7]]) await walkTo(page,p,'planet:fixture',true)
  for (const p of [[0,7],[.5,3.7]]) await walkTo(page,p,'ship:scout',true)
  await expect(page.locator('[data-prompt]')).toContainText('Resource Container')
  await page.keyboard.press('e')
  const storage = page.getByRole('dialog', { name: 'Resource Container', exact: true })
  await expect(storage).toBeVisible()
  await storage.getByRole('textbox', { name: 'Amount in grams' }).fill('1000')
  await storage.getByRole('button', { name: 'Review deposit' }).click()
  await storage.getByRole('button', { name: 'Confirm deposit' }).click()
  await page.keyboard.press('Escape')
  const stored = await saved(page)
  await page.reload(); await page.clock.runFor(200)
  const restored = await saved(page)
  expect(restored.location).toEqual(stored.location)
  expect(restored.location.anchor.frame).toBe('ship:scout')
  expect(restored.sources).toEqual(stored.sources)
  expect(restored.stores).toEqual(stored.stores)
  expect(restored.journal.marks).toEqual(marked.journal.marks)
  expect(restored.sources['source:landing:mineral:01'].remainingGrams + (restored.inventory.quantities.mineral ?? 0) + restored.stores['scout:storage'].quantities.mineral).toBe(60000)
  expect(restored.stores['scout:storage'].quantities.mineral).toBe(1000)
  expect(restored.scout.ship.position).toEqual(start.scout.ship.position)
  await expect.poll(async () => {
    await page.clock.runFor(50)
    return page.locator('[data-render-ready]').getAttribute('data-render-ready')
  }).toBe('true')
  await walkTicks(page, [], 180)
  await expect(page.locator('[data-entity="ship:scout"]')).toBeVisible()
  await page.screenshot({ path: info.outputPath('returned-storage.png') })
  expect(errors).toEqual([])
  await info.attach('journey.json', { body: JSON.stringify({ start, interrupted, marked, routes, far, restored, browser:page.context().browser()!.version(), input:'Playwright keyboard/mouse', renderer:'Chromium SVG/Canvas CPU; cockpit ANGLE SwiftShader', viewport:info.project.use.viewport, dpr:1, clock:'2026-11-04T00:00:00Z', limits:'Bounded region/sample/work counts; no hardware FPS or heap-byte claim' }), contentType:'application/json' })
})

test('PLANET-854 corrupt, mismatched publication and excessive inventory retain saved bytes', async ({page},info) => {
  const session = new ResourceSession({getItem:()=>null,setItem:()=>{}},createTopDownState())
  const valid = JSON.parse(JSON.stringify(session.snapshot,(_,v)=>typeof v==='bigint'?String(v):v))
  const cases = {
    malformed:'{partial',
    version:JSON.stringify({...valid,version:99}),
    publication:JSON.stringify({...valid,scout:{...valid.scout,surface:{...valid.scout.surface,definition:{...valid.scout.surface.definition,digest:'unknown'}}}}),
    inventory:JSON.stringify({...valid,inventory:{...valid.inventory,quantities:{mineral:10001}}}),
  }
  await boot(page,'','http://127.0.0.1:4175')
  for(const [name,raw] of Object.entries(cases)) {
    await page.evaluate(({key,raw})=>localStorage.setItem(key,raw),{key:RESOURCE_SAVE_KEY,raw})
    await page.reload();await page.clock.runFor(200)
    await expect(page.getByRole('main')).toContainText('Existing data kept')
    await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
    expect(await page.evaluate(key=>localStorage.getItem(key),RESOURCE_SAVE_KEY)).toBe(raw)
    await page.screenshot({path:info.outputPath(`${name}-retained.png`)})
  }
  await info.attach('rejections.json',{body:JSON.stringify(cases),contentType:'application/json'})
})
