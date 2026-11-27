import type { ReactNode } from 'react'
import type { Entity2D } from '../topDownRuntime.ts'
import { SCOUT_ENTRY_HALF_WIDTH_M } from '../scoutLayout.ts'
import type { WallJoins } from './scoutArt.ts'

const ivory = '#d8d5c4', graphite = '#303942', slate = '#788894', ochre = '#b68b51'

function PropulsionHousing() {
  return <g data-scout-module="propulsion">
    <path d="M2.83 2.5h.55l.32.33v2.56l-.2.38h-.67Z" fill="#4c5b60" stroke={graphite} strokeWidth=".13" />
    <path d="M3.16 2.82h.24v2.49h-.24z" fill={slate} />
    <path d="M3.2 3.04h.18m-.18.29h.18m-.18.29h.18m-.18 1.1h.18" stroke="#36474d" strokeWidth=".08" />
    <path d="M2.95 5.36h.58l-.15.34h-.42Z" fill="#2a353b" />
  </g>
}

function HullSide() {
  return <g>
    <PropulsionHousing />
    <g data-scout-module="hull-rail">
      <path d="M2.59-4.63h.45V5.49h-.45Z" fill={ivory} stroke={graphite} strokeWidth=".12" />
      <path d="M2.87-4.56v9.94" stroke="#a8a49a" strokeWidth=".08" />
      <path d="M2.59-3.86h.45m-.45 2.97h.45m-.45 2.97h.45m-.45 2.43h.45" stroke="#848b8a" strokeWidth=".055" />
      <path d="M2.68 3.34h.12m-.09.08h.17m-.2-5.42h.14" stroke="#767a74" strokeWidth=".04" />
    </g>
    <g data-scout-module="hull-corner">
      <path d="M2.58 5.28h.46v.24l-.88.98h-.68v-.63h.52Z" fill={ivory} stroke={graphite} strokeWidth=".12" />
      <path d="m2.92 5.52-.82.86h-.47v-.42h.41l.7-.76" fill="#a8aaa2" />
      <path d="M1.52 5.94h.48v.52h-.48Z" fill={ochre} />
    </g>
  </g>
}

function ScoutHull() {
  return <g data-scout-module="hull" strokeLinejoin="round">
    <HullSide />
    <g transform="scale(-1 1)"><HullSide /></g>
    <g data-scout-module="bow">
      <path d="m-3.04-4.6 1.27-2.4h3.54L3.04-4.6h-.49L1.45-6.48h-2.9L-2.55-4.6Z" fill={ivory} stroke={graphite} strokeWidth=".13"/>
      <path d="M-1.6-6.8h3.2" stroke={slate} strokeWidth=".08" />
      <path d="M-2.62-4.68l.42-.65m4.82.65-.42-.65" stroke={ochre} strokeWidth=".08" />
    </g>
  </g>
}

function ScoutDeck() {
  return <g data-scout-module="deck">
    <path data-scout-module="cockpit-floor" d="M-2.6-4.54-1.52-6.49h3.04l1.08 1.95v2.24h-5.2Z" fill="#596b74" />
    <path data-scout-module="cabin-floor" d="M-2.6-2.3h5.2v5h-5.2Z" fill="#d8c9aa" />
    <path data-scout-module="cargo-floor" d="M-2.6 2.7h5.2v2.7l-.66.72h-3.88l-.66-.72Z" fill="#77817f" />
    <path d="M-2.5-2.3H2.5m-5 5H2.5" stroke="#ad9a81" strokeWidth=".10" />
    <path d="M-2.28-4.4v1.84m4.56-1.84v1.84M-2.28 3.12v2.08m4.56-2.08v2.08" stroke="#adb9b6" strokeWidth=".08" />
    <path d="M-1.95-1.77h3.9m-3.9 3.94h3.9m-3.9-4.09v4.2m3.9-4.2v4.2" stroke="#b7a78d" strokeWidth=".04" />
    <path d="M-1.87 3.1H1.87m-3.6 1.36h3.46m-3.3.88h3.14" stroke="#697475" strokeWidth=".055"/>
    <rect x="-1.07" y=".86" width="2.14" height="1.47" rx=".08" fill="#ae7f65" opacity=".24" />
    <path d="M-1 .95h2m-2 1.28h2" stroke="#a78573" strokeWidth=".055" />
    <path d="M-2.2-2.13H-.84m1.68 0H2.2m-4.4 4.67H-.84m1.68 0H2.2" stroke={ochre} strokeWidth=".065" />
  </g>
}

