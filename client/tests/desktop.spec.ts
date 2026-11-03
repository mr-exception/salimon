import { test, expect, type Page } from '@playwright/test'
const stat=(page:Page,label:string)=>page.locator('.stat').filter({has:page.locator('span',{hasText:label})}).locator('strong')
const heading=(page:Page)=>stat(page,'Yaw / pitch')
async function stopped(page:Page) {await page.waitForTimeout(250);const value=await heading(page).textContent();await page.waitForTimeout(350);await expect(heading(page)).toHaveText(value!)}
test.beforeEach(async({page})=>{page.on('pageerror',error=>{throw error})})
test('scanner pulse, camera independence, clipping selection and manual alignment interruption',async({page})=>{
 await page.goto('/')
 await expect(page.locator('.scan-status')).toContainText('awaiting first')
 await expect(page.locator('#contact-select option')).toHaveCount(1)
 await expect(page.locator('.scan-status')).toContainText('5 temporary contacts',{timeout:15000})
 const ship=await page.locator('.stat').allTextContents(),before=await page.locator('canvas').screenshot()
 await page.mouse.move(300,300);await page.mouse.down();await page.mouse.move(550,420,{steps:10});await page.mouse.up();await page.waitForTimeout(500)
 expect(await page.locator('.stat').allTextContents()).toEqual(ship)
 expect((await page.locator('canvas').screenshot()).equals(before)).toBe(false)
 await page.getByRole('button',{name:'Reset camera',exact:true}).click()
 for(let i=0;i<4;i++)await page.getByRole('button',{name:'Zoom in',exact:true}).click()
 const edge=page.getByRole('button',{name:'◇ Caligo',exact:true});await expect(edge).toBeVisible();await edge.click()
 await expect(page.locator('.target-status')).toHaveText('Selected target: Caligo')
 await page.getByRole('button',{name:'Set direction',exact:true}).click();await expect(stat(page,'Alignment')).toHaveText('Turning')
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus();await page.keyboard.down('a');await page.waitForTimeout(200);await page.keyboard.up('a')
 await expect(stat(page,'Alignment')).toHaveText('Manual');await stopped(page)
 await page.getByRole('button',{name:'Reset camera',exact:true}).click();await expect(page.locator('.edge')).toHaveCount(0)
})
test('focused forms preserve keyboard behavior; button keyboard hold and blur release',async({page})=>{
 await page.goto('/');const initial=await heading(page).textContent()
 await page.getByLabel('Find scanned contacts').fill('wasd ');await page.keyboard.down('a');await page.waitForTimeout(250);await page.keyboard.up('a');await expect(heading(page)).toHaveText(initial!)
 const up=page.getByRole('button',{name:'Pitch up',exact:true});await up.focus();await page.keyboard.down('Space');await page.waitForTimeout(350);await page.keyboard.up('Space')
 await expect(heading(page)).not.toHaveText(initial!);await stopped(page)
 await up.focus();await page.keyboard.down('Enter');await page.waitForTimeout(200);await page.getByLabel('Find scanned contacts').focus();await page.keyboard.up('Enter');await stopped(page)
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus();await page.keyboard.down('d');await page.waitForTimeout(200);await page.evaluate(()=>window.dispatchEvent(new Event('blur')));await stopped(page);await page.keyboard.up('d')
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus();await page.keyboard.down('w');await page.waitForTimeout(200);await page.evaluate(()=>document.dispatchEvent(new Event('visibilitychange')));await stopped(page);await page.keyboard.up('w')
})
test('pointer capture releases outside the button, on cancellation and capture loss',async({page})=>{
 await page.goto('/')
 const left=page.getByRole('button',{name:'Yaw left',exact:true});await left.scrollIntoViewIfNeeded();const box=(await left.boundingBox())!
 await page.mouse.move(box.x+box.width/2,box.y+box.height/2);await page.mouse.down();await page.waitForTimeout(300);await page.mouse.move(30,30);await page.mouse.up();await stopped(page)
 for(const type of ['pointercancel','lostpointercapture']) {
  await left.scrollIntoViewIfNeeded();const b=(await left.boundingBox())!;await page.mouse.move(b.x+b.width/2,b.y+b.height/2);await page.mouse.down();await page.waitForTimeout(200)
  await left.dispatchEvent(type,{pointerId:1,pointerType:'mouse'});await stopped(page);await page.mouse.up()
 }
})
test('zero throttle coasts while ship rotates; focused slider keeps native keyboard input',async({page})=>{
 await page.goto('/');await page.getByLabel('Throttle',{exact:true}).fill('100');await page.waitForTimeout(450)
 await page.getByRole('button',{name:'Zero throttle (Space)',exact:true}).click();await page.waitForTimeout(250);const speed=await stat(page,'Speed').textContent();expect(speed).not.toBe('0.0 m/s')
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus();await page.keyboard.down('d');await page.waitForTimeout(400);await page.keyboard.up('d');await expect(stat(page,'Speed')).toHaveText(speed!)
 await page.getByLabel('Throttle',{exact:true}).focus();await page.keyboard.press('ArrowRight');await expect(page.getByLabel('Throttle',{exact:true})).toHaveValue('1')
 await page.keyboard.press('Space');await expect(page.getByLabel('Throttle',{exact:true})).toHaveValue('1')
 await page.getByRole('region',{name:'Navigation radar',exact:true}).focus();await page.keyboard.press('Space');await expect(page.getByLabel('Throttle',{exact:true})).toHaveValue('0')
})
for(const scenario of ['warning','destroyed','safe','asteroid'])test(`production App contact fixture: ${scenario}`,async({page})=>{
 await page.goto(`/tests/fixture.html?scenario=${scenario}`)
 if(scenario==='warning'){await expect(page.getByRole('status',{name:'Navigation warnings'})).toContainText('dangerous approach');return}
 await expect(page.locator('.contact-status')).toContainText(scenario==='destroyed'?'Journey ended':scenario==='safe'?'Safe surface contact':'Crash recovery')
 await expect(page.getByLabel('Throttle',{exact:true})).toBeDisabled();await expect(page.getByRole('button',{name:'Pitch up',exact:true})).toBeDisabled()
 await page.getByRole('button',{name:'Restart navigation',exact:true}).click();await expect(page.locator('.app')).toHaveAttribute('data-mode','flying');await expect(page.getByLabel('Throttle',{exact:true})).toBeEnabled()
})
test('dense groups and accessible contact picker keep all IDs with bounded DOM',async({page})=>{
 await page.goto('/tests/fixture.html?scenario=dense');await expect(page.locator('.scan-status')).toContainText('2000 temporary contacts')
 await expect(page.locator('#contact-select option')).toHaveCount(33)
 await page.getByRole('button',{name:'◉ 2000 contacts',exact:true}).click();await expect(page.locator('.contact-menu button')).toHaveCount(34)
 await page.getByRole('button',{name:'Next contacts',exact:true}).click();await page.getByRole('button',{name:'Dense contact 0032',exact:true}).click();await expect(page.locator('.target-status')).toContainText('Dense contact 0032')
 await page.getByLabel('Find scanned contacts').fill('1999');await page.getByLabel('Scanned contacts (2000)',{exact:true}).selectOption('dense-1999');await expect(page.locator('.target-status')).toContainText('Dense contact 1999')
})
