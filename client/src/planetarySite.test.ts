import { test } from 'node:test'
import assert from 'node:assert/strict'
import { relativeM } from './coordinates.ts'
import { point2, rotate2, type Anchor } from './topDownFrames.ts'
import { allowedMove, interactionInReach, nearbyObject } from './topDownInteraction.ts'
import { planetaryEntities, STARTING_SITE } from './planetarySite.ts'
import { createTopDownState, PLAYER_ID, requestInteraction, stepTopDown } from './topDownRuntime.ts'

const player = (state: ReturnType<typeof createTopDownState>) => state.entities.find(e => e.id === PLAYER_ID)!
test('default world starts outside at a stable site anchor, independent of ship/camera pose', () => {
  const first = createTopDownState(), second = createTopDownState()
  assert.deepEqual(first, second)
  assert.deepEqual(player(first).anchor, STARTING_SITE.spawn)
  assert.equal(player(first).anchor.frame, first.frames.planet.id)
  const changed = { ...first, frames: { ...first.frames, ship: { ...first.frames.ship, origin: point2(100, 200), angle: 1 } } }
  assert.deepEqual(changed.entities.filter(e => e.anchor.frame === 'planet:fixture'), first.entities.filter(e => e.anchor.frame === 'planet:fixture'))
  assert.equal(new Set(first.entities.map(e => e.id)).size, first.entities.length)
  // A spawn must be clear in every direction; do not start in a rock/organism.
  for (let i = 0; i < 8; i++) assert.equal(allowedMove(player(first).anchor, rotate2([.1, 0], i * Math.PI / 4), first.entities, first.frames), 1)
})
test('both fixed source archetypes are reachable, collidable and inert under repeated inspection', () => {
  const state = createTopDownState(), sources = state.entities.filter(e => e.source)
  assert.deepEqual(sources.map(e => e.source!.kind).sort(), ['biological', 'mineral'])
  for (const source of sources) {
    assert.equal(source.anchor.frame, 'planet:fixture'); assert.equal(source.source!.siteId, STARTING_SITE.id)
    assert.ok(source.footprint); assert.ok(source.interaction)
    const [sx, sy] = source.anchor.point.offsetM
    const approach: Anchor = { frame: 'planet:fixture', point: point2(sx, sy - source.footprint!.halfHeight - .6) }
    assert.ok(interactionInReach(approach, source, state.frames))
    assert.equal(nearbyObject(approach, state.entities, state.frames)?.id, source.id)
    assert.ok(allowedMove(approach, [0, 4], state.entities, state.frames) < 1)
    let inspected: ReturnType<typeof createTopDownState> = { ...state, entities: state.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: approach } : e) }
    for (let i = 0; i < 5; i++) inspected = requestInteraction(inspected, source.id)
    assert.equal(inspected.player.targetId, source.id)
    assert.deepEqual(inspected.entities, state.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: approach } : e))
    assert.deepEqual(planetaryEntities(), createTopDownState().entities.filter(e => e.anchor.frame === 'planet:fixture' && e.id !== PLAYER_ID))
  }
  assert.equal(requestInteraction(state, sources[0]!.id).player.targetId, null, 'distant request rejected')
})
test('planet walking stops at source and rock footprints without changing fixed identities', () => {
  const initial = createTopDownState()
  const source = initial.entities.find(e => e.source?.kind === 'mineral')!
  let state: ReturnType<typeof createTopDownState> = { ...initial, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'planet:fixture' as const, point: point2(-7, 0) } } : e) }
  for (let i = 0; i < 120; i++) state = stepTopDown(state, [-1, 0])
  assert.ok(Math.abs(player(state).anchor.point.offsetM[0] - (-7.34)) < 1e-7)
  assert.equal(state.selectedId, source.id)
  assert.deepEqual(state.entities.filter(e => e.id !== PLAYER_ID), initial.entities.filter(e => e.id !== PLAYER_ID))
  const rock = initial.entities.find(e => e.id === 'rock:west')!
  const from: Anchor = { frame: 'planet:fixture', point: point2(-10, -6) }
  assert.ok(allowedMove(from, [0, 8], [rock], initial.frames) < 1)
  assert.deepEqual(relativeM(rock.anchor.point, point2(-10, -3)), [0, 0, 0])
})
