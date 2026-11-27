import {test,expect,type Page} from '@playwright/test'
import { GEAR_DEFINITIONS, type GearInstance } from '../src/equipment.ts'
import {ResourceSession,RESOURCE_SAVE_KEY} from '../src/resourceSession.ts'
import {createTopDownState,PLAYER_ID} from '../src/topDownRuntime.ts'
import {point2} from '../src/topDownFrames.ts'
import {boot,walkTicks,scene} from './worldJourney.ts'
import {compareImages} from '../scripts/visual-comparison.ts'
function fixture(field=false){
 let raw:string|null=null;const initial=createTopDownState('exterior',true)
 const world={...initial,entities:initial.entities.map(e=>e.id===PLAYER_ID?{...e,anchor:{frame:field?'planet:fixture' as const:'ship:scout' as const,point:field?point2(129,0):point2(.7,4.1)}}:e)}
 const s=new ResourceSession({getItem:()=>raw,setItem:(_,v)=>{raw=v}},world,undefined,undefined,()=>Date.parse('2026-11-04T00:00:01Z'))
 s.upgrade(10000,12);expect(s.error).toBe('')
 return raw!
}
test.use({actionTimeout:10_000})
const pageErrors=new WeakMap<Page,string[]>()
test.beforeEach(({page})=>{const errors:string[]=[];pageErrors.set(page,errors);page.on('pageerror',e=>errors.push(e.message))})
test.afterEach(({page})=>expect(pageErrors.get(page)).toEqual([]))
const read=(page:import('@playwright/test').Page)=>page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
for(const [width,height] of [[960,600],[1280,720],[1920,1080]])test(`GEAR-920 production equipment transfers, confirmation, focus and responsive ${width}`,async({page},info)=>{
 await page.setViewportSize({width,height});const raw=fixture()
 await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw})
 await boot(page,'','http://127.0.0.1:4175')
 const cabinet=(await page.locator('[data-entity="scout:storage"]').boundingBox())!;await page.mouse.click(cabinet.x+cabinet.width/2,cabinet.y+cabinet.height/2,{button:'right'});await page.getByRole('menuitem',{name:/Manage equipment and storage/}).click()
 const dialog=page.getByRole('dialog',{name:'Equipment and storage',exact:true})
 await expect.poll(async()=>{await page.clock.runFor(32);return dialog.isVisible()}).toBe(true)
 await expect(dialog).toBeFocused()
 await dialog.getByRole('button',{name:'Store suit',exact:true}).click()
 await expect(dialog.getByRole('button',{name:'Confirm equipment change'})).toBeFocused()
 await dialog.getByRole('button',{name:'Confirm equipment change'}).click()
 let saved=await read(page);expect(saved.inventory.equipment.instances[0].location.kind).toBe('drawer')
 await expect(dialog.getByLabel('Equipped suit')).toContainText('No spacesuit')
 await dialog.getByRole('button',{name:'Equip suit',exact:true}).click();await dialog.getByRole('button',{name:'Confirm equipment change'}).click()
 await dialog.getByRole('button',{name:'Store module',exact:true}).click();await dialog.getByRole('button',{name:'Confirm equipment change'}).click()
 saved=await read(page);expect(saved.inventory.equipment.instances[1].location.kind).toBe('module-storage')
 await dialog.getByRole('button',{name:'Install in slot 2',exact:true}).click();await dialog.getByRole('button',{name:'Confirm equipment change'}).click()
 saved=await read(page);expect(saved.inventory.equipment.instances[1].location.slot).toBe(1)
 const box=(await dialog.boundingBox())!;expect(box.x).toBeGreaterThanOrEqual(0);expect(box.y).toBeGreaterThanOrEqual(0);expect(box.x+box.width).toBeLessThanOrEqual(width);expect(box.y+box.height).toBeLessThanOrEqual(height)
 expect(await dialog.evaluate(e=>e.scrollWidth>e.clientWidth)).toBe(false)
 expect(await dialog.evaluate(e=>getComputedStyle(e).animationDuration)).toBe('0s')
 await dialog.evaluate(e=>{e.scrollTop=0})
 const original=await dialog.screenshot({path:info.outputPath('equipment.png')}),repeat=await dialog.screenshot({path:info.outputPath('equipment-repeat.png')});expect(compareImages(original,repeat).pass).toBe(true)
 const defect=await page.addStyleTag({content:'.equipment-panel .ui-dialog { background: #ff00ff !important; }'})
 const negative=await dialog.screenshot({path:info.outputPath('equipment-negative.png')});expect(compareImages(original,negative).pass).toBe(false);await defect.evaluate(e=>e.parentNode?.removeChild(e))
 await page.keyboard.press('Shift+Tab');await page.keyboard.press('Tab');expect(await dialog.evaluate(e=>e.contains(document.activeElement))).toBe(true)
 await page.keyboard.press('Escape');await expect(dialog).toHaveCount(0)
 await page.reload();await expect.poll(async()=>{await page.clock.runFor(32);return scene(page).getAttribute('data-render-ready')}).toBe('true')
 expect((await read(page)).inventory.equipment).toEqual(saved.inventory.equipment)
 await info.attach('equipment-state.json',{body:JSON.stringify({saved,restored:await read(page)}),contentType:'application/json'})
})
test('GEAR-920 field disabled actions, live thresholds and destruction are committed and clear',async({page},info)=>{
 const seed=JSON.parse(fixture(true));seed.inventory.equipment.instances[1].remaining='1/1000'
 await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw:JSON.stringify(seed)})
 await boot(page,'','http://127.0.0.1:4175')
 await expect(page.getByLabel('Equipment and environmental condition')).toContainText('Local temperature')
 await expect(page.getByLabel('Equipment destruction')).toContainText('destroyed')
 await page.getByRole('button',{name:'Field inventory I'}).click();await page.getByRole('button',{name:'Equipment and storage',exact:true}).click()
 const dialog=page.getByRole('dialog',{name:'Equipment and storage',exact:true})
 await expect(dialog.getByRole('button',{name:'Store suit',exact:true})).toHaveAttribute('aria-disabled','true')
 await expect(dialog).toContainText('standing inside Scout')
 await page.keyboard.press('Escape');await walkTicks(page,[],5)
 expect((await read(page)).inventory.equipment.instances[1].location.kind).toBe('destroyed')
 await page.screenshot({path:info.outputPath('hazard-destruction.png')})
 await walkTicks(page,[],301);await expect(page.getByLabel('Equipment destruction')).toBeEmpty()
})

