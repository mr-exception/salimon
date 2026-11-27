import assert from 'node:assert/strict'
import test from 'node:test'
import { createTopDownState } from './topDownRuntime.ts'
import { point2 } from './topDownFrames.ts'
import { isScoutModule, scoutArtObstructs, scoutWallJoins } from './rendering/scoutArt.ts'

test('wall joins are derived from touching domain footprints without closing the walkway', () => {
  const { entities } = createTopDownState('interior', true)
  const left = entities.find(entity => entity.id === 'wall:cockpit-left')!
  const port = entities.find(entity => entity.id === 'wall:port')!
  const joins = scoutWallJoins(entities)
  assert.deepEqual(joins.get(left.id), [true, false])
  assert.deepEqual(joins.get('wall:cockpit-right'), [false, true])
  const moved = { ...port, anchor: { ...port.anchor, point: point2(-5, .5) } }
  assert.deepEqual(scoutWallJoins(entities.map(entity => entity === port ? moved : entity)).get(left.id), [false, false])
  assert.equal(left.footprint?.halfWidth, .85)
  assert.equal(isScoutModule(left), true)
  assert.equal(isScoutModule(entities.find(entity => entity.id === 'person:player')!), false)
})

test('art occlusion follows foreground overhang and preserves the boarding cue', () => {
  const { entities } = createTopDownState('interior', true)
  const crate = entities.find(entity => entity.id === 'fixture:crate')!
  const door = entities.find(entity => entity.id === 'scout:entry')!
  assert.equal(scoutArtObstructs(crate, [.17, -.95]), true)
  assert.equal(scoutArtObstructs(crate, [.17, .95]), false)
  assert.equal(scoutArtObstructs(crate, [.17, -1]), false)
  assert.equal(scoutArtObstructs(crate, [.83, -.7]), false)
  assert.equal(scoutArtObstructs({ ...door, appearance: 'door', footprint: { halfWidth: 1.5, halfHeight: .15 } }, [0, -.5]), false)
})
