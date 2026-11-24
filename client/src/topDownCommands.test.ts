import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createTopDownState, PLAYER_ID, stepTopDown } from './topDownRuntime.ts'
import { advanceObjectCommand, selectObjectCommand } from './topDownCommands.ts'
import { ResourceSession } from './resourceSession.ts'
import { ResourceContainerInteraction } from './resourceContainerInteraction.ts'
import { cancelGroundMovement, groundPositionClear, requestGroundMovement } from './topDownNavigation.ts'
import { interactionInReach } from './topDownInteraction.ts'
import { point2 } from './topDownFrames.ts'
import { synchronizeDoor } from './scoutDoor.ts'

const sourceId = 'source:landing:mineral:01'
function harness(view: 'interior' | 'exterior' = 'exterior') {
  let saved: string | null = null
  const storage = { getItem: () => saved, setItem: (_key: string, value: string) => { saved = value } }
  const session = new ResourceSession(storage, createTopDownState(view, true))
  const panel = new ResourceContainerInteraction(session)
  const context = () => ({ world: session.world, session, paused: false })
  let sequence = 0
  const select = (id: string, action: Parameters<typeof selectObjectCommand>[2]) => {
    session.world = selectObjectCommand(context(), id, action, ++sequence)
  }
  const tick = () => {
    session.world = session.advance(stepTopDown(session.world, [0, 0]))
    session.world = advanceObjectCommand(context(), world => panel.open(world))
    return session.world.objectCommand?.status
  }
  const finish = () => {
    const states: string[] = []
    for (let i = 0; i < 1800; i++) {
      const status = tick()!
      if (states.at(-1) !== status) states.push(status)
      if (['completed', 'cancelled', 'failed'].includes(status)) return states
    }
    assert.fail('command did not terminate')
  }
  return { session, panel, context, select, tick, finish, storage }
}
test('remote mining has deterministic full lifecycle, clear reachable arrival and exactly one start', () => {
  const h = harness(); h.session.toggleTool()
  let starts = 0
  const start = h.session.start.bind(h.session)
  h.session.start = id => { starts++; start(id) }
  h.select(sourceId, 'gather')
  assert.equal(h.session.world.objectCommand?.status, 'selected')
  assert.deepEqual(h.finish(), ['routing', 'approaching', 'revalidating', 'executing', 'completed'])
  assert.equal(starts, 1)
  const world = h.session.world, player = world.entities.find(e => e.id === PLAYER_ID)!
  assert.equal(groundPositionClear(world, player.anchor), true)
  assert.equal(interactionInReach(player.anchor, world.entities.find(e => e.id === sourceId)!, world.frames), true)
  assert.equal(selectObjectCommand(h.context(), sourceId, 'gather', 1), world)
  for (let i = 0; i < 60; i++) h.tick()
  assert.equal(starts, 1)
  assert.equal(h.session.snapshot.inventory.quantities.mineral, 1000)
  const before = JSON.stringify(h.session.snapshot.inventory)
  h.session.advance(h.session.world) // replaying a world tick cannot mine again
  assert.equal(JSON.stringify(h.session.snapshot.inventory), before)
})
test('container approach only opens confirmation; exact grams commit once, cockpit crosses doorway', () => {
  const h = harness(); h.session.toggleTool(); h.select(sourceId, 'gather'); h.finish()
  for (let i = 0; i < 60; i++) h.tick()
  h.select('scout:storage', 'container'); assert.equal(h.session.activeSourceId, null)
  assert.equal(h.finish().at(-1), 'completed')
  assert.equal(h.panel.opened, true)
  assert.equal(h.session.snapshot.inventory.quantities.mineral, 1000)
  assert.equal(h.session.snapshot.stores['scout:storage']!.quantities.mineral ?? 0, 0)
  assert.equal(h.panel.review('deposit', 'mineral', '333'), true)
  assert.equal(h.session.snapshot.inventory.quantities.mineral, 1000)
  assert.equal(h.panel.confirm(), true); assert.equal(h.panel.confirm(), false)
  assert.equal(h.session.snapshot.inventory.quantities.mineral, 667)
  h.panel.close(); h.select('scout:chair', 'cockpit')
  assert.equal(h.finish().at(-1), 'completed')
  assert.equal(h.session.world.cockpit.view, 'radar')
  assert.equal(h.session.world.cockpit.occupantId, 'player:local')
})
test('replacement, WASD, Escape/blur cancellation and reload never execute stale queued work', () => {
  for (const interruption of ['replacement', 'wasd', 'cancel', 'ground', 'reload'] as const) {
    const h = harness(); h.session.toggleTool(); h.select(sourceId, 'gather'); h.tick(); h.tick()
    assert.equal(h.session.world.objectCommand?.status, 'approaching')
    if (interruption === 'replacement') h.select('scout:chair', 'inspect')
    else if (interruption === 'wasd') h.session.world = stepTopDown(h.session.world, [1, 0])
    else if (interruption === 'ground') h.session.world = requestGroundMovement(h.session.world, { frame: 'ship:scout', point: point2(0, 4.4) })
    else if (interruption === 'reload') {
      h.session.checkpoint()
      const restored = new ResourceSession(h.storage, createTopDownState('exterior', true))
      assert.equal(restored.world.objectCommand, undefined); assert.equal(restored.world.groundCommand, undefined)
      assert.equal(restored.activeSourceId, null); continue
    } else h.session.world = cancelGroundMovement(h.session.world)
    for (let i = 0; i < 600; i++) h.tick()
    assert.equal(h.session.activeSourceId, null, interruption)
    assert.equal(h.session.snapshot.inventory.quantities.mineral ?? 0, 0, interruption)
  }
})
test('tool, target, capacity, survival, phase and site changes fail safely during approach', () => {
  const mutations: ((h: ReturnType<typeof harness>) => void)[] = [
    h => h.session.toggleTool(),
    h => { h.session.world = { ...h.session.world, entities: h.session.world.entities.filter(e => e.id !== sourceId) } },
    h => { h.session.snapshot = { ...h.session.snapshot, inventory: { ...h.session.snapshot.inventory, quantities: { mineral: 10_000 } } } },
    h => { h.session.survival = { ...h.session.survival, fatigue: 0 } },
    h => { h.session.world = { ...h.session.world, scoutLocation: { ...h.session.world.scoutLocation, siteId: 'changed' } } },
    h => { h.session.world = { ...h.session.world, scoutLocation: { ...h.session.world.scoutLocation, ship: { ...h.session.world.scoutLocation.ship, mode: 'taking-off' } } } },
  ]
  for (const mutate of mutations) {
    const h = harness(); h.session.toggleTool(); h.select(sourceId, 'gather'); h.tick(); h.tick(); mutate(h)
    h.session.world = advanceObjectCommand(h.context())
    assert.equal(h.session.world.objectCommand?.status, 'failed')
    assert.equal(h.session.world.groundCommand?.status, 'cancelled')
    assert.equal(h.session.activeSourceId, null)
  }
})
test('power/seat revocation just before execution blocks boarding; blocked and closed routes terminate', () => {
  for (const property of ['powered', 'ownerId', 'occupantId'] as const) {
    const h = harness('interior'); h.select('scout:chair', 'cockpit')
    for (let i = 0; i < 1000 && h.session.world.objectCommand?.status !== 'executing'; i++) h.tick()
    assert.equal(h.session.world.objectCommand?.status, 'executing')
    h.session.world = { ...h.session.world, cockpit: { ...h.session.world.cockpit, [property]: property === 'powered' ? false : 'other' } }
    h.tick(); assert.equal(h.session.world.objectCommand?.status, 'failed')
    assert.equal(h.session.world.cockpit.view, 'world')
  }
  const h = harness(); h.session.world = synchronizeDoor({ ...h.session.world, scoutLocation: { ...h.session.world.scoutLocation, boardingDoor: 'closed' } })
  h.select('scout:chair', 'cockpit'); assert.equal(h.finish().at(-1), 'failed')
  assert.equal(h.session.world.cockpit.view, 'world')
})
test('door executes once and changed door intent is rejected', () => {
  const h = harness(); h.select('scout:entry', 'door'); assert.equal(h.finish().at(-1), 'completed')
  assert.equal(h.session.world.scoutLocation.boardingDoor, 'closed')
  const world = h.session.world
  assert.equal(selectObjectCommand(h.context(), 'scout:entry', 'door', 1), world)
  for (let i = 0; i < 10; i++) h.tick()
  assert.equal(h.session.world.scoutLocation.boardingDoor, 'closed')
  h.select('scout:entry', 'door')
  h.session.world = synchronizeDoor({ ...h.session.world, scoutLocation: { ...h.session.world.scoutLocation, boardingDoor: 'open' } })
  h.tick(); assert.equal(h.session.world.objectCommand?.status, 'failed')
})
test('a door closed during approach blocks the route without delayed cockpit entry', () => {
  const h = harness(); h.select('scout:chair', 'cockpit'); h.tick(); h.tick()
  assert.equal(h.session.world.objectCommand?.status, 'approaching')
  h.session.world = synchronizeDoor({ ...h.session.world, scoutLocation: { ...h.session.world.scoutLocation, boardingDoor: 'closed' } })
  assert.equal(h.finish().at(-1), 'failed')
  assert.equal(h.session.world.cockpit.view, 'world')
})
test('invalid/replayed receipts and absent targets never dispatch; pause cancels a pending command', () => {
  const h = harness()
  for (const sequence of [NaN, Infinity, 0, -1, 1.5]) assert.equal(selectObjectCommand(h.context(), 'scout:entry', 'door', sequence), h.session.world)
  h.select('missing', 'door'); assert.equal(h.session.world.objectCommand?.status, 'failed')
  h.select('scout:chair', 'cockpit')
  h.session.world = advanceObjectCommand({ ...h.context(), paused: true })
  assert.equal(h.session.world.objectCommand?.status, 'cancelled')
  for (let i = 0; i < 120; i++) h.tick()
  assert.equal(h.session.world.cockpit.view, 'world')
})
