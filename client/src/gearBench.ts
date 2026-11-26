import * as q from './exact.ts'
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
export interface RepairRequest { readonly suitId:string; readonly amount:string; readonly initial:string }
export interface RepairReceipt extends RepairRequest { readonly jobId:number; readonly claimed:boolean }
export interface BenchJob { readonly repair?:RepairRequest; readonly id:number; readonly recipeId:string; readonly remainingTicks:number; readonly powerPaused:boolean }
export interface BenchTray { readonly repair?:RepairRequest; readonly jobId:number; readonly recipeId:string }
export interface BenchState {
 readonly repairs?:readonly RepairReceipt[];
 readonly started:number; readonly completed:Readonly<Record<string,number>>
 readonly claims:readonly { readonly jobId:number; readonly gearId:string; readonly recipeId:string }[]
 readonly job:BenchJob|null; readonly tray:BenchTray|null
}
export const freshBench=():BenchState=>({started:0,completed:{},claims:[],job:null,tray:null})
const integer=(v:unknown):number=>{if(typeof v!=='number'||!Number.isSafeInteger(v)||v<0)throw Error('Invalid bench integer');return v}
const record=(v:unknown,keys:string)=>{if(!v||typeof v!=='object'||Array.isArray(v)||Object.keys(v).sort().join()!==keys)throw Error('Invalid bench fields');return v as Record<string,unknown>}
export function validateBench(value:unknown):BenchState {
 const hasRepairs=!!value&&typeof value==='object'&&Object.hasOwn(value,'repairs')
 const s=record(value,hasRepairs?'claims,completed,job,repairs,started,tray':'claims,completed,job,started,tray');integer(s.started)
 const repairs=hasRepairs?s.repairs:[]
 if(!Array.isArray(repairs)||repairs.length>4096)throw Error('Invalid repair receipts')
 const repairJobs=new Set<number>()
 for(const raw of repairs){const r=record(raw,'amount,claimed,initial,jobId,suitId');validateRepair(r);if(!integer(r.jobId)||(r.jobId as number)>(s.started as number)||repairJobs.has(r.jobId as number)||typeof r.claimed!=='boolean')throw Error('Invalid repair receipt');repairJobs.add(r.jobId as number)}
 if(!s.completed||typeof s.completed!=='object'||Array.isArray(s.completed)||!Array.isArray(s.claims)||s.claims.length>4096)throw Error('Invalid bench receipts')
 let completed=0
 for(const [id,count] of Object.entries(s.completed)){if(!Object.hasOwn(GEAR_RECIPES,id)||!integer(count))throw Error('Unknown bench recipe');completed=integer(completed+(count as number))}
 if(s.job!==null){const j=record(s.job,Object.hasOwn(s.job as object,'repair')?'id,powerPaused,recipeId,remainingTicks,repair':'id,powerPaused,recipeId,remainingTicks');if(j.repair){record(j.repair,'amount,initial,suitId');validateRepair(j.repair);};if(j.id!==s.started||typeof j.recipeId!=='string'||!(j.recipeId==='repair'&&j.repair||Object.hasOwn(GEAR_RECIPES,j.recipeId)&&!j.repair)||!integer(j.remainingTicks)||(j.remainingTicks as number)>(j.repair?Math.ceil(repairCost(j.repair as RepairRequest)/BENCH_WATTS):GEAR_RECIPES[j.recipeId]!.ticks)||typeof j.powerPaused!=='boolean')throw Error('Invalid bench job')}
 if(s.tray!==null){const t=record(s.tray,Object.hasOwn(s.tray as object,'repair')?'jobId,recipeId,repair':'jobId,recipeId');if(t.repair){record(t.repair,'amount,initial,suitId');validateRepair(t.repair);};if(t.jobId!==s.started||typeof t.recipeId!=='string'||!(t.recipeId==='repair'&&t.repair||Object.hasOwn(GEAR_RECIPES,t.recipeId)&&!t.repair))throw Error('Invalid bench tray')}
 const allCompleted=completed+repairs.length
 if(allCompleted!== (s.started as number)-(s.job?1:0)||s.claims.length+repairs.filter(r=>r.claimed).length+(s.tray?1:0)!==allCompleted||s.job&&s.tray)throw Error('Invalid bench ownership')
 const ids=new Set<string>(),jobs=new Set<number>(), counts:Record<string,number>={}
 for(const claim of s.claims){const c=record(claim,'gearId,jobId,recipeId');if(!integer(c.jobId)||(c.jobId as number)>allCompleted||repairJobs.has(c.jobId as number)||jobs.has(c.jobId as number)||typeof c.gearId!=='string'||!new RegExp(`^gear:[0-9]+:bench-${c.jobId}$`).test(c.gearId)||ids.has(c.gearId)||typeof c.recipeId!=='string'||!Object.hasOwn(GEAR_RECIPES,c.recipeId))throw Error('Invalid bench claim');ids.add(c.gearId);jobs.add(c.jobId as number);counts[c.recipeId]=(counts[c.recipeId]??0)+1}
 if(s.tray){const t=s.tray as unknown as BenchTray;if(!t.repair)counts[t.recipeId]=(counts[t.recipeId]??0)+1}
 for(const id of new Set([...Object.keys(counts),...Object.keys(s.completed)]))if(counts[id]!== (s.completed as Record<string,number>)[id])throw Error('Bench recipe conservation failed')
 const state=structuredClone(value) as BenchState
 if(!Number.isSafeInteger(benchSpentJ60(state)))throw Error('Bench power overflow')
 return state
}
export const benchReserved=(s:BenchState)=>s.job?s.job.repair?{mineral:repairGrams(s.job.repair),biomass:0}:GEAR_RECIPES[s.job.recipeId]!:null
export const benchSpentJ60=(s:BenchState)=> Object.entries(s.completed).reduce((n,[id,count])=>n+GEAR_RECIPES[id]!.ticks*count*BENCH_WATTS,0)+(s.repairs??[]).reduce((n,r)=>n+repairCost(r),0)+(s.job?s.job.repair?repairCost(s.job.repair)-((s.job.remainingTicks-1)*BENCH_WATTS+repairCost(s.job.repair)-(Math.ceil(repairCost(s.job.repair)/BENCH_WATTS)-1)*BENCH_WATTS): (GEAR_RECIPES[s.job.recipeId]!.ticks-s.job.remainingTicks)*BENCH_WATTS:0)
export function benchTickCost(job:BenchJob):number {
 if(!job.repair||job.remainingTicks>1)return BENCH_WATTS
 const cost=repairCost(job.repair)
 return cost-(Math.ceil(cost/BENCH_WATTS)-1)*BENCH_WATTS
}
export function stepBench(s:BenchState,paid:number):BenchState {
 if(!s.job)return s
 if(paid!==0&&paid!==benchTickCost(s.job))throw Error('Invalid bench power receipt')
 if(!paid)return {...s,job:{...s.job,powerPaused:true}}
 if(s.job.remainingTicks>1)return {...s,job:{...s.job,remainingTicks:s.job.remainingTicks-1,powerPaused:false}}
 const j=s.job
 return j.repair?{...s,repairs:[...(s.repairs??[]),{...j.repair,jobId:j.id,claimed:false}],job:null,tray:{jobId:j.id,recipeId:'repair',repair:j.repair}}
 :{...s,completed:{...s.completed,[j.recipeId]:(s.completed[j.recipeId]??0)+1},job:null,tray:{jobId:j.id,recipeId:j.recipeId}}
}
const ceil=(v:q.Exact)=>Number((v[0]+v[1]-1n)/v[1])
export const repairCost=(r:RepairRequest)=>ceil(q.mul(q.decode(r.amount),q.decimal(6000)))
export const repairGrams=(r:RepairRequest)=>ceil(q.decode(r.amount))
export function validateRepair(value:unknown):asserts value is RepairRequest {
 const r=value as RepairRequest
 if(!r||typeof r.suitId!=='string'||!/^gear:[0-9]+:[a-zA-Z0-9_-]+$/.test(r.suitId))throw Error('Invalid repair suit')
 const hp=q.decode(r.amount),initial=q.decode(r.initial)
 if(q.cmp(hp,q.ZERO)<=0||q.cmp(initial,q.ZERO)<0||q.cmp(q.add(hp,initial),q.decimal(1000))>0)throw Error('Invalid requested repair amount')
}
/** Claimed current-generation manufactured identities cannot disappear or change definitions. */
export function validateBenchGear(s:BenchState,e:EquipmentState):void {
 for(const c of s.claims)if(c.gearId.startsWith(`gear:${e.generation}:`)&&!e.instances.some(g=>g.id===c.gearId&&g.definitionId===c.recipeId))throw Error('Missing claimed bench equipment')
 for(const r of s.repairs??[])if(!r.claimed&&(!s.tray?.repair||s.tray.jobId!==r.jobId))throw Error('Missing repaired output')
 const custody=s.job?.repair??s.tray?.repair
 const reserved=e.instances.filter(g=>g.location.kind==='bench')
 if(reserved.length!==(custody?1:0)||custody&&(!reserved.some(g=>g.id===custody.suitId&&GEAR_DEFINITIONS[g.definitionId]!.kind==='suit'&&g.remaining===(s.job?custody.initial:q.encode(q.add(q.decode(custody.initial),q.decode(custody.amount)))))))throw Error('Invalid repair custody')
 if(s.tray?.repair){const receipt=(s.repairs??[]).find(r=>r.jobId===s.tray!.jobId);if(!receipt||receipt.claimed||receipt.suitId!==s.tray.repair.suitId||receipt.amount!==s.tray.repair.amount||receipt.initial!==s.tray.repair.initial)throw Error('Invalid repair tray receipt')}
 for(const g of e.instances)if(g.id.includes(':bench-')&&!s.claims.some(c=>c.gearId===g.id&&c.recipeId===g.definitionId))throw Error('Unclaimed bench equipment')
}