function WindowPanel({ width, depth }: { width: number; depth: number }) {
  return <g data-scout-module="window">
    <rect x={-width / 2} y={-depth / 2} width={width} height={depth} rx=".045" fill="#354e56" />
    <rect x={-width / 2 + .08} y={-depth * .27} width={width - .16} height={depth * .54} rx=".025" fill="#65a6aa" />
    <path d={`M${-width / 2 + .13} ${-depth * .16}h${width - .26}M-.47 ${-depth / 2}v${depth}m.94 ${-depth}v${depth}`}
      stroke="#a0d3ca" strokeWidth=".03" />
  </g>
}

export function ScoutWallArt({ entity, joins, part }: { entity: Entity2D; joins: WallJoins; part: 'base' | 'cap' }) {
  const { halfWidth, halfHeight } = entity.footprint!
  const horizontal = halfWidth >= halfHeight
  const length = 2 * (horizontal ? halfWidth : halfHeight)
  const depth = 2 * (horizontal ? halfHeight : halfWidth)
  const corner = Math.min(.065, depth / 3)
  const left = -length / 2, right = length / 2, top = -depth / 2, bottom = depth / 2
  const start = joins[0] ? 0 : corner, end = joins[1] ? 0 : corner
  return <g transform={horizontal ? undefined : 'rotate(90)'}>
    {part === 'base' ? <path d={`M${left} ${top + start}L${left + start} ${top}H${right - end}L${right} ${top + end}V${bottom - end}L${right - end} ${bottom}H${left + start}L${left} ${bottom - start}Z`}
      fill="#4b5b5e" /> : <>
      <path d={`M${left + start} ${top + .035}H${right - end}V${bottom - .05}H${left + start}Z`}
        fill="#a9aeaa" />
      <path d={`M${left + start} ${bottom - .065}H${right - end}`} stroke="#758080" strokeWidth=".035" />
    </>}
  </g>
}

export function ScoutWall({ entity, joins, fade, base, cap }: {
  entity: Entity2D; joins: WallJoins; fade: boolean; base: ReactNode; cap: ReactNode
}) {
  return <g data-scout-module="wall" data-scout-joins={joins.map(join => join ? 'joined' : 'capped').join(' ')}>
    {base}
    <g className="scout-obstruction" data-scout-faded={String(fade)}>
      {cap}
      {entity.id === 'wall:nose' && <WindowPanel width={2 * entity.footprint!.halfWidth - .38} depth={2 * entity.footprint!.halfHeight} />}
    </g>
  </g>
}

function BoardingDoor({ closed }: { closed: boolean }) {
  const edge = SCOUT_ENTRY_HALF_WIDTH_M
  return <g data-scout-module="boarding-door">
    <rect x={-edge - .15} y="-.3" width={2 * edge + .3} height={closed ? .6 : 1.2} fill="transparent" />
    <path d={`M${-edge}-.13H${edge}v.28H${-edge}Z`} fill="#666866" />
    <path d={`M${-edge + .05} .11H${edge - .05}`} stroke="#d7a967" strokeWidth=".065" />
    <path d={`M${-edge + .12}-.12v.3m.16-.3v.3m${2 * edge - .56} -.3v.3m.16-.3v.3`}
      stroke="#e4b978" strokeWidth=".065" />
    {closed ? <g data-scout-door-leaf fill="#b28151" stroke="#573c30" strokeWidth=".038">
      <rect x={-edge + .2} y="-.13" height=".24" width={edge - .23} rx=".04" />
      <rect x=".03" y="-.13" height=".24" width={edge - .23} rx=".04" />
      <path d="M-.27-.015h.16m.22 0h.16" stroke="#fae1ac" strokeWidth=".055" />
    </g> : <path d="M-.17.32 0 .2l.17 .12" fill="none" stroke="#e4b978" strokeWidth=".05" />}
  </g>
}

export function ScoutModule({ entity }: { entity: Entity2D }) {
  switch (entity.appearance) {
    case 'hull': return <ScoutHull />
    case 'floor': return <ScoutDeck />
    case 'door': case 'threshold': return <BoardingDoor closed={entity.appearance === 'door'} />
    default: return null
  }
}
