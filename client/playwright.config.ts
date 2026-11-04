import { defineConfig } from '@playwright/test'
export default defineConfig({
 testDir:'./tests',timeout:30000,expect:{timeout:10000},workers:1,fullyParallel:false,retries:0,
 reporter:[['list'],['json',{outputFile:'test-results/results.json'}]],
 use:{baseURL:'http://127.0.0.1:5175',viewport:{width:1280,height:800},screenshot:'on',trace:'retain-on-failure',
  launchOptions:{executablePath:process.env.CHROMIUM_PATH,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader']}},
 projects:[{name:'desktop',testMatch:/(desktop|visuals|edges|panning)\.spec\.ts/},{name:'production',testMatch:/production\.spec\.ts/}],
 webServer:[
  {command:'npm run dev -- --host 127.0.0.1 --port 5175 --strictPort',url:'http://127.0.0.1:5175',reuseExistingServer:!process.env.CI},
  {command:'npm run preview -- --host 127.0.0.1 --port 4175 --strictPort',url:'http://127.0.0.1:4175',reuseExistingServer:!process.env.CI}
 ]
})
