import { LAYERS, type DrawItem, type Layer } from '../topDownRuntime.ts'

type LayerContents = {
  readonly id: Layer
  readonly beforeTerrain: DrawItem[]
  readonly sprites: DrawItem[]
}

/** Compose the retained SVG passes from the domain's ordered projection. */
export function svgLayers(orderedItems: readonly DrawItem[]) {
  const layers = new Map<Layer, LayerContents>(
    LAYERS.map(id => [id, { id, beforeTerrain: [], sprites: [] }]),
  )
  for (const item of orderedItems) {
    const layer = layers.get(item.entity.layer)!
    const appearance = item.entity.appearance
    const pass = appearance === 'ground' || appearance === 'surface' ? layer.beforeTerrain : layer.sprites
    pass.push(item)
  }
  return [...layers.values()]
}

export function svgSpriteTransform({ positionM: [x, y], angle }: DrawItem, scale: number) {
  return `translate(${x * scale} ${y * scale}) rotate(${angle * 180 / Math.PI}) scale(${scale})`
}

/** Conservative rectangle for rotated authored artwork, in camera metres. */
export function svgArtIntersects(
  { positionM: [x, y], angle }: DrawItem,
  [left, top, width, height]: readonly [number, number, number, number],
  viewWidthM: number, viewHeightM: number, marginM = 0,
) {
  const cos = Math.cos(angle), sin = Math.sin(angle)
  const localX = left + width / 2, localY = top + height / 2
  const cx = x + localX * cos - localY * sin
  const cy = y + localX * sin + localY * cos
  const hx = (width * Math.abs(cos) + height * Math.abs(sin)) / 2
  const hy = (width * Math.abs(sin) + height * Math.abs(cos)) / 2
  return Math.abs(cx) <= viewWidthM / 2 + hx + marginM &&
    Math.abs(cy) <= viewHeightM / 2 + hy + marginM
}
