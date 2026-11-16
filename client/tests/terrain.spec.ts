import {test,expect} from '@playwright/test'
import {ResourceSession,RESOURCE_SAVE_KEY} from '../src/resourceSession.ts'
import {createTopDownState,PLAYER_ID} from '../src/topDownRuntime.ts'
import {surfaceScoutLocation} from '../src/scoutLocation.ts'
import {resolveSurface,siteEntities} from '../src/surfaceSite.ts'
import {surfaceRegion,localToSurface,surfaceToLocal} from '../src/surfaceCoordinates.ts'
import {geologicalDefinition} from '../src/planetDefinition.ts'
import {samplePlanetSurface} from '../src/planetSurface.ts'
import {probeGeology} from '../src/planetGeology.ts'
import {depositFromId} from '../src/planetDeposits.ts'
import {normal,scale} from '../src/math.ts'
import {point2} from '../src/topDownFrames.ts'
import {compareImages} from '../scripts/visual-comparison.ts'
import {boot,position,walkTicks} from './worldJourney'

// Synthetic saved outdoor location in the same valid chart as a safely landed
// Scout. No shipping debug API or mock sampler; browser runs the built game.
function terrainSave() {
  const {bodyId,version,digest,radiusM}=geologicalDefinition('caligo'),definition={bodyId,version,digest}
  const base={bodyId,radialM:scale(normal([1,1,1]),radiusM+5)}
  const initialRegion=surfaceRegion(base,definition)
  const address={...localToSurface(initialRegion,[19500,0]),yaw:0,definition}
  const site=resolveSurface(address),location=surfaceScoutLocation(address),region=surfaceRegion(address,definition)
  const start=surfaceToLocal(base,region),anchor={frame:'planet:fixture' as const,point:point2(...start)}
  let world=createTopDownState()
  world={...world,scoutLocation:location,frames:site.frames,camera:{...world.camera,anchor},entities:[...world.entities.filter(e=>e.layer!=='terrain').map(e=>e.id===PLAYER_ID?{...e,anchor}:e),...siteEntities(site)]}
  let raw:string|null=null
  const session=new ResourceSession({getItem:()=>raw,setItem:(_:string,value:string)=>{raw=value}},world,undefined,undefined,()=>Date.parse('2026-11-04T00:00:00Z'))
  expect(session.error).toBe('')
  raw=JSON.stringify(session.snapshot,(_,v)=>typeof v==='bigint'?String(v):v)
  return {raw:raw!,start,region,definition}
}

test('TERRAIN-848 built movement stops at sampled steep rock, renders shared truth and reloads',async({page},info)=>{
  test.setTimeout(240000)
  const fixture=terrainSave(),errors:string[]=[]
  const physicalPosition=async()=>{
    const p=await position(page)
    const chart=await page.locator('[data-entity="person:player"]').getAttribute('data-surface-chart')
    return [...surfaceToLocal(localToSurface(chart ? JSON.parse(chart) : fixture.region,[p[0]!,p[1]!]),fixture.region)]
  }
  page.on('pageerror',e=>errors.push(e.message))
  await page.addInitScript(({key,value})=>{if(!localStorage.getItem(key))localStorage.setItem(key,value)},{key:RESOURCE_SAVE_KEY,value:fixture.raw})
  await boot(page,'','http://127.0.0.1:4175')
  // P05 adds mineable minerals to this fixed geological fixture. Validate the
  // actual browser sources against their canonical IDs and initial gram mass.
  const sources=page.locator('[data-source-kind]')
  await expect(sources).toHaveCount(4)
  const sourceRecords=await sources.evaluateAll(elements=>elements.map(el=>({id:el.getAttribute('data-entity')!,kind:el.getAttribute('data-source-kind'),grams:Number(el.getAttribute('data-remaining-grams'))})))
  for(const source of sourceRecords) {
    const deposit=depositFromId(source.id)
    expect(source.kind).toBe('mineral')
    expect(deposit.entity.anchor.chart!.bodyId).toBe(fixture.definition.bodyId)
    expect(source.grams).toBe(deposit.initial.remainingGrams)
  }
  await expect(page.locator('[data-generated-ground]')).toHaveAttribute('data-generated-ground',fixture.definition.digest)
  await expect(page.locator('[data-terrain-footprint="steep"]')).toBeVisible()
  expect(await page.locator('[data-terrain-footprint="steep"]').getAttribute('d')).not.toBe('')
  const capture=async(name:string)=>{
    const a=await page.screenshot({path:info.outputPath(`${name}.png`)}),b=await page.screenshot({path:info.outputPath(`${name}-repeat.png`)})
    const diff=compareImages(a,b);expect(diff.pass).toBe(true)
    await info.attach(`${name}-repeat.json`,{body:JSON.stringify({...diff,diff:undefined}),contentType:'application/json'})
    await info.attach(`${name}-diff.png`,{body:diff.diff,contentType:'image/png'})
  }
  await capture('generated-before')
  await walkTicks(page,'d',480)
  const stopped=await physicalPosition()
  expect(stopped[0]-fixture.start[0]).toBeGreaterThan(10)
  expect(stopped[0]-fixture.start[0]).toBeLessThan(17)
  expect(probeGeology(localToSurface(fixture.region,[stopped[0],stopped[1]]),fixture.definition,.46).collision).toBe(false)
  expect(probeGeology(localToSurface(fixture.region,[stopped[0],stopped[1]]),fixture.definition,.46,[.04,0],undefined,fixture.region).moveFraction).toBeLessThan(1)
  await walkTicks(page,'d',60);expect(await physicalPosition()).toEqual(stopped)
  // Renderer samples agree with domain material/slope; image-only art cannot
  // invent collision. Conservative hatching includes the footprint margin.
  const shown=await page.locator('[data-ground-position]').first().evaluate(el=>({point:el.getAttribute('data-ground-position')!.split(',').map(Number),slope:Number(el.getAttribute('data-ground-slope')),type:el.getAttribute('data-ground-type')}))
  const groundChart=JSON.parse((await page.locator('[data-ground-chart]').getAttribute('data-ground-chart'))!)
  const stream=JSON.parse((await page.locator('[data-stream]').getAttribute('data-stream'))!)
  expect(stream.regions).toBeLessThanOrEqual(32);expect(stream.samples).toBeLessThanOrEqual(32*2048);expect(stream.frameGenerated).toBeLessThanOrEqual(2048)
  const sampled=samplePlanetSurface(localToSurface(groundChart,shown.point as [number,number]),fixture.definition)
  expect(shown.slope).toBe(sampled.slopeRadians);expect(shown.type).toBe(sampled.groundType)
  await page.clock.runFor(1600);await capture('generated-blocked')
  await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
  const saved=await page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
  expect(saved.scout.surface.definition).toEqual(fixture.definition)
  await page.reload();await page.clock.runFor(200)
  await expect.poll(()=>physicalPosition()).toEqual(stopped)
  await walkTicks(page,'a',40);expect((await physicalPosition())[0]).toBeLessThan(stopped[0]-1)
  expect(errors).toEqual([])
  await info.attach('terrain-state.json',{body:JSON.stringify({start:fixture.start,stopped,shown,groundChart,stream,definition:fixture.definition,viewport:{width:1280,height:800},dpr:1,clock:'2026-11-04T00:00:00Z',input:'Playwright keyboard',backend:'Chromium SVG/Canvas CPU; GPU disabled',saved}),contentType:'application/json'})
})
