import { useEffect, useRef, useState } from 'react'
import type { DrawItem, TopDownState } from '../topDownRuntime.ts'
import { SCOUT_ANCHORS } from '../scoutLayout.ts'
import { relativeAnchor, worldToView } from '../topDownFrames.ts'
import { svgSpriteTransform } from './svgWorld.ts'

export const diagnosticsEnabled = import.meta.env.DEV &&
  new URLSearchParams(window.location.search).has('renderDiagnostics')

export function RenderDiagnostics({ state, items, art, activeCues }: {
  state: TopDownState
  items: readonly DrawItem[]
  art: { count: number; decodedBytes: number }
  activeCues: () => readonly string[]
}) {
  const scale = state.camera.pixelsPerM
  const chart = state.frames.surface?.chart
  const anchors = Object.entries(SCOUT_ANCHORS).map(([name, anchor]) =>
    ({ name, position: worldToView(relativeAnchor(anchor, state.camera.anchor, state.frames), state.frames) }))
  return <g data-dev-render-diagnostics pointerEvents="none" aria-hidden="true" fontFamily="monospace">
    {items.map(item => {
      const footprint = item.entity.footprint
      return <g key={item.entity.id} transform={svgSpriteTransform(item, scale)}>
        {footprint && <rect x={-footprint.halfWidth} y={-footprint.halfHeight}
          width={footprint.halfWidth * 2} height={footprint.halfHeight * 2}
          fill="none" stroke="#ffcf72" strokeWidth=".03" />}
        <circle r=".06" fill="#94ffc9" />
        <text y="-.18" fontSize=".18" fill="#fff" stroke="#233438" strokeWidth=".035" paintOrder="stroke">
          {item.entity.layer}/{item.entity.id}
        </text>
      </g>
    })}
    {anchors.map(({ name, position: [x, y] }) =>
      <g key={name} transform={`translate(${x * scale} ${y * scale})`}>
        <path d="M-4 0h8m-4-4v8" stroke="#96cffc" /><text x="5" y="-4" fontSize="9" fill="#96cffc">{name}</text>
      </g>)}
    <title>SVG CPU · {art.count} images · {art.decodedBytes} decoded RGBA8 bytes · {chart?.id ?? 'authored terrain'} · audio {activeCues().join(', ') || 'idle'}</title>
  </g>
}

export function FrameDiagnostics({ backend, details, measure }: { backend: string; details?: string; measure?: () => string }) {
  const [summary, setSummary] = useState('')
  const latestMeasure = useRef(measure)
  latestMeasure.current = measure
  useEffect(() => {
    const samples: number[] = []
    let frame = 0, previous = 0
    const collect = (now: number) => {
      if (!document.hidden && previous > 0) samples.push(now - previous)
      previous = document.hidden ? 0 : now
      if (samples.length === 60) {
        const ordered = [...samples].sort((a, b) => a - b)
        const measured = latestMeasure.current?.()
        setSummary(`frame p50 ${ordered[29]!.toFixed(1)} ms · p95 ${ordered[56]!.toFixed(1)} ms${measured ? ` · ${measured}` : ''}`)
        samples.length = 0
      }
      frame = requestAnimationFrame(collect)
    }
    frame = requestAnimationFrame(collect)
    return () => cancelAnimationFrame(frame)
  }, [])
  return <aside data-dev-frame-diagnostics className="world-render-diagnostics" aria-label="Rendering diagnostics">
    {backend}{details && ` · ${details}`}{summary && ` · ${summary}`}
  </aside>
}
