import { CELL_SIZE_M, radarPosition, relativeM, type Position } from './coordinates.ts'
import { length, type Vec3 } from './math.ts'
import { SCANNER_RADIUS_M } from './scanner.ts'
import { type Body } from './sampleUniverse.ts'
export interface ContactGroup {readonly id:string;readonly bodies:readonly Body[];readonly position:Vec3;readonly x:number;readonly y:number;readonly edge:boolean}
// One symbol/button per 44 CSS-pixel cell. Detection stays complete in Scanner;
// presentation groups retain every ID and expose a paged selection menu.
export function radarGroups(contacts:readonly Body[],ship:Position,project:(p:Vec3)=>Vec3,width:number,height:number):ContactGroup[] {
 const groups=new Map<string,{bodies:Body[];position:Vec3;x:number;y:number;edge:boolean}>()
 const cols=Math.max(1,Math.floor(width/44)),rows=Math.max(1,Math.floor(height/44))
 for(const body of contacts) {
  if(length(relativeM(body.position,ship))>SCANNER_RADIUS_M)continue
  const position=radarPosition(body.position,ship),ndc=project(position)
  if(!ndc.every(Number.isFinite))continue
  const edge=Math.abs(ndc[0])>0.9||Math.abs(ndc[1])>0.83||Math.abs(ndc[2])>1
  let x=ndc[0],y=ndc[1]
  if(edge) {
   const ratio=Math.max(Math.abs(x)/0.9,Math.abs(y)/0.83,1)
   x/=ratio;y/=ratio
   // Depth-only clipping still gets an honest border marker.
   if(Math.abs(x)<0.9 && Math.abs(y)<0.83) y=y>=0?0.83:-0.83
  }
  const sx=(x+1)/2,sy=(1-y)/2
  const k=`${edge?'edge':'visible'}:${Math.floor(sx*cols)}:${Math.floor(sy*rows)}`
  const group=groups.get(k)
  if(group)group.bodies.push(body)
  else groups.set(k,{bodies:[body],position,x:sx*100,y:sy*100,edge})
 }
 return [...groups.entries()].map(([id,g])=>({id,...g}))
}
// Anchor the local grid to a galactic lattice without narrowing global positions.
export function gridOffsetLs(ship:Position,spacingLs:number):Vec3 {
 const spacingM=BigInt(Math.round(spacingLs*299_792_458))
 return ship.cell.map((c,i)=>-((Number(c*BigInt(CELL_SIZE_M)%spacingM)+ship.offsetM[i])%Number(spacingM))/299_792_458) as [number,number,number]
}
