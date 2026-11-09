import { defineConfig } from '@playwright/test'
import {fileURLToPath} from 'node:url'
const launchOptions={executablePath:process.env.CHROMIUM_PATH,args:['--use-angle=swiftshader','--enable-unsafe-swiftshader','--disable-lcd-text']}
const referenceLaunchOptions={...launchOptions,env:{...process.env,FONTCONFIG_FILE:fileURLToPath(new URL('./tests/assets/fonts.conf',import.meta.url))}}
// SVG uses CPU rasterization; cockpit WebGL uses ANGLE SwiftShader.
const topDownLaunchOptions={...referenceLaunchOptions,args:[...launchOptions.args,'--disable-gpu']}
// Action traces and filmstrips retain failures without duplicating immutable
// sprite bytes in thousands of DOM snapshots. Explicit state JSON is retained.
const worldTrace={mode:'retain-on-failure',snapshots:false,screenshots:true,sources:true} as const
const evidenceDir=process.env.SALIMON_EVIDENCE_DIR??'test-results'
export default defineConfig({
 outputDir:evidenceDir,
 testDir:'./tests',timeout:30000,expect:{timeout:10000},workers:1,fullyParallel:false,retries:0,
 reporter:[['list'],['json',{outputFile:`${evidenceDir}/results.json`}]],
 use:{baseURL:'http://127.0.0.1:5175',viewport:{width:1280,height:800},deviceScaleFactor:1,locale:'en-US',timezoneId:'UTC',colorScheme:'dark',reducedMotion:'reduce',screenshot:'on',trace:'retain-on-failure',
  launchOptions},
 projects:[{name:'expedition',timeout:240000,testMatch:/expedition-captures\.spec\.ts/,use:{launchOptions:referenceLaunchOptions,trace:{mode:'retain-on-failure',snapshots:false,screenshots:false,sources:true}}},{name:'ui',testMatch:/ui\.spec\.ts/,use:{launchOptions:referenceLaunchOptions}},{name:'panoramic',timeout:180000,testMatch:/panoramic\.spec\.ts/,use:{viewport:{width:1280,height:720},trace:{mode:'retain-on-failure',snapshots:false,screenshots:false,sources:true}}},{name:'look-at',timeout:180000,testMatch:/look-at\.spec\.ts/,use:{viewport:{width:960,height:600},trace:{mode:'retain-on-failure',snapshots:false,screenshots:false,sources:true}}},{name:'perspective',timeout:180000,testMatch:/perspective\.spec\.ts/,use:{viewport:{width:960,height:600},trace:{mode:'on',snapshots:false,screenshots:false,sources:true}}},{name:'optical',timeout:120000,testMatch:/optical\.spec\.ts/},{name:'observation',testMatch:/observation\.spec\.ts/},{name:'cockpit',testMatch:/(boarding-door|flight|cockpit|playable-slice|gathering|storage|items|survival|hazards|viewport)\.spec\.ts/,timeout:180000,use:{launchOptions:topDownLaunchOptions,trace:worldTrace}},{name:'top-down',testMatch:/top-down\.spec\.ts/,use:{launchOptions:topDownLaunchOptions,trace:worldTrace}},{name:'navigation-captures',timeout:180000,testMatch:/(navigation|panoramic)-captures\.spec\.ts/,use:{trace:'on',launchOptions:referenceLaunchOptions}},{name:'deterministic',timeout:120000,testMatch:/deterministic\.spec\.ts/},{name:'desktop',testMatch:/(desktop|visuals|edges|panning)\.spec\.ts/},{name:'production',testMatch:/production\.spec\.ts/}],
 webServer:[
  {command:'npx vite build --config tests/hazard.vite.config.ts && npx vite preview --outDir .hazard-fixture --host 127.0.0.1 --port 4275 --strictPort',url:'http://127.0.0.1:4275/tests/hazards.html',reuseExistingServer:!process.env.CI},
  {command:'npm run dev -- --host 127.0.0.1 --port 5175 --strictPort',url:'http://127.0.0.1:5175',reuseExistingServer:!process.env.CI},
  {command:'npm run preview -- --host 127.0.0.1 --port 4175 --strictPort',url:'http://127.0.0.1:4175',reuseExistingServer:!process.env.CI}
 ]
})
