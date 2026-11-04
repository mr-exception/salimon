import {test} from 'node:test'
import assert from 'node:assert/strict'
import {ORIGIN,translate,LIGHT_SECOND_M} from './coordinates.ts'
import {radarGroups} from './radar.ts'
import type {Body} from './sampleUniverse.ts'
const body=(id:string,x:number,y:number):Body=>({id,name:id,kind:'planet',radiusKm:1,position:translate(ORIGIN,[x*LIGHT_SECOND_M,0,y*LIGHT_SECOND_M])})
const project=(p:readonly number[])=>[p[0]/100,p[2]/100,0] as const
test('out-of-view contacts have no presentation group, including depth and malformed projections',()=>{
 assert.deepEqual(radarGroups([body('outside',200,50)],ORIGIN,project,800,600),[])
 for(const ndc of [[0,0,2],[0,0,-2],[NaN,0,0],[0,Infinity,0]] as const)assert.deepEqual(radarGroups([body('x',1,1)],ORIGIN,()=>ndc,800,600),[])
 assert.equal(radarGroups([body('inside',95,90)],ORIGIN,project,800,600).length,1)
 assert.equal(radarGroups([body('boundary',100,100)],ORIGIN,project,800,600).length,1)
 assert.deepEqual(radarGroups([body('x',0,0)],ORIGIN,project,0,600),[])
})
test('visible dense groups retain every ID, deterministic selected representative and bounded presentation',()=>{
 const contacts=Array.from({length:2000},(_,i)=>body(`contact-${String(i).padStart(4,'0')}`,10+i/10000,15+i/10000))
 const a=radarGroups(contacts,ORIGIN,project,800,600,'contact-1999')
 assert.deepEqual(a,radarGroups([...contacts].reverse(),ORIGIN,project,800,600,'contact-1999'))
 assert.equal(a.length,1);assert.equal(a[0].bodies.length,2000);assert.equal(a[0].representative,'contact-1999')
 assert.deepEqual(radarGroups([body('outside-range',100001,0)],ORIGIN,()=>[0,0,0],800,600),[])
})
