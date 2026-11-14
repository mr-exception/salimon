import { test, expect } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { enterCockpit } from '../src/cockpit.ts'
import { NavigationRuntime } from '../src/navigationRuntime.ts'
import { SAMPLE_BODIES } from '../src/sampleUniverse.ts'
import { physicalRadiusM } from '../src/contact.ts'
import { translate } from '../src/coordinates.ts'
import { normal, scale } from '../src/math.ts'
import { surfaceScoutLocation } from '../src/scoutLocation.ts'
import { startingSurface, type SurfaceAddress } from '../src/surfaceSite.ts'
import { point2 } from '../src/topDownFrames.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
import { boot, walkTo, walkTicks } from './worldJourney'

// Synthetic interplanetary approaches use the actual authorized domain planner.
// Save the admitted trajectory shortly before automatic touchdown; browser input
// then exercises production frame/materialization, doorway and relaunch paths.
function landingApproachSave(raw: string | null, surface: SurfaceAddress): string {
  let saved = raw
  const storage = { getItem: () => saved, setItem: (_: string, value: string) => { saved = value } }
  let world = createTopDownState('interior', true)
  const session = new ResourceSession(storage, world, undefined, undefined, () => Date.parse('2026-11-04T00:00:00Z'))
  expect(session.error).toBe('')
  world = session.world
  world = { ...world, entities: world.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout', point: point2(0, -3.1) } } : e), selectedId: 'scout:chair' }
  world.scoutLocation.boardingDoor = 'closed'
  session.world = enterCockpit(world)
  const command = (action: Parameters<ResourceSession['transitionScout']>[0]) => session.transitionScout(action, session.snapshot.revision, session.world.cockpit)
  if (world.scoutLocation.binding) {
    if (raw === null) expect(session.transfer({ revision: session.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1000 })).toBe(true)
    expect(command({ kind: 'begin-takeoff' })).toBe(true)
    const navigation = new NavigationRuntime(SAMPLE_BODIES, world.scoutLocation)
    navigation.advance(4); navigation.advance(2)
  }
  const target = surfaceScoutLocation(surface).ship
  world.scoutLocation.ship = { ...world.scoutLocation.ship, mode: 'flying', contact: null,
    position: translate(target.position, scale(normal(surface.radialM), 100)), velocityMps: target.velocityMps }
  world.scoutLocation.scanner = { lastTick: world.scoutLocation.ship.tick, pulse: Math.floor(world.scoutLocation.ship.tick / 300), selected: surface.bodyId, contactIds: [surface.bodyId] }
  expect(command({ kind: 'begin-landing', bodyId: surface.bodyId, surface })).toBe(true)
  const navigation = new NavigationRuntime(SAMPLE_BODIES, world.scoutLocation), plan = world.scoutLocation.landing!
  while (navigation.clock.state.tick < plan.startTick + plan.durationTicks - 180) navigation.advance(1 / 60)
  session.checkpoint()
  expect(session.error).toBe('')
  return saved!
}

test('SURFACE-842 production interior, moon/Caligo exterior, relaunch and authored depleted return', async ({ page }, info) => {
  test.setTimeout(300000)
  const captures: Record<string, unknown> = {}, main = page.locator('main')
  const ship = async () => JSON.parse((await main.getAttribute('data-ship'))!)
  const saved = async () => { await page.evaluate(() => window.dispatchEvent(new Event('pagehide'))); return page.evaluate(key => localStorage.getItem(key)!, RESOURCE_SAVE_KEY) }
  const capture = async (name: string) => {
    const a = await page.screenshot({ path: info.outputPath(`${name}.png`) }), b = await page.screenshot({ path: info.outputPath(`${name}-repeat.png`) })
    const result = compareImages(a, b)
    await info.attach(`${name}-repeat.json`, { body: JSON.stringify({ ...result, diff: undefined }), contentType: 'application/json' })
    await info.attach(`${name}-diff.png`, { body: result.diff, contentType: 'image/png' })
    expect(result.pass).toBe(true)
    captures[name] = JSON.parse(await saved())
  }
  const moon: SurfaceAddress = { bodyId: 'vesper-moon', radialM: [physicalRadiusM(SAMPLE_BODIES[2]!), 0, 0], yaw: -.12 }
  const caligo: SurfaceAddress = { bodyId: 'caligo', radialM: [0, 0, -physicalRadiusM(SAMPLE_BODIES[3]!)], yaw: -.12 }
  const initial = landingApproachSave(null, moon)
  await page.addInitScript(({ key, value }) => { if (!localStorage.getItem(key)) localStorage.setItem(key, value) }, { key: RESOURCE_SAVE_KEY, value: initial })
  await page.clock.install({ time: new Date('2026-11-04T00:00:00Z') }); await page.clock.pauseAt(new Date('2026-11-04T00:00:01Z'))
  await page.goto('/?flightFixture=1')
  await expect.poll(async () => { await page.clock.runFor(50); return main.getAttribute('data-surface-bound') }).toBe('false')
  let launchRaw = initial
  for (const [name, target] of [['moon', moon], ['caligo', caligo], ['authored', startingSurface()]] as const) {
    if (name !== 'moon') {
      launchRaw = landingApproachSave(launchRaw, target)
      await page.evaluate(({ key, value }) => localStorage.setItem(key, value), { key: RESOURCE_SAVE_KEY, value: launchRaw })
      await page.reload(); await page.clock.runFor(64)
    }
    await expect.poll(async () => { await page.clock.runFor(50); return page.locator('.top-down-scene').getAttribute('data-render-ready') }).toBe('true')
    expect((await ship()).mode).toBe('landing')
    await expect(page.locator('[data-entity][data-frame="planet:fixture"]')).toHaveCount(0)
    const playerAnchor = await page.locator('[data-entity="person:player"]').getAttribute('data-position')
    await page.keyboard.press('e'); await page.clock.runFor(64)
    await page.getByRole('button', { name: 'finish-landing', exact: true }).click(); await page.clock.runFor(64)
    expect((await ship()).mode).toBe('landing')
    for (let i = 0; i < 35; i++) await page.clock.fastForward(100)
    expect((await ship()).mode).toBe('grounded')
    expect((await ship()).contact.bodyId).toBe(target.bodyId)
    await expect(main).toHaveAttribute('data-surface-bound', 'true')
    await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
    await expect(main).toHaveAttribute('data-cockpit-view', 'world')
    await expect(main).toHaveAttribute('data-player-mode', 'standing')
    expect(await page.locator('[data-entity="person:player"]').getAttribute('data-position')).toBe(playerAnchor)
    if (name !== 'authored') await expect(page.locator('[data-source-kind]')).toHaveCount(0)
    else {
      await expect(page.locator('[data-source-kind]')).toHaveCount(2)
      await expect(page.locator('[data-entity="source:landing:mineral:01"]')).toHaveAttribute('data-remaining-grams', '59000')
    }
    for (const p of [[0, -1.65], [1.35, -1.65], [1.35, .45], [0, .45], [0, 2], [0, 4.7]]) await walkTo(page, p, 'ship:scout', true)
    await page.keyboard.press('e'); await page.clock.runFor(32)
    await walkTicks(page, 's', 70)
    await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
    await page.clock.runFor(1600)
    await capture(`${name}-exterior`)
    const grounded = JSON.parse(await saved())
    await page.reload(); await page.clock.runFor(64)
    await expect.poll(async () => { await page.clock.runFor(50); return main.getAttribute('data-surface-bound') }).toBe('true')
    expect((await ship()).position).toEqual(grounded.scout.ship.position)
    await expect(page.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
    await walkTicks(page, 'w', 70)
    await walkTo(page, [0, 4.7], 'ship:scout', true)
    await page.keyboard.press('e'); await page.clock.runFor(32)
    for (const p of [[0, 2], [0, .45], [1.35, .45], [1.35, -1.65], [0, -1.65], [0, -3.1]]) await walkTo(page, p, 'ship:scout', true)
    await page.keyboard.press('e'); await page.clock.runFor(64)
    await page.getByRole('button', { name: 'begin-takeoff', exact: true }).click()
    for (let i = 0; i < 65; i++) await page.clock.fastForward(100)
    expect((await ship()).mode).toBe('flying')
    await page.getByRole('button', { name: 'Exit cockpit' }).click(); await page.clock.runFor(64)
    await expect(page.locator('[data-entity][data-frame="planet:fixture"]')).toHaveCount(0)
    await page.clock.runFor(1600)
    await capture(`${name}-relaunched`)
    launchRaw = await saved()
    expect(JSON.parse(launchRaw).sources['source:landing:mineral:01'].remainingGrams).toBe(59000)
  }
  // The shipping build consumes the same generated-site save, without debug controls.
  const built = await page.context().newPage()
  const moonSave = captures['moon-exterior'] as object
  await built.addInitScript(({ key, value }) => localStorage.setItem(key, JSON.stringify(value)), { key: RESOURCE_SAVE_KEY, value: moonSave })
  await boot(built, '', 'http://127.0.0.1:4175')
  await expect(built.locator('[data-source-kind]')).toHaveCount(0)
  await expect(built.locator('[data-entity="person:player"]')).toHaveAttribute('data-frame', 'planet:fixture')
  await built.screenshot({ path: info.outputPath('built-moon-exterior.png') })
  await built.close()
  await info.attach('journey.json', { body: JSON.stringify({ captures, viewport: { width: 1280, height: 800 }, dpr: 1, epoch: '2026-11-04T00:00:00Z', input: 'Playwright keyboard/click; source-session admitted landing plans from synthetic approaches', seed: 'sampleUniverse / surface:v1' }), contentType: 'application/json' })
})
