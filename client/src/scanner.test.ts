import {test} from 'node:test'
import assert from 'node:assert/strict'
import { ORIGIN, translate, type Position } from './coordinates.ts'
import { CatalogIndex, Scanner, SCANNER_RADIUS_M as R } from './scanner.ts'
import { radarGroups, gridOffsetLs } from './radar.ts'
import { type Body } from './sampleUniverse.ts'
const body=(id:string,x:number,anchor:Position=ORIGIN):Body=>({id,name:id,kind:'planet',radiusKm:1,position:translate(anchor,[x,0,0])})
test('indexed exact range includes boundary and rejects outside, including negative/huge anchors',()=>{
 for(const anchor of [ORIGIN,{cell:[-(10n**20n),10n**20n,0n] as const,offsetM:[-1,2,3] as const}]) {
  const index=new CatalogIndex([body('in',R,anchor),body('out',R+1,anchor),body('negative',-R,anchor),body('far',R*9,anchor)])
  assert.deepEqual(index.query(anchor).map(b=>b.id),['in','negative'])
 }
 assert.throws(()=>new CatalogIndex([body('same',0),body('same',1)]))
})
test('only five-second pulses detect; leaving range immediately evicts selected contact, with no history',()=>{
 const scanner=new Scanner(new CatalogIndex([body('x',R)]))
 scanner.update(299,ORIGIN);assert.equal(scanner.contacts.length,0)
 scanner.update(300,ORIGIN);assert.equal(scanner.pulse,1);scanner.select('x');assert.equal(scanner.selected,'x')
 scanner.update(301,translate(ORIGIN,[-1,0,0]));assert.equal(scanner.contacts.length,0);assert.equal(scanner.selected,null)
 scanner.update(599,ORIGIN);assert.equal(scanner.contacts.length,0)
 scanner.update(600,ORIGIN);assert.equal(scanner.contacts.length,1);assert.equal(scanner.selected,null)
 assert.throws(()=>scanner.update(599,ORIGIN))
})
test('large catalog query matches independent brute force without global precision loss',()=>{
 const bodies=Array.from({length:20000},(_,i)=>body(String(i),(i-10000)*R/100))
 const index=new CatalogIndex(bodies)
 assert.deepEqual(new Set(index.query(ORIGIN).map(b=>b.id)),new Set(bodies.filter((_,i)=>Math.abs((i-10000)*R/100)<=R).map(b=>b.id)))
})
test('dense presentation groups retain every detected ID; edge is frustum based, never radial',()=>{
 const dense=Array.from({length:10000},(_,i)=>body(String(i),i))
 const groups=radarGroups(dense,ORIGIN,()=>[0,0,0],800,600)
 assert.equal(groups.length,1);assert.equal(groups[0].bodies.length,10000)
 assert.deepEqual(radarGroups([body('edge',1)],ORIGIN,()=>[2,.5,0],800,600),[])
 assert.equal(radarGroups([body('hidden',R+1)],ORIGIN,()=>[0,0,0],800,600).length,0)
 assert.equal(radarGroups([body('corner',R)],ORIGIN,()=>[.8,.8,0],800,600).length,1)
 assert.deepEqual(radarGroups([body('depth',1)],ORIGIN,()=>[0,0,2],800,600),[])
})
test('grid lattice follows ship translation, preserving galactic orientation',()=>{
 const moved=translate(ORIGIN,[299792458,0,0]);assert.equal(gridOffsetLs(moved,100)[0],-1)
 assert.ok(gridOffsetLs({cell:[10n**20n,0n,0n],offsetM:[0,0,0]},100).every(Number.isFinite))
})

test('presentation count is bounded by viewport cells, not catalog length',()=>{
 const bodies=Array.from({length:20000},(_,i)=>body(String(i),i))
 let i=0
 const groups=radarGroups(bodies,ORIGIN,()=>{const n=i++;return [(n%200)/50-2,Math.floor(n/200)/25-2,0]},440,440)
 assert.ok(groups.length<=2*11*11)
 assert.ok(groups.reduce((sum,g)=>sum+g.bodies.length,0)<bodies.length,'off-screen contacts have no presentation group')
})
