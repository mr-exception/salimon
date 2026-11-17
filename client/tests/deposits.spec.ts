import { test, expect } from '@playwright/test'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { compareImages } from '../scripts/visual-comparison.ts'
import { boot, walkTicks, position } from './worldJourney'
import { depositFixture } from './depositFixtures'

for (const [name,index] of [['common',0],['dense',1],['rare',11],['barren',15]] as const) {
  test(`DEPOSIT-851 ${name} built cues, mining, conservation and reload`, async ({page},info) => {
    const {raw,deposit} = depositFixture(index), errors: string[] = []
    page.on('pageerror',e => errors.push(e.message))
    await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw})
    await boot(page,'','http://127.0.0.1:4175')
    const original = await page.screenshot({path:info.outputPath('source-original.png')})
    const repeat = await page.screenshot({path:info.outputPath('source-repeat.png')})
    expect(compareImages(original,repeat).pass).toBe(true)
    await expect(page.locator('[data-source-kind="biological"]')).toHaveCount(0)
    if (!deposit) {
      // Barren cell may neighbor productive cells; its own identity stays empty.
      await expect(page.locator(`[data-source-site="site:surface:v1:vesper-moon:0:1:0:${index}"]`)).toHaveCount(0)
    } else {
      const id = deposit.initial.id, source = page.locator(`[data-entity="${id}"]`)
      await expect(source).toHaveAttribute('data-remaining-grams',String(deposit.initial.remainingGrams))
      const sourceOriginal = await source.screenshot({path:info.outputPath('source-detail.png')})
      await page.getByRole('button',{name:'Equip gathering tool'}).click()
      await page.keyboard.press('e')
      await expect(page.getByLabel('Resource inventory')).toHaveAttribute('data-gathering',id)
      await walkTicks(page,[],59);await page.keyboard.press('Escape')
      let save = await page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
      expect(save.inventory.quantities.mineral).toBeUndefined()
      expect(save.miningWork[id]).toBe(944)
      await page.reload();await page.clock.runFor(200)
      await page.getByRole('button',{name:'Equip gathering tool'}).click()
      await page.keyboard.press('e');await walkTicks(page,[],1)
      await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
      await walkTicks(page,[],index===0?120:60)
      await page.keyboard.press('Escape')
      save = await page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
      const credited = index===0 ? deposit.initial.remainingGrams : 2000
      expect(save.inventory.quantities.mineral).toBe(credited)
      expect(save.sources[id].remainingGrams + credited).toBe(deposit.initial.remainingGrams)
      if(index===0) {
        await expect(source).toHaveAttribute('data-depleted','true')
        const detail = await source.screenshot({path:info.outputPath('source-depleted-detail.png')})
        expect(compareImages(sourceOriginal,detail).pass).toBe(false)
      }
      // A missing mineable silhouette must fail the same scene comparison.
      await source.evaluate(el=>el.setAttribute('visibility','hidden'))
      const missing = await page.screenshot({path:info.outputPath('negative-missing-source.png')})
      await source.evaluate(el=>el.removeAttribute('visibility'))
      expect(compareImages(await page.screenshot(),missing).pass).toBe(false)
      const mined = await page.screenshot({path:info.outputPath('source-mined.png')})
      expect(compareImages(original,mined).pass).toBe(false)
      await page.reload();await page.clock.runFor(200)
      await expect(source).toHaveAttribute('data-remaining-grams',String(deposit.initial.remainingGrams-credited))
      await info.attach('source-state.json',{body:JSON.stringify({save,deposit,position:await position(page),browser:info.project.use,renderer:'Chromium SVG/Canvas CPU',input:'Playwright keyboard',baseline:'new review candidates; existing references unchanged'},(_,v)=>typeof v==='bigint'?String(v):v),contentType:'application/json'})
    }
    expect(errors).toEqual([])
  })
}

test('DEPOSIT-851 real walking evicts worked source and returns without replenishment', async ({page},info) => {
  test.setTimeout(600000)
  const {raw,deposit}=depositFixture(0),id=deposit!.initial.id
  await page.addInitScript(({key,raw})=>{if(!localStorage.getItem(key))localStorage.setItem(key,raw)},{key:RESOURCE_SAVE_KEY,raw})
  await boot(page,'','http://127.0.0.1:4175')
  await page.keyboard.press('m')
  const notebook=page.getByRole('dialog',{name:'Field notebook'})
  await notebook.getByLabel('Name (optional)',{exact:true}).fill('Mined return site')
  await notebook.getByLabel('Short note (optional)',{exact:true}).fill('One kilogram removed')
  await notebook.getByRole('button',{name:'Mark current location'}).click()
  await page.keyboard.press('Escape')
  const marks=await page.evaluate(key=>JSON.parse(localStorage.getItem(key)!).journal.marks,RESOURCE_SAVE_KEY)
  expect(marks).toHaveLength(1)
  await page.locator('main').click({position:{x:640,y:350}})
  await page.getByRole('button',{name:'Equip gathering tool'}).click()
  await page.keyboard.press('e');await walkTicks(page,[],60);await page.keyboard.press('Escape')
  const mark=marks[0].id
  const before=await position(page)
  const move=async(key:string)=>{
    const tick=async()=>Number(await page.locator('main').getAttribute('data-tick')),target=await tick()+6000
    await page.keyboard.down(key)
    while(target-await tick()>16)await page.clock.fastForward(200)
    await page.keyboard.up(key)
    const rest=target-await tick();if(rest>0)await walkTicks(page,key,rest)
    expect(await tick()).toBe(target)
  }
  await move('w')
  await expect(page.locator(`[data-entity="${id}"]`)).toHaveCount(0)
  await page.screenshot({path:info.outputPath('evicted-region.png')})
  await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
  const farSave=await page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
  expect(farSave.journal.visits.length).toBeGreaterThan(2)
  expect(farSave.journal.visits.length).toBeLessThanOrEqual(128)
  expect(farSave.journal.marks).toEqual(marks)
  await page.reload();await page.clock.runFor(200)
  await page.keyboard.press('m')
  await expect(notebook.getByLabel('Known places')).toContainText('Mined return site')
  await page.keyboard.press('Escape')
  await page.locator('main').click({position:{x:640,y:350}})
  await move('s')
  const returned=await position(page)
  expect(Math.hypot(returned[0]!-before[0]!,returned[1]!-before[1]!)).toBeLessThan(.001)
  await expect(page.locator(`[data-entity="${id}"]`)).toHaveAttribute('data-remaining-grams',String(deposit!.initial.remainingGrams-1000))
  await expect(page.locator('[data-field-mark]')).toHaveCount(1)
  await expect(page.locator('[data-field-mark]')).toHaveAttribute('data-field-mark',mark!)
  await page.evaluate(()=>window.dispatchEvent(new Event('pagehide')))
  const returnedSave=await page.evaluate(key=>JSON.parse(localStorage.getItem(key)!),RESOURCE_SAVE_KEY)
  expect(returnedSave.inventory.quantities.mineral+returnedSave.sources[id].remainingGrams).toBe(deposit!.initial.remainingGrams)
  expect(returnedSave.journal.marks).toEqual(farSave.journal.marks)
  expect(returnedSave.scout.ship.position).toEqual(farSave.scout.ship.position)
  await page.screenshot({path:info.outputPath('revisited-region.png')})
  await info.attach('eviction-state.json',{body:JSON.stringify({id,before,returned,farSave,returnedSave,mark,input:'Playwright keyboard; 240 m out and back through region eviction and reload'}),contentType:'application/json'})
})