test('GEAR-920 full carried capacity explains rejection without changing equipment',async({page})=>{
 const seed=JSON.parse(fixture());seed.inventory.quantities.mineral=10000;seed.sources['source:landing:mineral:01'].remainingGrams-=10000
 await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw:JSON.stringify(seed)})
 await boot(page,'','http://127.0.0.1:4175')
 await page.getByRole('button',{name:'Field inventory I'}).click();await page.getByRole('button',{name:'Equipment and storage',exact:true}).click()
 const dialog=page.getByRole('dialog',{name:'Equipment and storage',exact:true}),before=await read(page)
 const carry=dialog.getByRole('button',{name:'Carry item',exact:true}).first()
 await expect(carry).toHaveAttribute('aria-disabled','true');await carry.focus();await carry.press('Enter')
 expect((await read(page)).inventory.equipment).toEqual(before.inventory.equipment)
 await expect(dialog.getByLabel('Confirm equipment change')).toHaveCount(0)
})

test('GEAR-920 production threshold warning stays stable and clears on removal',async({page})=>{
 const seed=JSON.parse(fixture(true));seed.inventory.equipment.instances[1].remaining='4001/20'
 await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw:JSON.stringify(seed)})
 await boot(page,'','http://127.0.0.1:4175');await walkTicks(page,[],5)
 const warning=page.getByLabel('Equipment warnings');await expect(warning).toContainText('20% or less')
 const text=await warning.innerText();await walkTicks(page,[],20);expect(await warning.innerText()).toBe(text)
 await page.getByRole('button',{name:'Pause [P]',exact:true}).click()
 let raw=await read(page);raw.inventory.equipment.instances[1].remaining='1001/20'
 await page.evaluate(({key,raw})=>localStorage.setItem(key,JSON.stringify(raw)),{key:RESOURCE_SAVE_KEY,raw});await page.reload()
 await expect.poll(async()=>{await page.clock.runFor(32);return scene(page).getAttribute('data-render-ready')}).toBe('true')
 await walkTicks(page,[],5);await expect(page.getByLabel('Equipment warnings')).toContainText('5% or less')
 raw=await read(page);raw.inventory.equipment.instances[1].location={kind:'inventory'}
 await page.evaluate(({key,raw})=>localStorage.setItem(key,JSON.stringify(raw)),{key:RESOURCE_SAVE_KEY,raw});await page.reload()
 await expect.poll(async()=>{await page.clock.runFor(32);return scene(page).getAttribute('data-render-ready')}).toBe('true')
 await expect(page.getByLabel('Equipment warnings')).toBeEmpty()
})

