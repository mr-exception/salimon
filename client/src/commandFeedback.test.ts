import { test } from 'node:test'
import assert from 'node:assert/strict'
import { createTopDownState } from './topDownRuntime.ts'
import { selectObjectCommand } from './topDownCommands.ts'
import { cancelGroundMovement, requestGroundMovement } from './topDownNavigation.ts'
import { point2 } from './topDownFrames.ts'
import { commandFeedback } from './commandFeedback.ts'

test('command feedback distinguishes queued, moving, mining, cancelled and failed work', () => {
  const initial = createTopDownState()
  assert.equal(commandFeedback(initial), null)
  const selected = selectObjectCommand({ world: initial, paused: false }, 'scout:chair', 'inspect', 1)
  assert.match(commandFeedback(selected)!.text, /Queued/)
  assert.equal(commandFeedback(selected)!.cancellable, true)
  assert.match(commandFeedback(cancelGroundMovement(selected))!.text, /cancelled/)
  assert.match(commandFeedback(selected, 'scout:chair')!.text, /Mining/)
  const failed = requestGroundMovement(initial, { frame: 'ship:scout', point: point2(1000, 1000) })
  assert.equal(commandFeedback(failed)!.failed, true)
  assert.equal(commandFeedback(failed)!.cancellable, false)
})

test('new ground routes supersede terminal object feedback without discarding replay receipts', () => {
  const initial = createTopDownState()
  const selected = selectObjectCommand({ world: initial, paused: false }, 'scout:chair', 'inspect', 1)
  const completed = { ...selected, objectCommand: { ...selected.objectCommand!, status: 'completed' as const, message: 'Old inspection.' } }
  const next = requestGroundMovement(completed, { frame: 'ship:scout', point: point2(1000, 1000) })
  assert.equal(next.objectCommand?.sequence, 1)
  assert.match(commandFeedback(next)!.text, /unreachable/)
  assert.doesNotMatch(commandFeedback(next)!.text, /Old inspection/)
  const work = { ...completed, objectCommand: { ...completed.objectCommand!, actionId: 'gather' as const } }
  assert.equal(commandFeedback(work, null, 'Inventory full: weight limit.')!.failed, true)
  assert.match(commandFeedback(work, null, 'Gathering stopped.')!.text, /stopped/)
})
