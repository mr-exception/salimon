import { test, expect } from '@playwright/test'
import { ResourceSession, RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { point2 } from '../src/topDownFrames.ts'
import { boot, walkTicks } from './worldJourney.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
function fixture(){
 let raw:string|null=null;const initial=createTopDownState('exterior',true)
 const world={...initial,entities:initial.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:'ship:scout' as const,point:point2(.8,1.1)}}:e)}
 const s=new ResourceSession({getItem:()=>raw,setItem:(_,v)=>{raw=v}},world,undefined,undefined,()=>Date.parse('2026-11-04T00:00:01Z'))
 expect(s.transfer({revision:0,from:{kind:'source',id:'source:landing:mineral:01'},to:{kind:'player'},resourceId:'mineral',grams:500})).toBe(true)
 return raw!
}
test('BENCH-918 production reachable console reserves, crafts, claims and reloads unique equipment',async({page},info)=>{
 const raw=fixture();await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw})
 await boot(page,'','http://127.0.0.1:4175')
 const target=page.locator('[data-entity="scout:gear-bench"]');await target.click({button:'right'})
 await page.getByRole('menuitem',{name:/Use Gear Crafting Bench/}).click()
 const dialog=page.getByRole('dialog',{name:'Gear Crafting Bench',exact:true});await expect.poll(async()=>{await page.clock.runFor(32);return dialog.isVisible()}).toBe(true)
 await dialog.getByRole('textbox',{name:'Amount in grams'}).fill('500')
 await dialog.getByRole('button',{name:'Review deposit'}).click();await dialog.getByRole('button',{name:'Confirm deposit'}).click()
 await dialog.getByLabel('Equipment recipe').selectOption('module:radiation:t1')
 await dialog.getByRole('button',{name:'Start gear crafting'}).click()
 await expect(dialog.locator('[data-bench-status]')).toContainText('Powered')
 await expect(target.locator('[data-scout-machine="bench"]')).toHaveAttribute('data-machine-state', 'active')
 await walkTicks(page,[],900)
 await expect(dialog.locator('[data-bench-tray]')).toContainText('Radiation module')
 await expect(target.locator('[data-scout-machine="bench"]')).toHaveAttribute('data-machine-state', 'output')
 await dialog.getByRole('button',{name:'Claim equipment'}).click()
 const read=()=>page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
 const claimed=await read();expect(claimed.bench.claims).toHaveLength(1)
 const gear=claimed.inventory.equipment.instances.find((g:{id:string})=>g.id===claimed.bench.claims[0].gearId)
 expect(gear.remaining).toBe('1000/1');expect(gear.location.kind).toBe('inventory')
 const original=await dialog.screenshot({path:info.outputPath('bench-claimed.png')});const repeat=await dialog.screenshot({path:info.outputPath('bench-repeat.png')});expect(compareImages(original,repeat).pass).toBe(true)
 await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0)
 await page.reload();await page.clock.runFor(200)
 expect((await read()).inventory.equipment).toEqual(claimed.inventory.equipment)
 await info.attach('bench-state.json',{body:JSON.stringify({claimed,restored:await read()}),contentType:'application/json'})
})
test('REPAIR-919 production repairs damaged suit with attached module and claims same identity',async({page},info)=>{
 const seed=JSON.parse(fixture()),suit=seed.inventory.equipment.instances[0]
 suit.remaining='1999/2';suit.location={kind:'inventory'}
 await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw:JSON.stringify(seed)})
 await boot(page,'','http://127.0.0.1:4175')
 await page.locator('[data-entity="scout:gear-bench"]').click({button:'right'});await page.getByRole('menuitem',{name:/Use Gear Crafting Bench/}).click()
 const dialog=page.getByRole('dialog',{name:'Gear Crafting Bench',exact:true})
 await expect.poll(async()=>{await page.clock.runFor(32);return dialog.isVisible()}).toBe(true)
 await dialog.getByRole('textbox',{name:'Amount in grams'}).fill('1');await dialog.getByRole('button',{name:'Review deposit'}).click();await dialog.getByRole('button',{name:'Confirm deposit'}).click()
 await dialog.getByLabel('Spacesuit to repair').selectOption(suit.id);await dialog.getByLabel('HP to restore').fill('0.5')
 await dialog.getByRole('button',{name:'Start suit repair'}).click();await walkTicks(page,[],3)
 await expect(dialog.locator('[data-bench-tray]')).toContainText('Repaired spacesuit')
 await dialog.getByRole('button',{name:'Claim equipment'}).click()
 const read=()=>page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY),claimed=await read()
 expect(claimed.inventory.equipment.instances[0]).toEqual({...suit,remaining:'1000/1'})
 expect(claimed.inventory.equipment.instances[1]).toEqual(seed.inventory.equipment.instances[1])
 expect(claimed.bench.repairs[0].claimed).toBe(true)
 await dialog.evaluate(e=>{e.scrollTop=0});await dialog.screenshot({path:info.outputPath('suit-repaired.png')})
 await page.keyboard.press('Escape');await page.reload();await page.clock.runFor(200)
 expect((await read()).inventory.equipment).toEqual(claimed.inventory.equipment)
 await info.attach('repair-state.json',{body:JSON.stringify({seed,claimed,restored:await read()}),contentType:'application/json'})
})
