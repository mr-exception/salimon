import { useMemo } from 'react'
import { ORIGIN, relativeM } from './coordinates.ts'
import { sampleGeologyBatch, WALK_MAX_SLOPE_RADIANS, TERRAIN_CURVATURE_BOUND } from './planetGeology.ts'
import { localToSurface, surfaceRegion } from './surfaceCoordinates.ts'
import { inPlanet, point2, relativeAnchor, worldToView } from './topDownFrames.ts'
import type { TopDownState } from './topDownRuntime.ts'

/** Minimal physical ground readability for P02, not P04's illustrated terrain.
 * Bounded 40×40 preview follows the camera; no terrain entities enter saves.
 * Elevation/material/steep rock all consume the authoritative continuous field. */
export function GeneratedGround({state}: {state: TopDownState}) {
  const address = state.scoutLocation.surface, reference = address.definition
  const active = !!state.scoutLocation.binding && reference?.version === 2
  const [cx, cy] = active ? relativeM(inPlanet(state.camera.anchor, state.frames), ORIGIN) : [0, 0]
  const x0 = Math.floor(cx / 8)*8-40, y0 = Math.floor(cy / 8)*8-40
  const cells = useMemo(() => {
    if (!active || !reference) return []
    const region = surfaceRegion(address, reference)
    const points = Array.from({length: 1600}, (_, i) => [x0 + i%40*2, y0 + Math.floor(i/40)*2] as const)
    const samples = sampleGeologyBatch(points.map(([x,y]) => localToSurface(region, [x+1,y+1])), reference)
    return points.map(([x,y],i) => ({x,y,sample:samples[i]!}))
  // Physical addresses/ref are immutable for a surface binding. Movement updates
  // camera, not the field; cache a bounded patch until its 8 m origin changes.
  }, [active, address, reference, x0, y0])
  const colors = {'basin-regolith':'#a7977e','highland-rock':'#918878','fractured-rock':'#736f66','polar-ice':'#becbd0','authored-ground':'#b5a58b'}
  return <g data-generated-ground={active ? reference.digest : undefined}>
    {cells.filter(c => c.sample.groundType !== 'authored-ground').map(({x,y,sample}) => {
      const [vx,vy] = worldToView(relativeAnchor({frame:'planet:fixture',point:point2(x,y)},state.camera.anchor,state.frames),state.frames)
      const steep = sample.grade + TERRAIN_CURVATURE_BOUND*(.46 + .25 + Math.SQRT2) > Math.tan(WALK_MAX_SLOPE_RADIANS)
      return <g key={`${x}:${y}`} data-ground-position={`${x+1},${y+1}`} data-ground-slope={sample.slopeRadians} data-ground-type={sample.groundType} data-ground-steep={String(steep)} transform={`translate(${vx*state.camera.pixelsPerM} ${vy*state.camera.pixelsPerM}) rotate(${-state.frames.ship.angle*180/Math.PI}) scale(${state.camera.pixelsPerM})`}>
        <rect width="2.01" height="2.01" fill={colors[sample.groundType]} />
        <rect width="2.01" height="2.01" fill="#28342f" opacity={Math.min(.28,sample.grade*.32)} />
        <path d={steep ? 'M.3 1.8l.35-.6-.2-.35.65-.6m-.45.95.65.15.45-.35' : 'M.2 1.6L1.2 1.4'} stroke={steep ? '#383d3b' : '#c5baa4'} strokeWidth={steep ? '.055' : '.04'} opacity={steep ? .8 : .35} />
      </g>
    })}
  </g>
}
