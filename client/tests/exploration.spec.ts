import { test, expect } from '@playwright/test'
import { RESOURCE_SAVE_KEY, ResourceSession } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { surfaceScoutLocation } from '../src/scoutLocation.ts'
import { resolveSurface, siteEntities } from '../src/surfaceSite.ts'
import { planetDefinition } from '../src/planetDefinition.ts'
import { point2 } from '../src/topDownFrames.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
import { boot, position, walkTicks } from './worldJourney'

function walkingSave() {
  const d=planetDefinition('vesper-moon'),address={bodyId:d.bodyId,radialM:[d.radiusM+5,0,0] as const,yaw:0}
  const scoutLocation=surfaceScoutLocation(address),site=resolveSurface(address),base=createTopDownState('exterior',true)
  const anchor={frame:'planet:fixture' as const,point:point2(-20,-20)}
  const session=new ResourceSession({getItem:()=>null,setItem:()=>{}},{...base,scoutLocation,frames:site.frames,camera:{...base.camera,anchor},entities:[...base.entities.filter(e=>e.layer!=='terrain').map(e=>e.id===PLAYER_ID?{...e,anchor}:e),...siteEntities(site)]})
  if(session.error)throw Error(session.error)
  return JSON.stringify(session.snapshot,(_,v)=>typeof v==='bigint'?String(v):v)
}

test('EXPLORE-852 deliberate discovery, notes, focus, projected physical cues and negative/repeat captures',async({page},info)=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message))
  await boot(page,'','http://127.0.0.1:4175')
  await expect(page.locator('[data-field-mark]')).toHaveCount(0)
  await page.keyboard.press('g')
  await expect(page.locator('[data-scout-guidance]')).toContainText('estimate')
  await page.getByRole('button',{name:'Help',exact:true}).click(); await page.getByRole('button',{name:'Inspect / mark',exact:true}).click()
  const dialog=page.getByRole('dialog',{name:'Field notebook'})
  await expect(dialog.getByLabel('Known places').locator('li')).toHaveCount(0)
  await expect(dialog.locator('[data-landmark]')).not.toHaveCount(0)
  await dialog.getByLabel('Name (optional)',{exact:true}).fill('West ridge note')
  await dialog.getByLabel('Short note (optional)',{exact:true}).fill('Walk around the west boulder')
  const initial=await position(page)
  await page.keyboard.press('w');await page.clock.runFor(100)
  expect(await position(page)).toEqual(initial)
  // Reset the typed movement character, then inspect one real physical source.
  await dialog.getByLabel('Short note (optional)',{exact:true}).fill('Walk around the west boulder')
  await dialog.getByRole('button',{name:'Inspect west boulder'}).click()
  await expect(dialog.getByLabel('Known places').locator('li')).toHaveCount(1)
  await expect(dialog.getByLabel('Known places')).toContainText('known physical source')
  const id=await dialog.locator('[data-known-mark]').getAttribute('data-known-mark')
  await page.keyboard.press('Escape')
  await expect(dialog).not.toBeVisible()
  await expect(page.getByRole('button',{name:'Help',exact:true})).toBeFocused()
  await expect(page.locator('[data-field-mark]')).toHaveAttribute('data-field-mark',id!)
  await page.clock.runFor(800)
  const original=await page.screenshot({path:info.outputPath('landmark-original.png')})
  expect(compareImages(original,await page.screenshot({path:info.outputPath('landmark-repeat.png')})).pass).toBe(true)
  const box=await page.locator('[data-field-mark]').boundingBox()
  const clip={x:box!.x-10,y:box!.y-10,width:box!.width+20,height:box!.height+20}
  const detail=await page.screenshot({clip,path:info.outputPath('mark-detail.png')})
  await page.locator('[data-field-mark]').evaluate(el=>el.setAttribute('visibility','hidden'))
  expect(compareImages(detail,await page.screenshot({clip,path:info.outputPath('negative-missing-mark.png')})).pass).toBe(false)
  await page.locator('[data-field-mark]').evaluate(el=>el.removeAttribute('visibility'))
  await page.locator('main').click({position:{x:640,y:350}}); await walkTicks(page,'d',30)
  expect(await position(page)).not.toEqual(initial)
  await page.locator('main').click({position:{x:640,y:350}}); await page.keyboard.press('m')
  await dialog.getByRole('button',{name:'Mark current location'}).click()
  await expect(dialog.getByLabel('Known places').locator('li')).toHaveCount(2)
  await page.screenshot({path:info.outputPath('notebook-original.png')})
  await info.attach('marks.json',{body:JSON.stringify({id,known:await dialog.getByLabel('Known places').innerText(),initial,position:await position(page),renderer:'Chromium SVG/Canvas CPU',viewport:info.project.use.viewport,dpr:1}),contentType:'application/json'})
  await page.keyboard.press('Escape')
  // Durable notes and the same physical landmark survive reload.
  await page.reload();await page.clock.runFor(200)
  await expect(page.locator('[data-field-mark]')).toHaveCount(2)
  await page.locator('main').click({position:{x:640,y:350}}); await page.keyboard.press('m')
  await expect(dialog.locator(`[data-landmark="landmark:authored:v1:vesper:rock:west"]`)).toHaveCount(1)
  expect(errors).toEqual([])
})

