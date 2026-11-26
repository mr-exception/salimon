import { expect, test, type Page } from '@playwright/test'
import { benchReserved, benchSpentJ60, GEAR_RECIPES, BENCH_ID } from '../src/gearBench.ts'
import { depositFromId } from '../src/planetDeposits.ts'
import { CORE_YIELD_J } from '../src/scoutEnergyCore.ts'
import { FABRICATOR_GRAMS, MARKER_KIT_ID, FABRICATOR_TICKS, FABRICATOR_WATTS } from '../src/scoutFabricator.ts'
import { RESOURCE_SAVE_KEY, type ResourceSave } from '../src/resourceSession.ts'
import { samplePlanetEnvironment } from '../src/planetEnvironment.ts'
import { healthValue } from '../src/survival.ts'
import * as exact from '../src/exact.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
import { origin, panel, transfer } from './fabricatorFixtures'
import { boot, walkTicks, walkTo } from './worldJourney'

const player = (page: Page) => page.locator('[data-entity="person:player"]')
const read = (page: Page): Promise<ResourceSave> => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
const checkpoint = async (page: Page) => {
  await page.evaluate(() => window.dispatchEvent(new Event('pagehide')))
  return read(page)
}
const shipPath = [[0, 4.7], [0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]
const mineral = 'source:landing:mineral:01'
const biomass = 'source:landing:biology:01'
const approachPath = [[-2, 3], [-2, 7]]
const depositId = 'source:deposit:v1:vesper:263916104a5852c0:1:1:-4:0:3'
const radModule = 'module:radiation:t1'

function conserve(save: ResourceSave) {
  const equipment = save.inventory.equipment!
  expect(new Set(equipment.instances.map(item => item.id)).size).toBe(equipment.instances.length)
  const reserved = benchReserved(save.bench)
  const repairs = (save.bench.repairs ?? []).reduce((sum, job) => sum + Math.ceil(exact.number(exact.decode(job.amount))), 0)
  const fuel = save.convertedGrams
  for (const [resource, initial, source] of [['mineral', 60_000, mineral], ['biomass', 15_000, biomass]] as const) {
    const stored = Object.values(save.stores).reduce((n, store) => n + (store.quantities[resource] ?? 0), 0)
    const manufactured = Object.entries(save.bench.completed)
      .reduce((n, [id, count]) => n + count * GEAR_RECIPES[id]![resource], 0)
    const products = (save.inventory.items[MARKER_KIT_ID] ?? 0) + (save.fabricator.tray?.count ?? 0) + save.deployedMarkers.length
    const fabricator = resource === 'mineral' ? products * FABRICATOR_GRAMS + (save.fabricator.job?.reservedGrams ?? 0) : 0
    const gatheredDeposit = resource === 'mineral' && save.sources[depositId] !== undefined
      ? depositFromId(depositId).initial.remainingGrams - save.sources[depositId]!.remainingGrams : 0
    expect(save.sources[source]!.remainingGrams + (save.inventory.quantities[resource] ?? 0) + stored
      + (fuel[resource] ?? 0) + manufactured + (reserved?.[resource] ?? 0) + fabricator
      + (resource === 'mineral' ? repairs : 0)).toBe(initial + gatheredDeposit)
  }
  expect(save.core.reserveJ60 + save.core.spentJ60).toBe((500_000 + Object.entries(fuel)
    .reduce((joules, [id, grams]) => joules + grams * CORE_YIELD_J[id]!, 0)) * 60)
  const fabTicks = save.fabricator.completed * FABRICATOR_TICKS + (save.fabricator.job ? FABRICATOR_TICKS - save.fabricator.job.remainingTicks : 0)
  expect(save.core.spentJ60).toBe(save.fabricator.otherSpentJ60 + fabTicks * FABRICATOR_WATTS + benchSpentJ60(save.bench))
}

async function openBench(page: Page) {
  await page.locator(`[data-entity="${BENCH_ID}"]`).click({ button: 'right' })
  await page.getByRole('menuitem', { name: /Use Gear Crafting Bench/ }).click()
  const bench = panel(page, 'Gear Crafting Bench')
  await expect.poll(async () => { await page.clock.runFor(32); return bench.isVisible() }).toBe(true)
  return bench
}

test.use({ actionTimeout: 10_000, trace: { mode: 'on', snapshots: false, screenshots: false, sources: true } })

test('H10-922 fresh production expedition conserves equipment, cargo, power and survival', async ({ page }, info) => {
  test.setTimeout(600_000)
  const stages: Record<string, ResourceSave> = {}
  const errors: string[] = []
  page.on('pageerror', error => errors.push(error.message))
  const record = async (stage: string, savePosition = false) => {
    const save = await (savePosition ? checkpoint(page) : read(page))
    stages[stage] = save
    conserve(save)
    return save
  }
  const capture = async (label: string) => {
    const original = await page.screenshot({ path: info.outputPath(`${label}.png`) })
    const repeat = await page.screenshot({ path: info.outputPath(`${label}-repeat.png`) })
    expect(compareImages(original, repeat).pass, label).toBe(true)
    return original
  }
  const main = page.locator('main')
  const pano = page.getByRole('region', { name: 'Panoramic navigation' })
  const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!) as ResourceSave['scout']['ship']
  const openCockpit = async () => {
    await page.getByRole('button', { name: 'Use cockpit', exact: true }).click()
    await page.clock.runFor(64)
    await expect(pano).toBeVisible()
  }
  const exitCockpit = async () => {
    await page.getByRole('button', { name: 'Exit cockpit' }).click()
    await page.clock.runFor(32)
    await expect(pano).toHaveCount(0)
  }
  const advance = async (ms: number) => {
    // Stay below the production clock's 250 ms admitted-frame limit.
    for (let elapsed = 0; elapsed < ms; elapsed += 200) await page.clock.fastForward(Math.min(ms - elapsed, 200))
  }
  const selectContact = async (name: string) => {
    await page.getByRole('button', { name: /Contacts \(C\)/ }).click()
    await page.getByRole('button', { name, exact: true }).click()
    await page.keyboard.press('Escape')
  }
  const refresh = async () => {
    await page.reload()
    await expect.poll(async () => { await page.clock.runFor(32); return page.locator('.top-down-scene').getAttribute('data-render-ready') }).toBe('true')
  }
  try {
    // Deliberately request legacy development switches against the shipping build.
    await boot(page, '?flightFixture&powerFixture&view=interior', origin)
    await expect(page.locator('.power-fixture')).toHaveCount(0)
    await expect(player(page)).toHaveAttribute('data-frame', 'planet:fixture')
    const start = await record('fresh', true)
    const starter = start.inventory.equipment!.instances
    expect(starter.map(item => item.location.kind)).toEqual(['equipped', 'installed'])
    for (const point of approachPath) await walkTo(page, point, 'planet:fixture', true)
    for (const point of [[0, 7], ...shipPath]) await walkTo(page, point, 'ship:scout', true)
    await openCockpit()
    await selectContact('Vesper Moon')
    await expect(page.getByText('Planetary environment · Gear warning', { exact: true })).toBeVisible()
    await capture('starter-scans-hazardous-moon')
    await exitCockpit()
    for (const point of [...shipPath.slice(0, -1).reverse(), [0, 7]]) await walkTo(page, point, 'ship:scout', true)
    for (const point of [...[...approachPath].reverse(), [-6, 3]]) await walkTo(page, point, 'planet:fixture', true)
    expect((await record('scanned', true)).inventory.equipment).toEqual(start.inventory.equipment)

    await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
    await page.getByRole('button', { name: 'Equip gathering tool' }).click()
    await page.keyboard.press('e'); await walkTicks(page, [], 180); await page.keyboard.press('Escape')
    await walkTo(page, [-6, 4.5], 'planet:fixture', true)
    await walkTo(page, [-5, 4.5], 'planet:fixture', true)
    await page.keyboard.press('e'); await walkTicks(page, [], 120); await page.keyboard.press('Escape')
    expect((await record('gathered')).inventory.quantities).toEqual({ mineral: 3000, biomass: 2000 })

    for (const point of [[-2, 4.5], [-2, 7]]) await walkTo(page, point, 'planet:fixture', true)
    for (const point of [[0, 7], [0, 3], [.8, 1.8]]) await walkTo(page, point, 'ship:scout', true)
    await page.keyboard.press('e')
    await expect(panel(page, 'Energy Core')).toBeVisible()
    await transfer(page, 'deposit', 100)
    await page.keyboard.press('Escape')
    await record('fueled')

    await walkTo(page, [.8, 1.1], 'ship:scout', true)
    const bench = await openBench(page)
    await transfer(page, 'deposit', 1000)
    await bench.getByLabel('Equipment recipe').selectOption(radModule)
    await bench.getByRole('button', { name: 'Start gear crafting' }).click()
    await expect(bench.locator('[data-bench-status]')).toContainText('Powered')
    await page.keyboard.press('Escape')
    for (const point of [[0, 3], [-.7, 3.9]]) await walkTo(page, point, 'ship:scout', true)
    await page.keyboard.press('e')
    await expect(panel(page)).toBeVisible()
    await transfer(page, 'deposit', 1000)
    await panel(page).getByRole('button', { name: 'Start fabrication' }).click()
    const concurrent = await record('both-consumers')
    expect(concurrent.bench.job).toBeTruthy()
    expect(concurrent.fabricator.job).toBeTruthy()
    await walkTicks(page, [], 60)
    const bothPaid = await record('both-paid')
    expect(concurrent.bench.job!.remainingTicks - bothPaid.bench.job!.remainingTicks).toBe(60)
    expect(concurrent.fabricator.job!.remainingTicks - bothPaid.fabricator.job!.remainingTicks).toBe(60)
    expect(bothPaid.core.spentJ60 - concurrent.core.spentJ60).toBe((2000 + FABRICATOR_WATTS) * 60)

    await page.keyboard.press('Escape')
    await page.getByRole('button', { name: 'Pause [P]', exact: true }).click()
    const paused = await read(page)
    await page.clock.runFor(1000)
    expect(await read(page)).toEqual(paused)
    await page.getByRole('button', { name: 'Resume [P]', exact: true }).click()
    await page.evaluate(() => {
      Object.defineProperty(document, 'hidden', { configurable: true, value: true })
      document.dispatchEvent(new Event('visibilitychange'))
    })
    const hidden = await read(page)
    await page.clock.runFor(1000)
    expect(await read(page)).toEqual(hidden)
    await page.evaluate(() => {
      Reflect.deleteProperty(document, 'hidden')
      document.dispatchEvent(new Event('visibilitychange'))
    })
    await advance(20_000)
    await page.keyboard.press('e')
    await panel(page).getByRole('button', { name: 'Claim output' }).click()
    await page.keyboard.press('Escape')
    for (const point of [[0, 3], [.8, 1.1]]) await walkTo(page, point, 'ship:scout', true)
    await openBench(page)
    await expect(bench.getByRole('button', { name: 'Start gear crafting' })).toHaveAttribute('aria-disabled', 'true')
    await bench.getByRole('button', { name: 'Claim equipment' }).click()
    const claim = (await record('claimed')).bench.claims[0]!
    await page.keyboard.press('Escape')
    for (const point of [[0, 3], [.5, 4.25]]) await walkTo(page, point, 'ship:scout', true)
    const cabinet = (await page.locator('[data-entity="scout:storage"]').boundingBox())!
    await page.mouse.click(cabinet.x + cabinet.width / 2, cabinet.y + cabinet.height / 2, { button: 'right' })
    await page.getByRole('menuitem', { name: /Manage equipment and storage/ }).click()
    const gear = panel(page, 'Equipment and storage')
    await expect.poll(async () => { await page.clock.runFor(32); return gear.isVisible() }).toBe(true)
    const newModule = gear.locator(`[data-gear-id="${claim.gearId}"]`)
    const confirmEquipment = gear.getByRole('button', { name: 'Confirm equipment change' })
    await newModule.getByRole('button', { name: 'Store module' }).click()
    await confirmEquipment.click()
    expect((await record('stored')).inventory.equipment!.instances.find(item => item.id === claim.gearId)?.location.kind).toBe('module-storage')
    await newModule.getByRole('button', { name: 'Install in slot 2' }).click()
    await confirmEquipment.click()
    expect((await record('installed')).inventory.equipment!.instances.find(item => item.id === claim.gearId)?.location)
      .toEqual({ kind: 'installed', suitId: starter[0]!.id, slot: 1 })
    await page.keyboard.press('Escape')

    await walkTo(page, [0, 4.7], 'ship:scout', true)
    await page.keyboard.press('e'); await page.clock.runFor(32)
    for (const point of shipPath.slice(1)) await walkTo(page, point, 'ship:scout', true)
    await openCockpit()
    await selectContact('Vesper')
    await expect(page.getByText('Planetary environment', { exact: true })).toBeVisible()
    await page.getByRole('button', { name: 'Take off' }).click()
    await expect(pano).toHaveAttribute('data-mode', 'taking-off')
    // Unattended flight still advances while the player stands in Scout.
    await exitCockpit()
    await advance(6500)
    expect((await ship()).mode).toBe('flying')
    await openCockpit()
    await pano.locator('canvas').focus()
    const beforeTurn = await ship()
    await page.keyboard.down('ArrowLeft')
    await advance(1167)
    await page.keyboard.up('ArrowLeft')
    expect((await ship()).yaw).toBeGreaterThan(beforeTurn.yaw)
    const throttle = page.getByRole('slider', { name: 'Throttle', exact: true })
    await throttle.fill('1')
    await advance(1000)
    await page.getByRole('button', { name: 'Zero throttle (Space)' }).click()
    await exitCockpit()
    await advance(27_000)
    await openCockpit()
    const warning = page.getByText('Planetary environment · Gear warning', { exact: true })
    await expect(warning).toBeVisible()
    await capture('hazardous-approach')
    const backend = await pano.locator('canvas').evaluate(node => {
      const gl = (node as HTMLCanvasElement).getContext('webgl2')!
      return gl.getParameter(gl.getExtension('WEBGL_debug_renderer_info')!.UNMASKED_RENDERER_WEBGL) as string
    })
    const land = page.getByRole('button', { name: 'Land', exact: true })
    await expect(land).toBeEnabled()
    await land.click(); await page.clock.runFor(32)
    await expect(pano).toHaveAttribute('data-mode', 'landing')
    await exitCockpit()
    for (let step = 0; step < 300 && (await ship()).mode === 'landing'; step++) await page.clock.fastForward(200)
    expect((await ship()).mode).toBe('grounded')
    await expect(main).toHaveAttribute('data-surface-bound', 'true')
    const touchdown = await record('touchdown')
    const site = samplePlanetEnvironment(touchdown.scout.surface, touchdown.scout.surface.definition)
    expect(site.safeStartingRegion).toBe(false)
    expect(touchdown.survival.health).toBe(200)
    await expect(page.getByLabel('Equipment and environmental condition')).toContainText('Local temperature: 24°C')

    for (const point of shipPath.slice(0, -1).reverse()) await walkTo(page, point, 'ship:scout', true)
    await page.keyboard.press('e'); await page.clock.runFor(32)
    await walkTicks(page, 's', 70)
    await expect(player(page)).toHaveAttribute('data-frame', 'planet:fixture')
    const exposed = await record('exposed')
    expect(exposed.hazards.exposure.poison).not.toBe('0/1')
    const source = page.locator(`[data-entity="${depositId}"]`)
    await expect(source).toBeVisible()
    // Walk along the stern corridor until the generated deposit is in reach.
    const gather = page.getByRole('button', { name: 'Gather', exact: true })
    for (let segment = 0; segment < 10 && !await gather.isVisible(); segment++) await walkTicks(page, 's', 12)
    await expect(gather).toBeVisible()
    await gather.click()
    await expect(page.getByLabel('Resource inventory')).toHaveAttribute('data-gathering', depositId)
    await walkTicks(page, [], 60); await page.keyboard.press('Escape')
    const gathered = await record('gathered-under-exposure')
    expect(gathered.sources[depositId]!.remainingGrams).toBe(depositFromId(depositId).initial.remainingGrams - 1000)
    expect(exact.cmp(healthValue(gathered.survival), healthValue(touchdown.survival))).toBeLessThan(0)
    const wornModule = gathered.inventory.equipment!.instances.find(item => item.id === claim.gearId)!
    const elapsed = exact.ratio(BigInt(gathered.survival.activeTicks - exposed.survival.activeTicks), 60n)
    const expectedWear = exact.mul(exact.decimal(2), elapsed)
    const temperatureRate = exact.ratio(BigInt(Math.abs(site.temperatureC - 24)), 10n)
    const damage = exact.mul(exact.sub(temperatureRate, exact.decimal(2)), elapsed)
    expect(healthValue(gathered.survival)).toEqual(exact.sub(healthValue(exposed.survival), damage))
    for (const [hazard, rate] of [['radiation', site.radiationHpPerSecond - 2], ['poison', site.toxicityHpPerSecond]] as const) {
      const expected = exact.add(exact.decode(exposed.hazards.exposure[hazard]), exact.mul(exact.decimal(rate), elapsed))
      expect(exact.decode(gathered.hazards.exposure[hazard])).toEqual(expected)
    }
    expect(wornModule.remaining).toBe(exact.encode(exact.sub(
      exact.decode(exposed.inventory.equipment!.instances.find(item => item.id === claim.gearId)!.remaining), expectedWear)))
    await capture('gathering-under-exposure')

    for (let segment = 0; segment < 20 && await player(page).getAttribute('data-frame') === 'planet:fixture'; segment++) {
      await walkTicks(page, 'w', 12)
    }
    await expect(player(page)).toHaveAttribute('data-frame', 'ship:scout')
    await expect(page.getByLabel('Equipment and environmental condition')).toContainText('Damage: 0 HP/s')
    const inside = await record('back-in-scout')
    await advance(15_200)
    const recovered = await record('recovering-safely', true)
    expect(healthValue(recovered.survival)).toEqual(healthValue(inside.survival))
    expect(recovered.inventory.equipment).toEqual(inside.inventory.equipment)
    expect(exact.cmp(exact.decode(recovered.hazards.exposure.poison), exact.decode(inside.hazards.exposure.poison))).toBeLessThan(0)
    await page.clock.setSystemTime(new Date('2026-11-04T02:00:00Z'))
    await refresh()
    const restored = await record('offline-reload')
    expect(restored.inventory).toEqual(recovered.inventory)
    const admittedTicks = restored.survival.activeTicks - recovered.survival.activeTicks
    expect(admittedTicks).toBeGreaterThanOrEqual(0)
    expect(admittedTicks).toBeLessThanOrEqual(5)
    for (const [hazard, recoveryRate] of [['radiation', 7], ['poison', 5]] as const) {
      const expected = exact.max(exact.ZERO, exact.sub(exact.decode(recovered.hazards.exposure[hazard]),
        exact.ratio(BigInt(recoveryRate * admittedTicks), 60n)))
      expect(exact.decode(restored.hazards.exposure[hazard])).toEqual(expected)
    }
    expect(restored.survival.fatigue).toBeGreaterThan(recovered.survival.fatigue)
    expect(restored.stores).toEqual(recovered.stores)
    expect(healthValue(restored.survival)).toEqual(healthValue(recovered.survival))
    // Separate depleted-durability boundary on the just-produced valid save.
    // This compresses hundreds of seconds of wear while retaining actual
    // sampling, destruction, salvage-free tombstones and subsequent crafting.
    const boundary = {
      ...restored, inventory: {
        ...restored.inventory, equipment: {
          ...restored.inventory.equipment!,
          instances: restored.inventory.equipment!.instances.map(item =>
            item.id === claim.gearId ? { ...item, remaining: '1/1000' } : item),
        },
      },
    }
    await page.evaluate(({ key, boundary }) => localStorage.setItem(key, JSON.stringify(boundary)), { key: RESOURCE_SAVE_KEY, boundary })
    await refresh()
    expect((await record('pre-worn-boundary')).inventory.equipment).toEqual(boundary.inventory.equipment)
    await walkTicks(page, 's', 70)
    await expect(player(page)).toHaveAttribute('data-frame', 'planet:fixture')
    await expect(page.getByLabel('Equipment destruction')).toContainText('Radiation module')
    const destroyed = await record('live-module-destruction')
    expect(destroyed.inventory.equipment!.instances.find(item => item.id === claim.gearId)).toEqual({
      ...boundary.inventory.equipment.instances.find(item => item.id === claim.gearId),
      location: { kind: 'destroyed' }, remaining: '0/1',
    })
    await capture('module-destruction')
    for (let segment = 0; segment < 20 && await player(page).getAttribute('data-frame') === 'planet:fixture'; segment++) {
      await walkTicks(page, 'w', 12)
    }
    await expect(player(page)).toHaveAttribute('data-frame', 'ship:scout')
    for (const point of [[0, 3], [.8, 1.1]]) await walkTo(page, point, 'ship:scout', true)
    await openBench(page)
    await bench.getByLabel('Equipment recipe').selectOption(radModule)
    await bench.getByRole('button', { name: 'Start gear crafting' }).click()
    await advance(15_200)
    await bench.getByRole('button', { name: 'Claim equipment' }).click()
    const replacement = await record('replacement-claimed')
    const newId = replacement.bench.claims.at(-1)!.gearId
    expect(newId).not.toBe(claim.gearId)
    expect(replacement.inventory.equipment!.instances.find(item => item.id === newId)?.remaining).toBe('1000/1')
    expect(replacement.inventory.equipment!.instances.find(item => item.id === claim.gearId)?.location.kind).toBe('destroyed')
    await page.keyboard.press('Escape')
    await refresh()
    expect((await record('replacement-reload')).inventory.equipment).toEqual(replacement.inventory.equipment)
    expect(errors).toEqual([])
    await info.attach('run-context.json', { body: JSON.stringify({ backend, browser: page.context().browser()!.version(), viewport: page.viewportSize(), site }), contentType: 'application/json' })
  } finally {
    await info.attach('expedition-states.json', { body: JSON.stringify(stages), contentType: 'application/json' })
  }
})
