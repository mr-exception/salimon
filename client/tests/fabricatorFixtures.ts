import { expect, type Page } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { point2 } from '../src/topDownFrames.ts'
import { FABRICATOR_RECIPE_ID } from '../src/scoutFabricator.ts'

export const origin = 'http://127.0.0.1:4175'
export const panel = (page: Page, name = 'Fabricator') => page.getByRole('dialog', { name, exact: true })
export const saved = (page: Page) => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
export async function seed(page: Page, raw: string) {
  await page.addInitScript(({ key, raw }) => { if (!localStorage.getItem(key)) localStorage.setItem(key, raw) }, { key: RESOURCE_SAVE_KEY, raw })
}
export async function transfer(page: Page, direction: 'deposit' | 'withdraw', grams: number) {
  await page.getByRole('textbox', { name: 'Amount in grams' }).fill(String(grams))
  await page.getByRole('button', { name: direction === 'deposit' ? 'Review deposit' : 'Review withdrawal' }).click()
  await page.getByRole('button', { name: `Confirm ${direction}` }).click()
}
export function fixture(state: 'journey' | 'empty' | 'ready' | 'powered' | 'paused' | 'complete' | 'full' | 'marker') {
  const initial = createTopDownState('exterior', true)
  const world = state === 'journey' || state === 'marker' ? initial : { ...initial, entities: initial.entities.map(e => e.id === PLAYER_ID ? { ...e, anchor: { frame: 'ship:scout' as const, point: point2(-.7, 3.9) } } : e) }
  let raw: string | null = null
  const s = new ResourceSession({ getItem: () => raw, setItem: (_, v) => { raw = v } }, world, undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
  // A valid prior expedition spent its reserve. The journey itself gathers,
  // fuels, starts, exhausts power and resumes only through production controls.
  if (state === 'journey' || state === 'paused') s.advance({ ...s.world, tick: 1 }, { watts: s.snapshot.core.reserveJ60 - (state === 'journey' ? 5000 * 60 : 0) })
  else if (state === 'empty') {
    expect(s.transfer({ revision: 0, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 1 })).toBe(true)
  } else s.checkpoint()
  if (!['journey', 'empty'].includes(state)) {
    expect(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 2000 })).toBe(true)
    expect(s.depositFabricatorInput('mineral', 2000, s.snapshot.revision)).toBe(true)
    if (state !== 'ready') {
      expect(s.startFabricator(FABRICATOR_RECIPE_ID, s.snapshot.revision)).toBe(true)
      s.advance({ ...s.world, tick: s.world.tick + (state === 'complete' || state === 'full' || state === 'marker' ? 1200 : 1) })
    }
    if (state === 'marker') expect(s.claimFabricator(1, s.snapshot.revision)).toBe(true)
    if (state === 'full') expect(s.transfer({ revision: s.snapshot.revision, from: { kind: 'source', id: 'source:landing:mineral:01' }, to: { kind: 'player' }, resourceId: 'mineral', grams: 10000 })).toBe(true)
  }
  return raw!
}
