import { test, expect } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { point2 } from '../src/topDownFrames.ts'
import { boot, walkTicks, scene } from './worldJourney.ts'
test('HAZ-917 production physical sampling wears gear, pauses and restores exact committed state',async({page},info)=>{
 let raw:string|null=null
 const initial=createTopDownState('exterior',true)
 const world={...initial,entities:initial.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'planet:fixture' as const,point:point2(129,0)}}:e)}
 const s=new ResourceSession({getItem:()=>raw,setItem:(_,v)=>{raw=v}},world,undefined,undefined,()=>Date.parse('2026-11-04T00:00:01Z'))
 s.upgrade(10000,12);expect(s.error).toBe('')
 await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw!)},{key:RESOURCE_SAVE_KEY,raw})
 await boot(page,'','http://127.0.0.1:4175')
 const read=()=>page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
 const first=await read();await walkTicks(page,[],20)
 const worn=await read();expect(worn.survival.health).toBeLessThan(first.survival.health)
 expect(worn.inventory.equipment.instances[1].remaining).not.toBe(first.inventory.equipment.instances[1].remaining)
 await page.getByRole('button',{name:'Pause [P]',exact:true}).click()
 const paused=await read();await page.clock.runFor(1000);expect(await read()).toEqual(paused)
 await page.reload();await expect.poll(async()=>{await page.clock.runFor(32);return scene(page).getAttribute('data-render-ready')}).toBe('true')
 const restored=await read();expect(restored.survival.health).toBeLessThanOrEqual(paused.survival.health)
 expect(restored.inventory.equipment.instances[1].id).toBe(paused.inventory.equipment.instances[1].id)
 await page.screenshot({path:info.outputPath('live-exposure.png')})
 await info.attach('live-hazard-state.json',{body:JSON.stringify({first,worn,paused,restored}),contentType:'application/json'})
})

test('destroyed installed module disappears from the production pilot', async ({ page }, info) => {
  let raw = ''
  const initial = createTopDownState('exterior', true)
  const world = { ...initial, entities: initial.entities.map(entity => entity.id === PLAYER_ID
    ? { ...entity, anchor: { frame: 'planet:fixture' as const, point: point2(129, 0) } }
    : entity) }
  const session = new ResourceSession({ getItem: () => raw || null, setItem: (_, value) => { raw = value } },
    world, undefined, undefined, () => Date.parse('2026-11-04T00:00:01Z'))
  session.upgrade(10000, 12)
  expect(session.error).toBe('')
  const save = JSON.parse(raw)
  const module = save.inventory.equipment.instances[1]
  module.remaining = '1/1000'
  await page.addInitScript(({ key, snapshot }) => {
    if (!localStorage.getItem(key)) localStorage.setItem(key, snapshot)
  }, { key: RESOURCE_SAVE_KEY, snapshot: JSON.stringify(save) })
  await boot(page, '', 'http://127.0.0.1:4175')
  await walkTicks(page, [], 20)
  await expect(page.locator('[data-character]')).toHaveAttribute('data-suit-equipped', 'true')
  await expect(page.locator('[data-character-module="module:temperature:t1"]')).toHaveCount(0)
  await expect(page.getByLabel('Equipment destruction')).toContainText('Temperature module')
  const destroyed = await page.evaluate(({ key, id }) => JSON.parse(localStorage.getItem(key)!)
    .inventory.equipment.instances.find((item: { id: string }) => item.id === id),
  { key: RESOURCE_SAVE_KEY, id: module.id })
  expect(destroyed).toEqual({ ...module, remaining: '0/1', location: { kind: 'destroyed' } })
  await page.screenshot({ path: info.outputPath('pilot-module-destroyed.png') })
})
