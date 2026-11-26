import { GEAR_DEFINITIONS, type EquipmentState } from './equipment.ts'
import type { ResourceStoreCatalog } from './resourceLedger.ts'
export const BENCH_ID = 'scout:gear-bench'
export const BENCH_INPUT_ID = 'scout:gear-bench:input'
export const BENCH_WATTS = 2000
export const BENCH_STORES = { [BENCH_INPUT_ID]: { shipId:'ship:scout', capacityGrams:50_000 } } as const satisfies ResourceStoreCatalog
export interface GearRecipe { readonly definitionId:string; readonly mineral:number; readonly biomass:number; readonly ticks:number }
export const GEAR_RECIPES: Readonly<Record<string,GearRecipe>> = Object.freeze(Object.fromEntries(Object.entries(GEAR_DEFINITIONS).map(([id,d])=>{
 const tier=d.tier??0
 const [mineral,biomass,seconds]=d.kind==='suit'?[2000,1000,60]:tier===3?[3000,1000,60]:tier===2?[1500,500,30]:d.hazard==='temperature'?[400,100,15]:d.hazard==='radiation'?[500,0,15]:[200,300,15]
 return [id,Object.freeze({definitionId:id,mineral:mineral!,biomass:biomass!,ticks:seconds!*60})]
})))
export interface BenchJob { readonly id:number; readonly recipeId:string; readonly remainingTicks:number; readonly powerPaused:boolean }
export interface BenchTray { readonly jobId:number; readonly recipeId:string }
export interface BenchState {
 readonly started:number; readonly completed:Readonly<Record<string,number>>
 readonly claims:readonly { readonly jobId:number; readonly gearId:string; readonly recipeId:string }[]
 readonly job:BenchJob|null; readonly tray:BenchTray|null
}
export const freshBench=():BenchState=>({started:0,completed:{},claims:[],job:null,tray:null})
const integer=(v:unknown):number=>{if(typeof v!=='number'||!Number.isSafeInteger(v)||v<0)throw Error('Invalid bench integer');return v}
const record=(v:unknown,keys:string)=>{if(!v||typeof v!=='object'||Array.isArray(v)||Object.keys(v).sort().join()!==keys)throw Error('Invalid bench fields');return v as Record<string,unknown>}
export function validateBench(value:unknown):BenchState {
 const s=record(value,'claims,completed,job,started,tray');integer(s.started)
 if(!s.completed||typeof s.completed!=='object'||Array.isArray(s.completed)||!Array.isArray(s.claims)||s.claims.length>4096)throw Error('Invalid bench receipts')
 let completed=0
 for(const [id,count] of Object.entries(s.completed)){if(!Object.hasOwn(GEAR_RECIPES,id)||!integer(count))throw Error('Unknown bench recipe');completed=integer(completed+(count as number))}
 if(s.job!==null){const j=record(s.job,'id,powerPaused,recipeId,remainingTicks');if(j.id!==s.started||typeof j.recipeId!=='string'||!Object.hasOwn(GEAR_RECIPES,j.recipeId)||!integer(j.remainingTicks)||(j.remainingTicks as number)>GEAR_RECIPES[j.recipeId]!.ticks||typeof j.powerPaused!=='boolean')throw Error('Invalid bench job')}
 if(s.tray!==null){const t=record(s.tray,'jobId,recipeId');if(t.jobId!==s.started||typeof t.recipeId!=='string'||!Object.hasOwn(GEAR_RECIPES,t.recipeId))throw Error('Invalid bench tray')}
 if(completed!== (s.started as number)-(s.job?1:0)||s.claims.length+(s.tray?1:0)!==completed||s.job&&s.tray)throw Error('Invalid bench ownership')
 const ids=new Set<string>(),jobs=new Set<number>(), counts:Record<string,number>={}
 for(const claim of s.claims){const c=record(claim,'gearId,jobId,recipeId');if(!integer(c.jobId)||(c.jobId as number)>completed||jobs.has(c.jobId as number)||typeof c.gearId!=='string'||!new RegExp(`^gear:[0-9]+:bench-${c.jobId}$`).test(c.gearId)||ids.has(c.gearId)||typeof c.recipeId!=='string'||!Object.hasOwn(GEAR_RECIPES,c.recipeId))throw Error('Invalid bench claim');ids.add(c.gearId);jobs.add(c.jobId as number);counts[c.recipeId]=(counts[c.recipeId]??0)+1}
 if(s.tray){const t=s.tray as unknown as BenchTray;counts[t.recipeId]=(counts[t.recipeId]??0)+1}
 for(const id of new Set([...Object.keys(counts),...Object.keys(s.completed)]))if(counts[id]!== (s.completed as Record<string,number>)[id])throw Error('Bench recipe conservation failed')
 const state=structuredClone(value) as BenchState
 if(!Number.isSafeInteger(benchSpentJ60(state)))throw Error('Bench power overflow')
 return state
}
export const benchReserved=(s:BenchState)=>s.job?GEAR_RECIPES[s.job.recipeId]!:null
export const benchSpentJ60=(s:BenchState)=> (Object.entries(s.completed).reduce((n,[id,count])=>n+GEAR_RECIPES[id]!.ticks*count,0)+(s.job?GEAR_RECIPES[s.job.recipeId]!.ticks-s.job.remainingTicks:0))*BENCH_WATTS
export function stepBench(s:BenchState,paid:number):BenchState {
 if(!s.job)return s
 if(paid!==0&&paid!==BENCH_WATTS)throw Error('Invalid bench power receipt')
 if(!paid)return {...s,job:{...s.job,powerPaused:true}}
 if(s.job.remainingTicks>1)return {...s,job:{...s.job,remainingTicks:s.job.remainingTicks-1,powerPaused:false}}
 return {...s,completed:{...s.completed,[s.job.recipeId]:(s.completed[s.job.recipeId]??0)+1},job:null,tray:{jobId:s.job.id,recipeId:s.job.recipeId}}
}
/** Claimed current-generation manufactured identities cannot disappear or change definitions. */
export function validateBenchGear(s:BenchState,e:EquipmentState):void {
 for(const c of s.claims)if(c.gearId.startsWith(`gear:${e.generation}:`)&&!e.instances.some(g=>g.id===c.gearId&&g.definitionId===c.recipeId))throw Error('Missing claimed bench equipment')
 for(const g of e.instances)if(g.id.includes(':bench-')&&!s.claims.some(c=>c.gearId===g.id&&c.recipeId===g.definitionId))throw Error('Unclaimed bench equipment')
}
