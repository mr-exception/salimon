import { CELL_SIZE_M, radarPosition, relativeM, type Position } from './coordinates.ts'
import { length, type Vec3 } from './math.ts'
import { SCANNER_RADIUS_M } from './scanner.ts'
import { type Body } from './sampleUniverse.ts'
export type Point2=readonly [number,number]
export interface DirectionLine {readonly start:Point2;readonly end:Point2}
export interface ContactGroup {
 readonly id:string;readonly bodies:readonly Body[];readonly position:Vec3
 readonly x:number;readonly y:number;readonly edge:boolean
 readonly line:DirectionLine|null;readonly representative:string;readonly depthOnly:boolean
}
/** Clip the ship→target segment to the actual NDC viewport, preserving direction.
 * A ship outside the view may produce two boundary intersections or no segment.
 * Parallel, coincident, tangent and non-finite projections never invent a line.
 */
export function clipViewportSegment(start:Point2,end:Point2):DirectionLine|null {
 if(![...start,...end].every(Number.isFinite))return null
 const dx=end[0]-start[0],dy=end[1]-start[1]
 if(!Number.isFinite(dx)||!Number.isFinite(dy)||(dx===0&&dy===0))return null
 let enter=0,leave=1
 for(const [p,q] of [[-dx,start[0]+1],[dx,1-start[0]],[-dy,start[1]+1],[dy,1-start[1]]]) {
  if(p===0){if(q<0)return null;continue}
  const t=q/p
  if(p<0)enter=Math.max(enter,t);else leave=Math.min(leave,t)
  if(enter>=leave)return null
 }
 const clamp=(v:number)=>Math.max(-1,Math.min(1,v))
 return {start:[clamp(start[0]+enter*dx),clamp(start[1]+enter*dy)],end:[clamp(start[0]+leave*dx),clamp(start[1]+leave*dy)]}
}
const percent=(p:Point2):Point2=>[(p[0]+1)*50,(1-p[1])*50]
// Odd slot counts give cardinal directions a stable central slot instead of a
// bin boundary, so nearly collinear rays do not split at the screen midpoint.
function axisSlot(value:number,extent:number,inset:number,gap:number):{slot:number;center:number} {
 const first=Math.min(extent/2,inset+gap),last=extent-first
 let slots=Math.max(1,Math.floor((last-first)/gap)+1)
 if(slots%2===0)slots--
 if(slots===1)return {slot:0,center:extent/2}
 const span=(last-first)/(slots-1),slot=Math.max(0,Math.min(slots-1,Math.round((value-first)/span)))
 return {slot,center:first+slot*span}
}
// Stable perimeter slots leave space for labels and merge collinear/nearby rays.
// The representative ray retains its exact endpoint, independently of its label.
function edgeSlot(point:Point2,width:number,height:number):{id:string;x:number;y:number} {
 const [x,y]=percent(point),px=x*width/100,py=y*height/100
 const insetX=Math.min(95,width/2),insetY=Math.min(82,height/2)
 const left=px<=insetX,right=px>=width-insetX,top=py<=insetY+28,bottom=py>=height-insetY-28
 if((left||right)&&(top||bottom))return {id:`corner:${left?'l':'r'}${top?'t':'b'}`,x:(left?insetX:width-insetX)/width*100,y:(top?insetY:height-insetY)/height*100}
 if(top||bottom) {
  const {slot,center}=axisSlot(px,width,insetX,190)
  return {id:`${top?'top':'bottom'}:${slot}`,x:center/width*100,y:(top?insetY:height-insetY)/height*100}
 }
 const {slot,center}=axisSlot(py,height,insetY,56)
 return {id:`${left?'left':'right'}:${slot}`,x:(left?insetX:width-insetX)/width*100,y:center/height*100}
}
// Detection stays complete in Scanner. Visible 44px bins and perimeter slots
// retain every ID, while one representative ray/mesh bounds presentation.
export function radarGroups(contacts:readonly Body[],ship:Position,project:(p:Vec3)=>Vec3,width:number,height:number,selected:string|null=null):ContactGroup[] {
 if(!Number.isFinite(width)||!Number.isFinite(height)||width<=0||height<=0)return []
 type MutableGroup={bodies:Body[];position:Vec3;x:number;y:number;edge:boolean;line:DirectionLine|null;representative:string;depthOnly:boolean;distance:number}
 const groups=new Map<string,MutableGroup>()
 const cols=Math.max(1,Math.floor(width/44)),rows=Math.max(1,Math.floor(height/44))
 const shipNdc=project([0,0,0])
 for(const body of contacts) {
  const distance=length(relativeM(body.position,ship))
  if(!Number.isFinite(distance)||distance>SCANNER_RADIUS_M)continue
  const position=radarPosition(body.position,ship),ndc=project(position)
  if(!ndc.every(Number.isFinite))continue
  const outsideXY=Math.abs(ndc[0])>1||Math.abs(ndc[1])>1
  const depthOnly=!outsideXY&&Math.abs(ndc[2])>1
  const edge=outsideXY||depthOnly
  const clipped=outsideXY?clipViewportSegment([shipNdc[0],shipNdc[1]],[ndc[0],ndc[1]]):null
  const line=clipped?{start:percent(clipped.start),end:percent(clipped.end)}:null
  const projected=percent([ndc[0],ndc[1]])
  const ratio=Math.max(Math.abs(ndc[0]),Math.abs(ndc[1]),1)
  const anchor=clipped?.end??(depthOnly?[0,1] as const:[ndc[0]/ratio,ndc[1]/ratio] as const)
  const slot=edge?edgeSlot(anchor,width,height):{id:`${Math.floor(projected[0]/100*cols)}:${Math.floor(projected[1]/100*rows)}`,x:projected[0],y:projected[1]}
  const k=`${edge?'edge':'visible'}:${slot.id}`
  const candidate={position,x:slot.x,y:slot.y,edge,line,representative:body.id,depthOnly,distance}
  const group=groups.get(k)
  if(group) {
   group.bodies.push(body)
   const priority=body.id===selected?0:edge&&!line?2:1,previous=group.representative===selected?0:group.edge&&!group.line?2:1
   if(priority<previous||(priority===previous&&(distance<group.distance||(distance===group.distance&&body.id<group.representative))))Object.assign(group,candidate)
  } else groups.set(k,{bodies:[body],...candidate})
 }
 return [...groups.entries()].sort(([a],[b])=>a.localeCompare(b)).map(([id,{distance:_,...g}])=>({id,...g,bodies:g.bodies.sort((a,b)=>a.id.localeCompare(b.id))}))
}
// Anchor the local lattice to a galactic frame without narrowing global positions.
export function gridOffsetLs(ship:Position,spacingLs:number):Vec3 {
 const spacingM=BigInt(Math.round(spacingLs*299_792_458))
 return ship.cell.map((c,i)=>-((Number(c*BigInt(CELL_SIZE_M)%spacingM)+ship.offsetM[i])%Number(spacingM))/299_792_458) as [number,number,number]
}
