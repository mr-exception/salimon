import type { GroundSample } from './regionStream.ts'
import type { Vec3 } from './math.ts'

export type InkPoint = readonly [number, number]
/** Flat pigment veins only: no raised silhouette or collectable decal. */
export function sedimentField(p:Vec3,seed:string):number {
  const phase=Number(BigInt(`0x${seed}`)%65521n)/65521*6.283185
  return Math.sin(p[0]*.24+p[1]*.19-p[2]*.13+phase)+.45*Math.sin(p[0]*.51-p[1]*.11+p[2]*.39+phase)
}
const mix = (a: readonly number[], b: readonly number[], t: number) => a.map((n,i) => n+(b[i]!-n)*Math.max(0,Math.min(1,t)))
/** Continuous pigments; fine albedo is body-fixed, seeded and cosmetic. */
export function terrainPigment(sample: GroundSample, physical: Vec3, seed: string): number[] {
  const w=sample.weights, phase=Number(BigInt(`0x${seed}`)%65521n)/65521*6.283185
  let color=mix([179,160,132],[133,131,118],w.highland)
  color=mix(color,[160,140,119],w.impact*.45)
  color=mix(color,[103,108,105],w.fracture*.65)
  color=mix(color,[187,211,216],w.ice)
  const grain=Math.sin(physical[0]*.49+physical[1]*.31+physical[2]*.27+phase)*Math.sin(physical[0]*.17-physical[2]*.37+phase)
  const wash=sedimentField(physical,seed)
  return color.map(n=>Math.round(n+grain*5+wash*6-Math.min(24,sample.grade*24)))
}
/** Marching triangles avoid ambiguous saddles. Absolute physical contour
 * levels survive patch replacement and region boundaries. */
export function contourTriangle(points: readonly InkPoint[], values: readonly number[], level: number): string {
  const hits: InkPoint[]=[]
  for(let i=0;i<3;i++) {
    const j=(i+1)%3, a=values[i]!,b=values[j]!
    if((a<level)===(b<level))continue
    const t=(level-a)/(b-a),p=points[i]!,q=points[j]!
    hits.push([p[0]+(q[0]-p[0])*t,p[1]+(q[1]-p[1])*t])
  }
  return hits.length===2 ? `M${hits[0]![0]},${hits[0]![1]}L${hits[1]![0]},${hits[1]![1]}` : ''
}
/** Clip a physical grade triangle into a continuous overhead rock footprint. */
export function gradeFootprint(points:readonly InkPoint[],values:readonly number[],level:number):string {
  const polygon:InkPoint[]=[]
  for(let i=0;i<3;i++) {
    const j=(i+1)%3,p=points[i]!,q=points[j]!,a=values[i]!,b=values[j]!
    if(a>=level)polygon.push(p)
    if((a<level)!==(b<level)) {
      const t=(level-a)/(b-a);polygon.push([p[0]+(q[0]-p[0])*t,p[1]+(q[1]-p[1])*t])
    }
  }
  return polygon.length>=3 ? `M${polygon.map(p=>p.join(',')).join('L')}Z` : ''
}
/** Fixed sample budget; diagonal coverage encloses every ship-up rotation. */
export function terrainPatch(cx:number,cy:number,width:number,height:number,pixelsPerM:number) {
  const step=2*Math.max(1,2**Math.ceil(Math.log2(Math.hypot(width,height)/pixelsPerM/64)))
  const x=Math.floor(cx/(step*4))*step*4-step*20,y=Math.floor(cy/(step*4))*step*4-step*20
  return {x,y,step,count:41}
}
