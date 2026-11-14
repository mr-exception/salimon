import { useEffect, useState } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { Illustration } from './TopDownIllustration.tsx'
import type { Entity2D } from './topDownRuntime.ts'

type Bounds = readonly [number, number, number, number]
interface Sprite { readonly url: string; readonly bounds: Bounds }
interface CachedSprite { readonly promise: Promise<Sprite>; sprite?: Sprite }
const cache = new Map<string, CachedSprite>()
const key = (entity: Entity2D, selected = false) => `${selected ? 'selection' : entity.appearance}:${entity.footprint?.halfWidth ?? 0}:${entity.footprint?.halfHeight ?? 0}`

// Fixed authored-art bounds, in metres. Padding includes strokes; it never
// changes logical footprints. The terrain details end well inside its tile;
// the stage's identical flat ground colour continues outside the authored site.
function boundsFor(entity: Entity2D, selected: boolean): Bounds {
  if (selected) {
    const hw = (entity.footprint?.halfWidth ?? .4) + .3, hh = (entity.footprint?.halfHeight ?? .4) + .3
    return [-hw, -hh, hw * 2, hh * 2]
  }
  switch (entity.appearance) {
    case 'surface': case 'ground': return [-36, -24, 72, 48]
    case 'hull': return [-4, -7.3, 8, 14.1]
    case 'floor': return [-3, -6.8, 6, 13.2]
    case 'rock': return [-1.9, -1.6, 3.8, 3]
    case 'mineral': return [-1.4, -1.1, 2.8, 2.2]
    case 'flora': return [-1.4, -1.1, 2.8, 2]
    case 'stones': return [-.7, -.6, 1.4, 1.2]
    case 'player': return [-.6, -.55, 1.2, 1.1]
    case 'desk': return [-1.7, -1.3, 3.4, 1.9]
    case 'bed': return [-.6, -1.4, 1.2, 2.8]
    case 'core': return [-.6, -.9, 1.2, 1.8]
    case 'fabricator': return [-.8, -.85, 1.6, 1.7]
    case 'storage': return [-.75, -.9, 1.5, 1.8]
    case 'chair': return [-.55, -.55, 1.1, 1.1]
    case 'threshold': return [-1.65, -.3, 3.3, 1.2]
    case 'cue': return [-.5, -.5, 1, 1]
    case 'wall': case 'door': case 'crate': {
      const hw = entity.footprint!.halfWidth + .15, hh = entity.footprint!.halfHeight + .15
      return [-hw, -hh, hw * 2, hh * 2]
    }
  }
}
function loadSprite(entity: Entity2D, selected = false): CachedSprite {
  const id = key(entity, selected), existing = cache.get(id)
  if (existing) return existing
  const entry: CachedSprite = { promise: createSprite(entity, selected) }
  entry.promise.then(sprite => { entry.sprite = sprite }, () => {})
  cache.set(id, entry)
  return entry
}
async function createSprite(entity: Entity2D, selected: boolean): Promise<Sprite> {
  await document.fonts.ready
  const density = (entity.appearance === 'ground' || entity.appearance === 'surface') ? 32 : 64
  const [x, y, width, height] = boundsFor(entity, selected)
  // Align the asset itself to a fixed integer backing grid, independent of the
  // world camera, render cadence, view switching and screenshot readback.
  const left = Math.floor(x * density) - 2, top = Math.floor(y * density) - 2
  const pixelsW = Math.ceil((x + width) * density) - left + 2, pixelsH = Math.ceil((y + height) * density) - top + 2
  const bounds: Bounds = [left / density, top / density, pixelsW / density, pixelsH / density]
  const hw = entity.footprint?.halfWidth ?? .4, hh = entity.footprint?.halfHeight ?? .4
  const art = selected
    ? <rect x={-hw - .15} y={-hh - .15} width={hw * 2 + .3} height={hh * 2 + .3} rx=".1" fill="none" stroke="#bce9df" strokeWidth=".08" />
    : <Illustration appearance={entity.appearance} footprint={entity.footprint} />
  const svg = renderToStaticMarkup(<svg xmlns="http://www.w3.org/2000/svg" width={pixelsW} height={pixelsH} viewBox={bounds.join(' ')} shapeRendering="geometricPrecision" textRendering="geometricPrecision" style={{ fontFamily: 'system-ui,sans-serif' }}>{art}</svg>)
  const objectUrl = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }))
  try {
    const bitmap = new Image()
    await new Promise<void>((resolve, reject) => {
      bitmap.onload = () => resolve()
      bitmap.onerror = () => reject(new Error(`Unable to rasterize world sprite ${key(entity, selected)}`))
      bitmap.src = objectUrl
    })
    const canvas = document.createElement('canvas')
    canvas.width = pixelsW; canvas.height = pixelsH
    const context = canvas.getContext('2d')
    if (!context) throw new Error('Canvas 2D is required to initialize world sprites')
    context.drawImage(bitmap, 0, 0)
    const png = await new Promise<Blob>((resolve, reject) => canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error('Unable to encode world sprite')), 'image/png'))
    // Blob references keep immutable assets out of per-frame DOM snapshots.
    // This bounded cache lives as long as the persistent world/document.
    return { url: URL.createObjectURL(png), bounds }
  } finally { URL.revokeObjectURL(objectUrl) }
}
export function useWorldSprites(entities: readonly Entity2D[]) {
  const signature = entities.map(e => key(e) + (e.interaction ? `/${key(e, true)}` : '')).sort().join('|')
  const [ready, setReady] = useState(false), [error, setError] = useState<Error | null>(null)
  useEffect(() => {
    let alive = true
    setReady(false)
    const loads = entities.flatMap(e => [loadSprite(e).promise, ...(e.interaction ? [loadSprite(e, true).promise] : [])])
    Promise.all(loads).then(() => { if (alive) setReady(true) }, reason => { if (alive) setError(reason instanceof Error ? reason : new Error(String(reason))) })
    return () => { alive = false }
    // Entity positions/frames do not invalidate authored illustration assets.
  }, [signature])
  if (error) throw error
  return { ready }
}
export function WorldSprite({ entity, selected = false }: { entity: Entity2D; selected?: boolean }) {
  const sprite = cache.get(key(entity, selected))?.sprite
  if (!sprite) return null
  const [x, y, width, height] = sprite.bounds
  return <image href={sprite.url} x={x} y={y} width={width} height={height} />
}
