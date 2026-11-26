import { test, expect } from '@playwright/test'
import { boot, walkTo, walkTicks, scene } from './worldJourney.ts'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
const origin = 'http://127.0.0.1:4275'
const suit = 'gear:0:starter-suit', module = 'gear:0:starter-temperature'

test('GEAR-915 built production adapter rechecks actual boarding/reach and atomically stores, installs, reloads and fails writes', async ({ page }, info) => {
 const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
 await boot(page, 'tests/equipment-transfers.html', origin)
 const result = page.getByLabel('Equipment fixture result')
 const click = async (name: string) => { await page.getByRole('button',{name,exact:true}).click(); await page.clock.runFor(32); await page.getByRole('button',{name,exact:true}).evaluate(e => (e as HTMLElement).blur()) }
 const read = () => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
 await click('Store suit fixture'); await expect(result).toContainText('inside Scout')
 for (const p of [[-2,3],[-2,7]]) await walkTo(page,p,'planet:fixture',true)
 for (const p of [[0,7],[.5,4.25]]) await walkTo(page,p,'ship:scout',true)
 await click('Store suit fixture'); await expect(result).toHaveText('Committed')
 let state = await read(); expect(state.inventory.equipment.instances.find((g: any) => g.id === suit).location).toEqual({kind:'drawer'})
 await click('Store module fixture'); await expect(result).toHaveText('Committed')
 state = await read(); expect(state.inventory.equipment.instances.find((g: any) => g.id === module).location).toEqual({kind:'module-storage'})
 await page.reload()
 await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
 expect((await read()).inventory.equipment).toEqual(state.inventory.equipment)
 await click('Install module fixture'); await expect(result).toHaveText('Committed')
 await click('Equip suit fixture'); await expect(result).toHaveText('Committed')
 const before = await read()
 await click('Stale suit fixture'); await expect(result).toContainText('Stale equipment revision')
 expect((await read()).inventory.equipment).toEqual(before.inventory.equipment)
 // Actually cross the threshold before issuing another command.
 for (const p of [[0,4.5],[0,7]]) await walkTo(page,p,'ship:scout',true)
 await click('Store suit fixture'); await expect(result).toContainText('inside Scout')
 expect((await read()).inventory.equipment).toEqual(before.inventory.equipment)
 await walkTo(page,[0,4.5],'ship:scout',true); await walkTo(page,[.5,4.25],'ship:scout',true)
 await walkTicks(page,[],60)
 const bytes = await page.evaluate(key => localStorage.getItem(key),RESOURCE_SAVE_KEY)
 await click('Fail equipment saves fixture'); await click('Store suit fixture'); await expect(result).toContainText('Saving failed')
 expect(await page.evaluate(key => localStorage.getItem(key),RESOURCE_SAVE_KEY)).toBe(bytes)
 await page.screenshot({path:info.outputPath('equipment-rollback.png'),fullPage:true})
 expect(errors).toEqual([])
 await info.attach('equipment-transactions.json',{body:JSON.stringify({stored:state,beforeFailedWrite:before,errors},null,2),contentType:'application/json'})
})
test('normal shipping build excludes equipment fixture controls and code', async ({ page }) => {
 await page.goto('http://127.0.0.1:4175/?equipmentFixture=1')
 await expect(page.getByRole('navigation',{name:'Non-shipping equipment adapter'})).toHaveCount(0)
 const response = await page.request.get('http://127.0.0.1:4175/tests/equipment-transfers.html')
 expect(await response.text()).not.toContain('equipment-transfers.tsx')
})
