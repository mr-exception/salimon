import {writeFileSync} from 'node:fs'
import {test,expect} from '@playwright/test'
import {RESOURCE_SAVE_KEY} from '../src/resourceSession.ts'
import {compareImages} from '../scripts/visual-comparison.ts'
import {boot,walkTicks,position} from './worldJourney'
import {ART_FIXTURES,terrainArtSave} from './terrainArtFixtures'

for(const fixture of ART_FIXTURES)test(`ART-850 ${fixture.name} production illustration, repeat and reload`,async({page},info)=>{
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message))
  const raw=terrainArtSave(fixture.bodyId,fixture.direction,fixture.offset)
  await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw})
  await boot(page,'','http://127.0.0.1:4175')
  await expect(page.locator('[data-art-samples]')).toHaveAttribute('data-art-samples','1681')
  await expect(page.locator('[data-source-kind="biological"]')).toHaveCount(0)
  const ground=page.locator('[data-generated-ground]'),url=await ground.locator('image').getAttribute('href')
  expect(url!.length).toBeGreaterThan(1000)
  const original=await page.screenshot({path:info.outputPath('original.png')})
  const repeat=await page.screenshot({path:info.outputPath('repeat.png')})
  const comparison=compareImages(original,repeat)
  expect(comparison.pass).toBe(true)
  // Actual wrong-material mutation must fail the same pixel comparison policy.
  await ground.evaluate(el=>el.setAttribute('style','filter:brightness(.5)'))
  const missing=await page.screenshot({path:info.outputPath('negative-wrong-material.png')})
  const negative=compareImages(original,missing)
  expect(negative.pass).toBe(false)
  writeFileSync(info.outputPath('negative-diff.png'),negative.diff)
  await ground.evaluate(el=>el.removeAttribute('style'))
  await info.attach('pixels.json',{body:JSON.stringify({repeat:{...comparison,diff:undefined},negative:{...negative,diff:undefined},references:'new review candidates; no approved references replaced'}),contentType:'application/json'})
  const before=await position(page)
  await walkTicks(page,'a',60)
  expect(await page.locator('[data-entity="person:player"]').count()).toBe(1)
  await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
  const moved=await position(page)
  await page.reload();await page.clock.runFor(200)
  expect(await position(page)).toEqual(moved)
  await page.screenshot({path:info.outputPath('reload.png')})
  if(fixture.name==='barren') {
    expect(moved[0]).toBeLessThan(before[0]-1)
    for(const viewport of [{width:1920,height:600},{width:600,height:1200}]) {
      await page.setViewportSize(viewport);await page.clock.runFor(100)
      await page.screenshot({path:info.outputPath(`aspect-${viewport.width}.png`)})
      await expect(ground).toHaveAttribute('data-art-samples','1681')
    }
  }
  const stats=JSON.parse((await ground.getAttribute('data-stream'))!)
  expect(stats.frameGenerated).toBeLessThanOrEqual(2048)
  expect(stats.regions).toBeLessThanOrEqual(32)
  expect(errors).toEqual([])
  await info.attach('state.json',{body:JSON.stringify({fixture,stats,before,moved,viewport:info.project.use.viewport,dpr:1,clock:'2026-11-04T00:00:00Z',backend:'Chromium SVG/Canvas CPU',input:'Playwright keyboard'}),contentType:'application/json'})
})

test('ART-850 generated four-direction borders retain continuous albedo and physical state',async({page},info)=>{
  test.setTimeout(600000)
  const raw=terrainArtSave('caligo',[1,0,0],[1,1])
  await page.addInitScript(({key,raw})=>localStorage.setItem(key,raw),{key:RESOURCE_SAVE_KEY,raw})
  await boot(page,'','http://127.0.0.1:4175')
  const ground=page.locator('[data-generated-ground]'),before=await position(page),routes:unknown[]=[]
  const move=async(key:string)=>{
    const tick=async()=>Number(await page.locator('main').getAttribute('data-tick')),target=await tick()+4250
    await page.keyboard.down(key)
    while(target-await tick()>16)await page.clock.fastForward(200)
    await page.keyboard.up(key)
    const remainder=target-await tick();if(remainder>0)await walkTicks(page,key,remainder)
    expect(await tick()).toBe(target)
  }
  const firstUrl=await ground.locator('image').getAttribute('href')
  for(const [out,back] of [['a','d'],['d','a'],['w','s'],['s','w']]) {
    await move(out!)
    const far=await position(page)
    expect(Math.hypot(far[0]!-before[0]!,far[1]!-before[1]!)).toBeGreaterThan(169)
    await page.screenshot({path:info.outputPath(`border-${out}.png`)})
    expect(await ground.locator('image').getAttribute('href')).not.toBe(firstUrl)
    const stats=JSON.parse((await ground.getAttribute('data-stream'))!)
    expect(stats.regions).toBeLessThanOrEqual(32);expect(stats.frameGenerated).toBeLessThanOrEqual(2048)
    await move(back!)
    const returned=await position(page)
    expect(Math.hypot(returned[0]!-before[0]!,returned[1]!-before[1]!)).toBeLessThan(.001)
    // The exact same albedo bytes return after region eviction/backtracking.
    expect(await ground.locator('image').getAttribute('href')).toBe(firstUrl)
    routes.push({out,far,returned,stats})
  }
  await info.attach('border-state.json',{body:JSON.stringify({routes,input:'Playwright keyboard',viewport:info.project.use.viewport,dpr:1}),contentType:'application/json'})
})
