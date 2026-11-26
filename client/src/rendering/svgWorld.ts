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
