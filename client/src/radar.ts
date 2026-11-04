import { CELL_SIZE_M, radarPosition, relativeM, type Position } from './coordinates.ts'
import { length, type Vec3 } from './math.ts'
import { SCANNER_RADIUS_M } from './scanner.ts'
import { type Body } from './sampleUniverse.ts'
export interface ContactGroup {
 readonly id:string;readonly bodies:readonly Body[];readonly position:Vec3
 readonly x:number;readonly y:number;readonly representative:string
}
// Only visible contacts have scene groups. Every scanned ID remains accessible
// through the separate paged picker, including contacts outside the viewport.
export function radarGroups(contacts:readonly Body[],ship:Position,project:(p:Vec3)=>Vec3,width:number,height:number,selected:string|null=null):ContactGroup[] {
 if(!Number.isFinite(width)||!Number.isFinite(height)||width<=0||height<=0)return []
 type Group={bodies:Body[];position:Vec3;x:number;y:number;representative:string;distance:number}
 const groups=new Map<string,Group>()
 const cols=Math.max(1,Math.floor(width/44)),rows=Math.max(1,Math.floor(height/44))
 for(const body of contacts) {
  const distance=length(relativeM(body.position,ship))
  if(!Number.isFinite(distance)||distance>SCANNER_RADIUS_M)continue
  const position=radarPosition(body.position,ship),ndc=project(position)
  if(!ndc.every(Number.isFinite)||ndc.some(v=>Math.abs(v)>1))continue
  const x=(ndc[0]+1)*50,y=(1-ndc[1])*50
  const k=`visible:${Math.min(cols-1,Math.floor(x/100*cols))}:${Math.min(rows-1,Math.floor(y/100*rows))}`
  const candidate={position,x,y,representative:body.id,distance}
  const group=groups.get(k)
  if(group) {
   group.bodies.push(body)
   if(body.id===selected||(group.representative!==selected&&(distance<group.distance||(distance===group.distance&&body.id<group.representative))))Object.assign(group,candidate)
  } else groups.set(k,{bodies:[body],...candidate})
 }
 return [...groups.entries()].sort(([a],[b])=>a.localeCompare(b)).map(([id,{distance:_,...g}])=>({id,...g,bodies:g.bodies.sort((a,b)=>a.id.localeCompare(b.id))}))
}
// Anchor the local lattice to a galactic frame without narrowing global positions.
export function gridOffsetLs(ship:Position,spacingLs:number):Vec3 {
 const spacingM=BigInt(Math.round(spacingLs*299_792_458))
 return ship.cell.map((c,i)=>-((Number(c*BigInt(CELL_SIZE_M)%spacingM)+ship.offsetM[i])%Number(spacingM))/299_792_458) as [number,number,number]
}
