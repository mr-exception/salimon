import { test } from 'node:test'
import assert from 'node:assert/strict'
import { controlsCockpit, enterCockpit, leaveCockpit, LOCAL_PLAYER, setCockpitPower } from './cockpit.ts'
import { SCOUT_ANCHORS } from './scoutLayout.ts'
import { point2 } from './topDownFrames.ts'
import { createTopDownState, drawList, PLAYER_ID, requestInteraction, stepTopDown } from './topDownRuntime.ts'
import { allowedMove } from './topDownInteraction.ts'
const approach = () => {
  const s = createTopDownState('interior', true)
  return { ...s, selectedId: 'scout:chair', entities: s.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(0, -3.1) } } : e) }
}
test('one owner chair, physically seated avatar, no walking and no second occupant', () => {
  const s = approach(), seated = requestInteraction(s)
  assert.equal(seated.cockpit.occupantId, LOCAL_PLAYER)
  assert.equal(seated.cockpit.view, 'radar')
  assert.equal(controlsCockpit(seated.cockpit, LOCAL_PLAYER), true)
  assert.equal(controlsCockpit(seated.cockpit, 'visitor'), false)
  assert.deepEqual(seated.entities.find(e => e.id === PLAYER_ID)!.anchor, SCOUT_ANCHORS.chair)
  assert.deepEqual(stepTopDown(seated, [1, 1]).player, seated.player)
  assert.deepEqual(stepTopDown(seated, [1, 1]).entities, seated.entities)
  assert.equal(drawList(seated).find(e => e.entity.id === PLAYER_ID)!.entity.layer, 'cues')
  assert.equal(enterCockpit(seated, 'visitor').cockpit, seated.cockpit)
  const occupied = { ...s, cockpit: { ...s.cockpit, occupantId: 'visitor' } }
  assert.equal(enterCockpit(occupied).cockpit, occupied.cockpit)
  assert.equal(enterCockpit(createTopDownState('exterior', true)).cockpit.view, 'world')
})
test('voluntary exit stands safely; blackout retains seat; restoration requires explicit interaction', () => {
  const s = approach(), seated = enterCockpit(s), standing = leaveCockpit(seated)
  assert.equal(standing.player.mode, 'standing'); assert.equal(standing.cockpit.occupantId, null)
  assert.equal(standing.cockpit.view, 'world')
  assert.equal(allowedMove(standing.entities.find(e => e.id === PLAYER_ID)!.anchor, [0, .01], standing.entities, standing.frames), 1)
  const blackout = setCockpitPower(seated, false)
  assert.equal(blackout.player.mode, 'seated'); assert.equal(blackout.cockpit.occupantId, LOCAL_PLAYER)
  assert.equal(blackout.cockpit.view, 'world'); assert.equal(controlsCockpit(blackout.cockpit, LOCAL_PLAYER), false)
  assert.equal(enterCockpit(blackout).cockpit.view, 'world')
  assert.equal(leaveCockpit(blackout).player.mode, 'standing')
  const restored = stepTopDown(setCockpitPower(blackout, true), [0, 0])
  assert.equal(restored.cockpit.view, 'world'); assert.equal(restored.player.mode, 'seated')
  assert.equal(requestInteraction(restored).cockpit.view, 'radar')
  assert.deepEqual(restored.frames, s.frames)
})
