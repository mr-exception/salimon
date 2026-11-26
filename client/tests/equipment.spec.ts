import { test, expect } from '@playwright/test'
import { boot, walkTicks, scene } from './worldJourney.ts'
import { RESOURCE_SAVE_KEY } from '../src/resourceSession.ts'
import { freshEquipment } from '../src/equipment.ts'

test('GEAR-914 built starter survives actual gathering, migration and reload without regrant', async ({ page }, info) => {
 const errors: string[] = []; page.on('pageerror', e => errors.push(e.message))
 await boot(page, '', 'http://127.0.0.1:4175')
 await walkTicks(page, 'w', 75); await walkTicks(page, 'a', 25)
 await page.getByRole('button', { name: 'Equip gathering tool' }).click()
 await page.keyboard.press('e'); await walkTicks(page, [], 60); await page.keyboard.press('Escape')
 await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
 const read = () => page.evaluate(key => JSON.parse(localStorage.getItem(key)!), RESOURCE_SAVE_KEY)
 const first = await read()
 expect(first.version).toBe(19); expect(first.inventory.equipment).toEqual(freshEquipment())
 // A valid prior expedition's committed exact wear must not be refreshed by reload.
 first.inventory.equipment.instances[1].remaining = '1999/2'
 await page.evaluate(({ key, save }) => localStorage.setItem(key, JSON.stringify(save)), { key: RESOURCE_SAVE_KEY, save: first })
 await page.reload(); await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
 await page.evaluate(() => document.fonts.ready); await page.clock.runFor(100)
 await expect(page.locator('[data-resource="mineral"]')).toHaveText('1 kg')
 expect((await read()).inventory.equipment.instances[1].remaining).toBe('1999/2')
 await page.screenshot({ path: info.outputPath('starter-reload.png') })
 const old = await read(); old.version = 17; delete old.bench; if (old.stores) delete old.stores['scout:gear-bench:input']; delete old.inventory.equipment
 await page.evaluate(({ key, save }) => localStorage.setItem(key, JSON.stringify(save)), { key: RESOURCE_SAVE_KEY, save: old })
 await page.reload(); await page.clock.runFor(1200)
 // Sole-clock survival checkpoint publishes migration; no new resources are granted.
 const migrated = await read(); expect(migrated.version).toBe(19)
 expect(migrated.inventory.equipment).toEqual(freshEquipment()); expect(migrated.inventory.quantities).toEqual(old.inventory.quantities)
 expect(migrated.sources).toEqual(old.sources)
 await page.reload(); await expect.poll(async () => { await page.clock.runFor(50); return scene(page).getAttribute('data-render-ready') }).toBe('true')
 await page.evaluate(() => document.fonts.ready); await page.clock.runFor(100)
 expect((await read()).inventory.equipment).toEqual(migrated.inventory.equipment)
 expect(errors).toEqual([])
 await info.attach('equipment-state.json', { body: JSON.stringify({ first, migrated, errors }, null, 2), contentType: 'application/json' })
})
