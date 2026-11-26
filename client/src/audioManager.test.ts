import assert from 'node:assert/strict'
import test from 'node:test'
import { AUDIO_PREFERENCES_KEY, AudioManager, DEFAULT_VOLUMES, readAudioPreferences, spatialMix, storeAudioPreferences, type AudioBackend } from './audio/audioManager.ts'

function harness() {
  let now = 0
  const events: unknown[] = []
  const pending: ((sample: string) => void)[] = []
  const endings: (() => void)[] = []
  const backend: AudioBackend<string> = {
    async unlock() { events.push('unlock') },
    setHidden(hidden) { events.push({ hidden }) },
    setPreferences(preferences) { events.push(preferences) },
    load(id) { events.push(id); return new Promise(resolve => pending.push(resolve)) },
    start(buffer, channel, mix, loop, end) {
      events.push({ buffer, channel, mix, loop })
      endings.push(end)
      return { stop: seconds => events.push({ stop: seconds }) }
    },
    dispose() { events.push('disposal') },
  }
  const normal = { volumes: DEFAULT_VOLUMES, mono: false }
  const audio = new AudioManager(backend, normal, () => now)
  const world = { active: true, hidden: false, view: 'world' }
  audio.setScope(world)
  return { audio, events, pending, endings, world, normal, advance: (ms: number) => { now += ms } }
}

test('preferences validate on load and use a key independent of gameplay saves', () => {
  const items = new Map([['salimon:expedition:v1', 'game'], [AUDIO_PREFERENCES_KEY, '{bad json']])
  const storage = { getItem: (key: string) => items.get(key) ?? null, setItem: (key: string, value: string) => items.set(key, value) }
  assert.deepEqual(readAudioPreferences(storage), { volumes: DEFAULT_VOLUMES, mono: false })
  items.set(AUDIO_PREFERENCES_KEY, JSON.stringify({ volumes: { master: 150, warnings: -1, music: '40' }, mono: true }))
  const value = readAudioPreferences(storage)
  assert.deepEqual(value, { volumes: { ...DEFAULT_VOLUMES, warnings: 0 }, mono: true })
  storeAudioPreferences(storage, value)
  assert.deepEqual(readAudioPreferences(storage), value)
  assert.equal(items.get('salimon:expedition:v1'), 'game')
})

test('requires unlock, limits pending and playing instances, and deduplicates warning cadence', async () => {
  const { audio, events, pending, endings, advance, normal } = harness()
  assert.equal(await audio.play('conditionWarning'), false)
  await Promise.all([audio.unlock(), audio.unlock()])
  assert.equal(events.filter(event => event === 'unlock').length, 1)
  const one = audio.play('conditionWarning')
  assert.equal(await audio.play('conditionWarning'), false)
  pending.shift()!('warning')
  assert.equal(await one, true)
  assert.equal(await audio.play('conditionWarning'), false)
  endings.shift()!()
  advance(1499)
  assert.equal(await audio.play('conditionWarning'), false)
  advance(1)
  const allowed = audio.play('conditionWarning')
  pending.shift()!('warning')
  assert.equal(await allowed, true)
  audio.setPreferences({ ...normal, volumes: { ...DEFAULT_VOLUMES, warnings: 0 } })
  assert.equal(await audio.play('conditionWarning'), false)
})

test('pause, visibility, cockpit, death and unmount discard stale work and reset cleanly', async () => {
  const { audio, events, pending, world } = harness()
  await audio.unlock()
  for (const scope of [{ ...world, active: false }, { ...world, hidden: true },
    { ...world, view: 'radar' }, { ...world, view: 'ended', active: false }]) {
    const stale = audio.play('boardingDoor')
    audio.setScope(scope)
    pending.shift()!('door')
    assert.equal(await stale, false)
    audio.setScope(world)
  }
  assert.ok(events.some(event => typeof event === 'object' && event !== null && 'hidden' in event && event.hidden))
  const playing = audio.play('boardingDoor', { distanceM: 4, rightM: 2, obstruction: .5 })
  pending.shift()!('door')
  assert.equal(await playing, true)
  assert.deepEqual(events.at(-1), { buffer: 'door', channel: 'machinery', mix: spatialMix({ distanceM: 4, rightM: 2, obstruction: .5 }), loop: false })
  assert.deepEqual(spatialMix({ distanceM: 0, rightM: -3 }), { gain: 1, pan: -1, lowpassHz: 20000 })
  audio.setScope({ ...world, active: false })
  assert.deepEqual(events.at(-1), { stop: .12 })
  audio.setScope(world)
  const duringUnmount = audio.play('boardingDoor')
  audio.dispose()
  audio.dispose()
  pending.shift()!('door')
  assert.equal(await duringUnmount, false)
  assert.equal(events.filter(event => event === 'disposal').length, 1)
  assert.equal(await harness().audio.play('boardingDoor'), false)
})
