import {test,expect,type Page} from '@playwright/test'
import {ResourceSession,RESOURCE_SAVE_KEY} from '../src/resourceSession.ts'
import {createTopDownState,PLAYER_ID} from '../src/topDownRuntime.ts'
import {surfaceScoutLocation} from '../src/scoutLocation.ts'
import {resolveSurface,siteEntities} from '../src/surfaceSite.ts'
import {planetDefinition} from '../src/planetDefinition.ts'
import {localToSurface,surfaceRegion} from '../src/surfaceCoordinates.ts'
import {point2} from '../src/topDownFrames.ts'
import {compareImages} from '../scripts/visual-comparison.ts'
import {boot,position,walkTicks} from './worldJourney'

function fixture(x=-20) {
  // Original flat publication deliberately exercises unrestricted walking; P02's
  // separate built terrain scenario verifies slope blocking on the v2 sampler.
  const definition=planetDefinition('vesper-moon')
  const address={bodyId:definition.bodyId,radialM:[definition.radiusM+5,0,0] as const,yaw:0}
  const location=surfaceScoutLocation(address),site=resolveSurface(address),base=createTopDownState()
  const anchor={frame:'planet:fixture' as const,point:point2(x,-20)}
  const world={...base,scoutLocation:location,frames:site.frames,camera:{...base.camera,anchor},entities:[...base.entities.filter(e=>e.layer!=='terrain').map(e=>e.id===PLAYER_ID?{...e,anchor}:e),...siteEntities(site)]}
  const session=new ResourceSession({getItem:()=>null,setItem:()=>{}},world,undefined,undefined,()=>Date.parse('2026-11-04T00:00:00Z'))
  expect(session.error).toBe('')
  return JSON.stringify(session.snapshot,(_,v)=>typeof v==='bigint'?String(v):v)
}

async function moveAtBatchedCadence(page: Page, key: string, ticks: number) {
    const tick=async()=>Number(await page.locator('main').getAttribute('data-tick'))
    const target=await tick()+ticks
    await page.keyboard.down(key)
    // Exercise normal bounded catch-up at 5 Hz rendering: each 200 ms frame
    // still executes all twelve production 60 Hz movement/collision steps.
    for (let batch = 0; batch < Math.ceil(ticks / 6) && target - await tick() > 16; batch++) await page.clock.fastForward(200)
    await page.keyboard.up(key)
    const remainder=target-await tick()
    if(remainder>0) await walkTicks(page,key,remainder)
    expect(await tick()).toBe(target)
  }

