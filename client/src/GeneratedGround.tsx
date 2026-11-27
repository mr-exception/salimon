import { useMemo, useRef } from 'react'
import { RegionStream } from './regionStream.ts'
import { ORIGIN, relativeM } from './coordinates.ts'
import { WALK_MAX_SLOPE_RADIANS, TERRAIN_CURVATURE_BOUND } from './planetGeology.ts'
import { localToSurface, surfaceRegion } from './surfaceCoordinates.ts'
import { inPlanet, point2, relativeAnchor, worldToView } from './topDownFrames.ts'
import { planetDefinition } from './planetDefinition.ts'
import { contourTriangle, gradeFootprint, sedimentField, terrainPatch, terrainPigment } from './terrainIllustration.ts'
import type { TopDownState } from './topDownRuntime.ts'

/** Illustrated albedo and physical contours below people/hull. No new objects,
 * collision, resource identities or persistence state enter this display. */
export function GeneratedGround({state,width,height,rasterScale=1,detail=.75,diagnostics=false}: {state: TopDownState;width:number;height:number;rasterScale?:number;detail?:number;diagnostics?:boolean}) {
  const stream=useRef<RegionStream | null>(null)
  stream.current ??= new RegionStream()
  const address=state.scoutLocation.surface,reference=address.definition
  const active=!!state.scoutLocation.binding && reference?.version===2
  const [cx,cy]=active ? relativeM(inPlanet(state.camera.anchor,state.frames),ORIGIN) : [0,0]
  const {x:x0,y:y0,step,count}=terrainPatch(cx,cy,width,height,state.camera.pixelsPerM)
  const chart=state.frames.surface?.chart
  const patch=useMemo(()=>{
    if(!active || !reference)return null
    const region=chart ?? surfaceRegion(address,reference),definition=planetDefinition(address.bodyId,reference)
    const player=state.entities.find(e=>e.id==='person:player')!
    const [px,py]=relativeM(inPlanet(player.anchor,state.frames),ORIGIN)
    stream.current!.select(localToSurface(region,[px,py]),reference)
    const vertices=Array.from({length:count*count},(_,i)=>{
      const x=x0+i%count*step,y=y0+Math.floor(i/count)*step,physical=localToSurface(region,[x,y])
      const sample=stream.current!.sample(physical)
      const radius=address.bodyId==='vesper' && physical.radialM[1]>0 ? Math.hypot(physical.radialM[0],physical.radialM[2]) : Infinity
      return {x,y,sample,sediment:sedimentField(physical.radialM,definition.domains.visual.seed),color:terrainPigment(sample,physical.radialM,definition.domains.visual.seed),alpha:Math.max(0,Math.min(1,(radius-190)/40))}
    })
    stream.current!.prefetch()
    // Small interpolated raster avoids hard rectangular material seams. Crisp
    // SVG contour ink supplies the illustrated geological structure above it.
    const dimension=Math.round(320*rasterScale)
    const canvas=document.createElement('canvas');canvas.width=dimension;canvas.height=dimension
    const context=canvas.getContext('2d')!,image=context.createImageData(dimension,dimension)
    for(let py=0;py<dimension;py++)for(let px=0;px<dimension;px++) {
      const gx=px*(count-1)/dimension,gy=py*(count-1)/dimension,ix=Math.floor(gx),iy=Math.floor(gy),u=gx-ix,v=gy-iy
      const a=vertices[iy*count+ix]!,b=vertices[iy*count+ix+1]!,c=vertices[(iy+1)*count+ix]!,d=vertices[(iy+1)*count+ix+1]!
      const k=(py*dimension+px)*4
      for(let channel=0;channel<3;channel++)image.data[k+channel]=(1-v)*((1-u)*a.color[channel]!+u*b.color[channel]!)+v*((1-u)*c.color[channel]!+u*d.color[channel]!)
      image.data[k+3]=255*((1-v)*((1-u)*a.alpha+u*b.alpha)+v*((1-u)*c.alpha+u*d.alpha))
    }
    context.putImageData(image,0,0)
    let contours='',ridges='',ice='',veins='',rock=''
    const cells=vertices.flatMap((a,i)=>{
      if(i%count===count-1 || i>=count*(count-1))return []
      const b=vertices[i+1]!,c=vertices[i+count]!,d=vertices[i+count+1]!
      if(a.alpha>0)for(const tri of [[a,b,d],[a,d,c]]) {
        const points=tri.map(p=>[p.x-x0,p.y-y0] as const),heights=tri.map(p=>p.sample.heightM)
        for(let h=Math.ceil(Math.min(...heights)/1.5)*1.5;h<=Math.max(...heights);h+=1.5)contours+=contourTriangle(points,heights,h)
        ridges+=contourTriangle(points,tri.map(p=>p.sample.grade),Math.tan(WALK_MAX_SLOPE_RADIANS)-TERRAIN_CURVATURE_BOUND*(.46+.25))
        rock+=gradeFootprint(points,tri.map(p=>p.sample.grade),Math.tan(WALK_MAX_SLOPE_RADIANS)-TERRAIN_CURVATURE_BOUND*(.46+.25))
        veins+=contourTriangle(points,tri.map(p=>p.sediment),.85)
        if(a.sample.weights.ice>.5)ice+=contourTriangle(points,heights,Math.ceil(Math.min(...heights)/.5)*.5)
      }
      return [a]
    })
    return {url:canvas.toDataURL(),contours,ridges,ice,veins,rock,cells,region}
  },[active,address,reference,chart?.id,x0,y0,step,count,rasterScale])
  if(!patch || !reference)return null
  const [vx,vy]=worldToView(relativeAnchor({frame:'planet:fixture',point:point2(x0,y0),chart},state.camera.anchor,state.frames),state.frames)
  return <g data-generated-ground={reference.digest} data-stream={JSON.stringify(stream.current.stats)} data-ground-chart={JSON.stringify(patch.region)} data-art-samples={count*count} pointerEvents="none" transform={`translate(${vx*state.camera.pixelsPerM} ${vy*state.camera.pixelsPerM}) rotate(${-state.frames.ship.angle*180/Math.PI}) scale(${state.camera.pixelsPerM})`}>
    <image href={patch.url} width={40*step} height={40*step} preserveAspectRatio="none" />
    <path data-terrain-footprint="steep" d={patch.rock} fill="#3f453f" opacity=".22" />
    <g fill="none" strokeLinecap="round" strokeLinejoin="round">
      <path d={patch.veins} stroke="#f2e5c9" strokeWidth=".055" opacity={.26*detail/.75} />
      <path data-terrain-contours="height" d={patch.contours} stroke="#d8c8a9" strokeWidth=".12" opacity=".45" />
      <path d={patch.contours} stroke="#554e46" strokeWidth=".035" opacity=".65" />
      <path d={patch.ice} stroke="#e5f1ef" strokeWidth=".065" opacity={.6*detail/.75} />
      <path data-terrain-contours="blocked-grade" d={patch.ridges} stroke="#484a43" strokeWidth=".16" />
      <path d={patch.ridges} stroke="#d8c8a9" strokeWidth=".045" />
    </g>
    {diagnostics && <g data-dev-render-chunk>
      <rect width={40*step} height={40*step} fill="none" stroke="#96cffc" strokeWidth=".12" strokeDasharray=".7 .4" />
      <text x=".5" y="1.4" fontSize=".52" fill="#e8f6ff">{patch.region.id}</text>
    </g>}
    {patch.cells.map(({x,y,sample})=><g key={`${x}:${y}`} data-ground-position={`${x},${y}`} data-ground-type={sample.groundType} data-ground-slope={sample.slopeRadians} data-ground-steep={String(sample.grade+TERRAIN_CURVATURE_BOUND*(.46+.25+step*Math.SQRT1_2)>Math.tan(WALK_MAX_SLOPE_RADIANS))} />)}
  </g>
}