test('every delivered protection and gathering icon exports a transparent 256px image', async ({ page }, info) => {
 const seed = JSON.parse(fixture())
 const inventory = seed.inventory.equipment.instances
 for (const definitionId of Object.keys(GEAR_DEFINITIONS)) {
  if (inventory.some((item: GearInstance) => item.definitionId === definitionId)) continue
  inventory.push({ id: `gear:0:icon-${definitionId.replaceAll(':', '-')}`, definitionId,
   remaining: definitionId === 'module:poison:t2' ? '400/1' : '1000/1', location: { kind: 'module-storage' } })
 }
 await page.addInitScript(({ key, raw }) => {
  if (!localStorage.getItem(key)) localStorage.setItem(key, raw)
 }, { key: RESOURCE_SAVE_KEY, raw: JSON.stringify(seed) })
 await boot(page, '', 'http://127.0.0.1:4175')
 await page.getByRole('button', { name: /Field inventory/ }).click()
 await page.getByRole('button', { name: 'Equipment and storage' }).click()
 const dialog = page.getByRole('dialog', { name: 'Equipment and storage' })
 await expect(dialog).toBeVisible()
 await expect(dialog.locator('[data-equipment-icon]')).toHaveCount(Object.keys(GEAR_DEFINITIONS).length)
 await expect(dialog.locator('[data-equipment-icon="module:poison:t2"]')).toHaveAttribute('data-icon-condition', 'worn')
 const assets = await page.locator('[data-equipment-icon]').evaluateAll(async icons => {
  return Promise.all(icons.map(async icon => {
   const source = new XMLSerializer().serializeToString(icon)
   const image = new Image()
   image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(source)}`
   await image.decode()
   const canvas = document.createElement('canvas')
   canvas.width = image.naturalWidth
   canvas.height = image.naturalHeight
   const context = canvas.getContext('2d')!
   context.drawImage(image, 0, 0)
   const { data } = context.getImageData(0, 0, canvas.width, canvas.height)
   let left = canvas.width, top = canvas.height, right = -1, bottom = -1
   for (let y = 0; y < canvas.height; y++) for (let x = 0; x < canvas.width; x++) {
    if (!data[(y * canvas.width + x) * 4 + 3]) continue
    left = Math.min(left, x); top = Math.min(top, y); right = Math.max(right, x); bottom = Math.max(bottom, y)
   }
   return { id: (icon as SVGSVGElement).dataset.equipmentIcon!, dimensions: [canvas.width, canvas.height],
    bounds: [left, top, right, bottom], png: canvas.toDataURL() }
  }))
 })
 expect(assets.map(asset => asset.id).sort()).toEqual([...Object.keys(GEAR_DEFINITIONS), 'gathering-tool'].sort())
 expect(new Set(assets.map(asset => asset.png)).size).toBe(assets.length)
 for (const asset of assets) {
  expect(asset.dimensions).toEqual([256, 256])
  const [left, top, right, bottom] = asset.bounds
  expect(Math.min(left, top, 255 - right, 255 - bottom)).toBeGreaterThan(0)
  expect(right - left).toBeGreaterThan(20)
  expect(bottom - top).toBeGreaterThan(20)
  await info.attach(asset.id.replaceAll(':', '-'), { body: Buffer.from(asset.png.split(',')[1]!, 'base64'), contentType: 'image/png' })
 }
 await dialog.getByRole('list', { name: 'Equipment inventory' }).evaluate(list => list.scrollIntoView({ block: 'start' }))
 const original = await dialog.screenshot({ path: info.outputPath('equipment-icons.png') })
 const repeat = await dialog.screenshot({ path: info.outputPath('equipment-icons-repeat.png') })
 expect(compareImages(original, repeat).pass).toBe(true)
})
