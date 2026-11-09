import { CELL_SIZE_M, LIGHT_SECOND_M, relativeM, translate, type Position } from './coordinates.ts'
import { length } from './math.ts'
import { SCANNER_RADIUS_LS, type Body } from './sampleUniverse.ts'
export const SCAN_TICKS = 300
export const SCANNER_RADIUS_M = SCANNER_RADIUS_LS*LIGHT_SECOND_M
const BUCKET_CELLS=32n
function floorDiv(a:bigint,b:bigint) {const q=a/b;return a%b<0n?q-1n:q}
function bucket(position:Position): readonly bigint[] {
 return position.cell.map((c,i)=>floorDiv(c,BUCKET_CELLS)-(c%BUCKET_CELLS===0n && position.offsetM[i]<0?1n:0n))
}
const key=(b:readonly bigint[])=>b.join(':')
export class CatalogIndex {
 private readonly buckets=new Map<string,Body[]>()
 constructor(bodies:readonly Body[]) {
  const ids=new Set<string>()
  for(const body of bodies) {
   if(!body.id || ids.has(body.id)) throw new Error('Catalog IDs must be unique and nonempty')
   if(!Number.isFinite(body.radiusKm)||body.radiusKm<=0 || !body.position.offsetM.every(v=>Number.isFinite(v)&&Math.abs(v)<=CELL_SIZE_M/2)) throw new Error('Invalid catalog body')
   if(body.angularVelocityRadps && !body.angularVelocityRadps.every(Number.isFinite))throw new Error('Invalid body rotation')
   ids.add(body.id)
   const k=key(bucket(body.position));const entries=this.buckets.get(k)??[];entries.push(body);this.buckets.set(k,entries)
  }
 }
 query(position:Position):Body[] {
  const r=SCANNER_RADIUS_M
  const min=bucket(translate(position,[-r,-r,-r])),max=bucket(translate(position,[r,r,r]))
  const result:Body[]=[]
  for(let x=min[0];x<=max[0];x++)for(let y=min[1];y<=max[1];y++)for(let z=min[2];z<=max[2];z++) {
   for(const body of this.buckets.get(key([x,y,z]))??[]) if(length(relativeM(body.position,position))<=r) result.push(body)
  }
  return result.sort((a,b)=>a.id.localeCompare(b.id))
 }
}
export interface ScannerSnapshot { lastTick: number; pulse: number; selected: string | null; contactIds: string[] }
export class Scanner {
 readonly index:CatalogIndex
 contacts:readonly Body[]=[]
 selected:string|null=null
 pulse=0
 private lastTick=0
 private readonly onSelect?: (snapshot: ScannerSnapshot) => void
 constructor(index:CatalogIndex, onSelect?: (snapshot: ScannerSnapshot) => void) {this.index=index;this.onSelect=onSelect}
 snapshot(): ScannerSnapshot { return { lastTick: this.lastTick, pulse: this.pulse, selected: this.selected, contactIds: this.contacts.map(b=>b.id) } }
 restore(value: ScannerSnapshot, bodies: readonly Body[]) {
  if (!Number.isSafeInteger(value.lastTick) || value.lastTick < 0 || value.pulse !== Math.floor(value.lastTick / SCAN_TICKS) || !Array.isArray(value.contactIds) || new Set(value.contactIds).size !== value.contactIds.length || value.contactIds.some(id=>!bodies.some(b=>b.id===id)) || value.selected !== null && !value.contactIds.includes(value.selected)) throw new Error('Invalid saved scanner')
  this.lastTick=value.lastTick; this.pulse=value.pulse; this.selected=value.selected; this.contacts=value.contactIds.map(id=>bodies.find(b=>b.id===id)!)
 }
 select(id:string) {if(this.contacts.some(b=>b.id===id)){this.selected=id;this.onSelect?.(this.snapshot())}}
 update(tick:number,position:Position) {
  if(!Number.isSafeInteger(tick)||tick<this.lastTick)throw new RangeError('Scanner requires monotonic simulation ticks')
  // Runtime calls on every fixed step; no detection before the first five seconds.
  const pulses=Math.floor(tick/SCAN_TICKS)-Math.floor(this.lastTick/SCAN_TICKS)
  if(pulses>0){this.contacts=this.index.query(position);this.pulse+=pulses}
  else this.contacts=this.contacts.filter(b=>length(relativeM(b.position,position))<=SCANNER_RADIUS_M)
  this.lastTick=tick
  if(this.selected && !this.contacts.some(b=>b.id===this.selected))this.selected=null
 }
}