test('EXPLORE-852 bounded return walk crosses seams, preserves marks/resources and loses an unsupported fix on reload',async({page},info)=>{
  test.setTimeout(600000)
  await page.addInitScript(({key,value})=>{if(!localStorage.getItem(key))localStorage.setItem(key,value)},{key:RESOURCE_SAVE_KEY,value:walkingSave()})
  await boot(page,'','http://127.0.0.1:4175')
  const save=async()=>{await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')));return page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)}
  const initial=await save(),start=await position(page)
  // Starting outside visual reacquisition range has no invented fix.
  await page.keyboard.press('g')
  await expect(page.locator('[data-scout-guidance]')).toHaveAttribute('data-fix','false')
  const move=async(key:string,ticks:number)=>{
    const tick=async()=>Number(await page.locator('main').getAttribute('data-tick')),target=await tick()+ticks
    await page.keyboard.down(key)
    while(target-await tick()>16)await page.clock.fastForward(200)
    await page.keyboard.up(key)
    const rest=target-await tick();if(rest>0)await walkTicks(page,key,rest)
  }
  // Sight the real hull to obtain a local estimate, then walk 140 m out/back.
  await move('d',200)
  await expect(page.locator('[data-scout-guidance]')).toHaveAttribute('data-fix','true')
  await page.locator('main').click({position:{x:640,y:350}}); await page.keyboard.press('m')
  await page.getByRole('button',{name:'Mark current location',exact:true}).click()
  await page.keyboard.press('Escape')
  const markId=await page.locator('[data-field-mark]').getAttribute('data-field-mark')
  await page.locator('main').click({position:{x:640,y:350}})
  const near=await position(page)
  await move('w',3500)
  await expect(page.locator('[data-scout-guidance]')).toHaveAttribute('data-fix','true')
  await expect(page.locator('[data-scout-guidance]')).toContainText('behind')
  const far=await save()
  const farPosition=await position(page)
  expect(Math.hypot(farPosition[0]!-near[0]!,farPosition[1]!-near[1]!)).toBeCloseTo(140,2)
  await page.screenshot({path:info.outputPath('guidance-far.png')})
  await move('s',3500)
  await expect(page.locator('[data-field-mark]')).toHaveAttribute('data-field-mark',markId!)
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
  const returned=await save()
  const returnedPosition=await position(page)
  expect(Math.hypot(returnedPosition[0]!-near[0]!,returnedPosition[1]!-near[1]!)).toBeLessThan(.001)
  expect(returned.sources).toEqual(initial.sources);expect(returned.inventory).toEqual(initial.inventory)
  expect(returned.scout.surface).toEqual(initial.scout.surface);expect(returned.scout.ship.position).toEqual(initial.scout.ship.position);expect(returned.scout.ship.mode).toBe(initial.scout.ship.mode)
  await move('w',3500);await save();await page.reload();await page.clock.runFor(200)
  await page.keyboard.press('g')
  await expect(page.locator('[data-scout-guidance]')).toHaveAttribute('data-fix','false')
  await page.screenshot({path:info.outputPath('guidance-unavailable.png')})
  await info.attach('return-route.json',{body:JSON.stringify({start,near,farPosition,returnedPosition,far:far.location,returned:returned.location,markId,ship:returned.scout,input:'real keyboard, 140 m out/back across several 64 m regions',viewport:info.project.use.viewport,dpr:1,renderer:'Chromium SVG CPU'}),contentType:'application/json'})
})

test('EXPLORE-852 generated ridge inspection, physical alignment and readable guidance at viewport extremes',async({page},info)=>{
  const {terrainArtSave}=await import('./terrainArtFixtures')
  const {geologicalDefinition}=await import('../src/planetDefinition.ts')
  const {localToSurface,surfaceRegion,surfaceToLocal}=await import('../src/surfaceCoordinates.ts')
  const {normal,scale}=await import('../src/math.ts')
  const {regionLandmarks}=await import('../src/exploration.ts')
  const {probeGeology}=await import('../src/planetGeology.ts')
  const d=geologicalDefinition('caligo'),origin=surfaceRegion({bodyId:d.bodyId,radialM:scale(normal([1,1,1]),d.radiusM+5)},d)
  const region=surfaceRegion(localToSurface(origin,[448,1000]),d),landmark=regionLandmarks(region)[0]!
  const physical=localToSurface(region,[10,0])
  expect(probeGeology(physical,d,.46).collision).toBe(false)
  const raw=terrainArtSave('caligo',[1,1,1],surfaceToLocal(physical,origin))
  await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw})
  await boot(page,'','http://127.0.0.1:4175')
  await expect(page.locator('[data-field-mark]')).toHaveCount(0)
  await page.keyboard.press('m')
  const dialog=page.getByRole('dialog',{name:'Field notebook'})
  await expect(dialog.getByLabel('Known places').locator('li')).toHaveCount(0)
  await dialog.getByRole('button',{name:'Inspect Fractured ridge',exact:true}).click()
  await expect(dialog.locator('[data-known-mark]')).toHaveAttribute('data-known-mark',`mark:v1:${landmark.id}`)
  await page.keyboard.press('Escape')
  await expect(page.locator('[data-field-mark]')).toHaveAttribute('data-field-mark',`mark:v1:${landmark.id}`)
  await page.locator('main').click({position:{x:640,y:350}});await page.keyboard.press('g')
  await expect(page.locator('[data-scout-guidance]')).toHaveAttribute('data-fix','false')
  for(const viewport of [{width:960,height:600},{width:1920,height:800},{width:600,height:1200}]) {
    await page.setViewportSize(viewport);await page.clock.runFor(200)
    const a=await page.screenshot({path:info.outputPath(`ridge-${viewport.width}.png`)})
    expect(compareImages(a,await page.screenshot({path:info.outputPath(`ridge-${viewport.width}-repeat.png`)})).pass).toBe(true)
    const boxes=await page.locator('.survival-hud,.resource-hud,.top-down-bottom,.exploration-hud').evaluateAll(els=>els.map(el=>{const r=el.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height}}))
    for(const box of boxes){expect(box.x).toBeGreaterThanOrEqual(0);expect(box.y).toBeGreaterThanOrEqual(0);expect(box.x+box.width).toBeLessThanOrEqual(viewport.width);expect(box.y+box.height).toBeLessThanOrEqual(viewport.height)}
    const panel=boxes.at(-1)!
    // The new overlay may not obscure inventory, survival or nearby actions.
    for(const other of boxes.slice(0,-1))expect(panel.x+panel.width<=other.x || other.x+other.width<=panel.x || panel.y+panel.height<=other.y || other.y+other.height<=panel.y).toBe(true)
  }
  await info.attach('physical-landmark.json',{body:JSON.stringify({landmark,physical,sample:probeGeology(landmark.location,d),renderer:'Chromium SVG/Canvas CPU',seed:d.domains.geology.seed}),contentType:'application/json'})
})

