import { useEffect, useRef, useState } from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import { Illustration } from './TopDownIllustration.tsx'
import type { Entity2D } from './topDownRuntime.ts'

type Bounds = readonly [number, number, number, number]
interface Sprite { readonly url: string; readonly bounds: Bounds; readonly bytes: number; readonly decodedBytes: number }
type SpriteRequest = { entity: Entity2D; selected?: boolean }
type SpriteOutcome = { key: string; sprite?: Sprite; failed?: boolean }

const spriteKey = ({ entity, selected }: SpriteRequest) =>
  `${selected ? 'selection' : entity.appearance}:${entity.footprint?.halfWidth ?? 0}:${entity.footprint?.halfHeight ?? 0}`
const essential = ({ entity, selected }: SpriteRequest) =>
  selected || !['ground', 'surface', 'stones'].includes(entity.appearance)

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
      bitmap.onerror = () => reject(new Error(`Unable to rasterize world sprite ${spriteKey({ entity, selected })}`))
      bitmap.src = objectUrl
    })
    const canvas = document.createElement('canvas')
    canvas.width = pixelsW; canvas.height = pixelsH
    const context = canvas.getContext('2d')
    if (!context) throw new Error('Canvas 2D is required to initialize world sprites')
    context.drawImage(bitmap, 0, 0)
    const png = await new Promise<Blob>((resolve, reject) => canvas.toBlob(blob => blob ? resolve(blob) : reject(new Error('Unable to encode world sprite')), 'image/png'))
    return { url: URL.createObjectURL(png), bounds, bytes: png.size, decodedBytes: pixelsW * pixelsH * 4 }
  } finally { URL.revokeObjectURL(objectUrl) }
}
export function spriteBounds({ entity, selected = false }: SpriteRequest): Bounds {
  const [x, y, width, height] = boundsFor(entity, selected)
  // Include backing-grid rounding and padding in the viewport test.
  const padding = 3 / 32
  return [x - padding, y - padding, width + 2 * padding, height + 2 * padding]
}

export function usePresentationVisibility() {
  const [visible, setVisible] = useState(() => !document.hidden)
  useEffect(() => {
    const update = () => setVisible(!document.hidden)
    document.addEventListener('visibilitychange', update)
    return () => document.removeEventListener('visibilitychange', update)
  }, [])
  return visible
}

export function useWorldSprites(requests: readonly SpriteRequest[]) {
  const unique = new Map(requests.map(request => [spriteKey(request), request]))
  const keys = [...unique.keys()].sort()
  const signature = keys.join('|')
  const cache = useRef(new Map<string, Promise<SpriteOutcome>>()).current
  const [outcomes, setOutcomes] = useState(new Map<string, SpriteOutcome>())
  const [attempt, retry] = useState(0)
  const lastAttempt = useRef(attempt)
  const lastRequests = useRef(unique)
  lastRequests.current = unique

  useEffect(() => {
    const requested = lastRequests.current
    const retrying = lastAttempt.current !== attempt
    lastAttempt.current = attempt
    let active = true
    const release = (entry: Promise<SpriteOutcome>) => {
      void entry.then(({ sprite }) => { if (sprite) URL.revokeObjectURL(sprite.url) })
    }
    for (const [key, entry] of cache) {
      if (!requested.has(key) || retrying) {
        cache.delete(key)
        release(entry)
      }
    }
    const load = (key: string) => {
      const previous = cache.get(key)
      if (previous) return previous
      const { entity, selected } = requested.get(key)!
      const entry = createSprite(entity, selected ?? false).then(
        sprite => ({ key, sprite }),
        () => ({ key, failed: true }),
      )
      cache.set(key, entry)
      return entry
    }
    const ordered = [...requested.keys()].sort()
    const required = ordered.filter(key => essential(requested.get(key)!))
    const decorative = ordered.filter(key => !essential(requested.get(key)!))
    setOutcomes(previous => new Map([...previous].filter(([key]) => requested.has(key) && !retrying)))
    void (async () => {
      const first = await Promise.all(required.map(load))
      if (!active) return
      setOutcomes(previous => new Map([...previous, ...first.map(entry => [entry.key, entry] as const)]))
      const second = await Promise.all(decorative.map(load))
      if (active) setOutcomes(new Map([...first, ...second].map(entry => [entry.key, entry])))
    })()
    return () => { active = false }
  }, [attempt, cache, signature])

  useEffect(() => () => {
    for (const entry of cache.values()) {
      void entry.then(({ sprite }) => { if (sprite) URL.revokeObjectURL(sprite.url) })
    }
    cache.clear()
  }, [cache])

  let encodedBytes = 0, decodedBytes = 0, count = 0
  let ready = true, error = false
  for (const [key, request] of unique) {
    const outcome = outcomes.get(key)
    if (!outcome) ready = false
    if (outcome?.failed && essential(request)) error = true
    if (outcome?.sprite) {
      encodedBytes += outcome.sprite.bytes
      decodedBytes += outcome.sprite.decodedBytes
      count++
    }
  }
  return {
    ready: ready && !error, error, retry: () => retry(value => value + 1),
    stats: { count, encodedBytes, decodedBytes },
    get: (request: SpriteRequest) => outcomes.get(spriteKey(request)),
  }
}

export function WorldSprite({ request, outcome }: { request: SpriteRequest; outcome?: SpriteOutcome }) {
  if (outcome?.failed) {
    if (request.selected) return null
    // The retained vector source supplies the same silhouette if its raster
    // cannot be decoded. The retry control is reserved for essential artwork.
    return <Illustration appearance={request.entity.appearance} footprint={request.entity.footprint} />
  }
  if (!outcome?.sprite) return null
  const { url, bounds: [x, y, width, height] } = outcome.sprite
  return <image href={url} x={x} y={y} width={width} height={height} />
}