test('STREAM-849 built real-input four-direction region journeys, chart handoff, reload and physical Scout',async({page},info)=>{
  test.setTimeout(600000)
  const errors:string[]=[],routes:unknown[]=[]
  page.on('pageerror',e=>errors.push(e.message))
  await page.addInitScript(({key,value})=>{if(!localStorage.getItem(key))localStorage.setItem(key,value)},{key:RESOURCE_SAVE_KEY,value:fixture()})
  await boot(page,'','http://127.0.0.1:4175')
  const save=async()=>{
    await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
    return page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
  }
  const physical=(saved:Awaited<ReturnType<typeof save>>)=>localToSurface(saved.location.anchor.chart??surfaceRegion(saved.scout.surface),saved.location.anchor.point.offsetM.slice(0,2))
  const initial=await save(),start=physical(initial)
  const before=await page.screenshot({path:info.outputPath('scout-near.png')})
  for(const [out,back] of [['a','d'],['d','a'],['w','s'],['s','w']]) {
    await moveAtBatchedCadence(page,out!,4250)
    const far=await save(),distance=Math.hypot(...physical(far).radialM.map((n,i)=>n-start.radialM[i]!))
    expect(distance).toBeGreaterThan(169);expect(distance).toBeLessThan(171)
    expect(surfaceRegion(physical(far)).id).not.toBe(surfaceRegion(start).id)
    await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(0)
    await expect(page.locator('[data-entity="person:player"]')).toHaveCount(1)
    if(out==='a') {
      await page.clock.runFor(1600)
      const a=await page.screenshot({path:info.outputPath('region-far.png')}),b=await page.screenshot({path:info.outputPath('region-far-repeat.png')})
      expect(compareImages(a,b).pass).toBe(true);expect(compareImages(before,a).pass).toBe(false)
    }
    await moveAtBatchedCadence(page,back!,4250)
    const returned=await save()
    expect(Math.hypot(...physical(returned).radialM.map((n,i)=>n-start.radialM[i]!))).toBeLessThan(.001)
    await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
    routes.push({out,far:far.location,returned:returned.location,distance})
  }
  // A further uninterrupted walk exceeds the original chart's small working
  // radius; reload restores its physical address rather than landing-site x/y.
  await moveAtBatchedCadence(page,'a',6500)
  const rebased=await save();expect(rebased.location.anchor.chart).toBeTruthy()
  await page.reload();await page.clock.runFor(200)
  const replay=await save();expect(physical(replay)).toEqual(physical(rebased))
  expect(replay.sources).toEqual(initial.sources)
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(0)
  await moveAtBatchedCadence(page,'d',6500)
  await expect(page.locator('[data-entity="ship:scout"]')).toHaveCount(1)
  expect(Math.hypot(...physical(await save()).radialM.map((n,i)=>n-start.radialM[i]!))).toBeLessThan(.001)
  expect(errors).toEqual([])
  await info.attach('streaming-state.json',{body:JSON.stringify({routes,rebased,replay,viewport:info.project.use.viewport,dpr:1,input:'Playwright keyboard',renderer:'Chromium SVG CPU',position:await position(page)}),contentType:'application/json'})
})

test('built artwork stays bounded across repeated physical region transitions', async ({ page }, info) => {
  await page.addInitScript(({ key, value }) => {
    if (!localStorage.getItem(key)) localStorage.setItem(key, value)
  }, { key: RESOURCE_SAVE_KEY, value: fixture(-31.5) })
  await boot(page, '', 'http://127.0.0.1:4175')
  const world = page.locator('.top-down-scene')
  const art = async () => {
    await expect(world).toHaveAttribute('data-render-ready', 'true')
    return JSON.parse((await world.getAttribute('data-art-resources'))!)
  }
  const address = JSON.parse(fixture(-31.5)).scout.surface
  const chart = surfaceRegion(address)
  const physical = async () => {
    const [x, y] = await position(page)
    return localToSurface(chart, [x!, y!])
  }
  const start = await physical()
  const records = []
  const capture = (name: string) => world.screenshot({
    path: info.outputPath(name),
    style: '.survival-hud,.resource-hud,.top-down-bottom,.exploration-hud { visibility: hidden }',
  })
  const captures: Buffer[] = []
  const initial = await art()
  for (let round = 0; round < 2; round++) {
    await moveAtBatchedCadence(page, 'a', 48)
    const far = await physical()
    expect(surfaceRegion(far).id).not.toBe(surfaceRegion(start).id)
    const away = await art()
    expect(away.decodedBytes).toBeLessThan(128 * 1024 * 1024)
    await moveAtBatchedCadence(page, 'd', 48)
    for (let settle = 0; settle < 7; settle++) await page.clock.fastForward(200)
    expect(surfaceRegion(await physical()).id).toBe(surfaceRegion(start).id)
    expect(await art()).toEqual(initial)
    records.push({ round, far, away, returned: await physical() })
    captures.push(await capture(`region-return-${round}.png`))
  }
  const comparison = compareImages(captures[0]!, captures[1]!)
  await info.attach('region-diff.png', { body: comparison.diff, contentType: 'image/png' })
  expect(comparison.pass).toBe(true)
  await info.attach('region-art.json', { body: JSON.stringify({ initial, records, changedPixels: comparison.changedPixels }), contentType: 'application/json' })
})