test('PERSIST-853 notebook write failure keeps durable note and shows recovery inside the modal',async({page},info)=>{
  await boot(page,'','http://127.0.0.1:4175')
  await page.keyboard.press('m')
  const dialog=page.getByRole('dialog',{name:'Field notebook'})
  await dialog.getByLabel('Name (optional)',{exact:true}).fill('Retained note')
  await dialog.getByRole('button',{name:'Mark current location'}).click()
  await expect(dialog.getByLabel('Known places')).toContainText('Retained note')
  const before=await page.evaluate(key=>localStorage.getItem(key),RESOURCE_SAVE_KEY)
  await page.evaluate(key=>{
    const original=Storage.prototype.setItem
    Storage.prototype.setItem=function(name,value){
      if(name===key)throw new DOMException('Storage quota exceeded','QuotaExceededError')
      return original.call(this,name,value)
    }
  },RESOURCE_SAVE_KEY)
  await dialog.getByLabel('Name (optional)',{exact:true}).fill('Rejected replacement')
  await dialog.getByRole('button',{name:'Mark current location'}).click()
  await expect(dialog.getByRole('status').filter({hasText:'Saving failed'})).toContainText('reload after resolving storage access')
  await expect(dialog.getByLabel('Known places')).toContainText('Retained note')
  await expect(dialog.getByLabel('Known places')).not.toContainText('Rejected replacement')
  await expect(dialog.getByRole('button',{name:'Mark current location'})).toBeDisabled()
  expect(await page.evaluate(key=>localStorage.getItem(key),RESOURCE_SAVE_KEY)).toBe(before)
  await page.screenshot({path:info.outputPath('notebook-quota-error.png')})
  await info.attach('retained-save.json',{body:before!,contentType:'application/json'})
})
