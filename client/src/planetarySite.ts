import { point2, type Anchor } from './topDownFrames.ts'
import type { Entity2D } from './topDownRuntime.ts'

// Authored fictional living-world sample, NOT astronomy or global generation.
// Stable v1 planet-fixed anchors are independent of camera, ship and visit time.
export const STARTING_SITE = {
  id: 'site:illustrated-landing:v1', version: 1,
  spawn: { frame: 'planet:fixture', point: point2(-6, 3) } satisfies Anchor,
  shipOrigin: point2(2, -1), shipAngle: -.12,
} as const
export interface SourceIdentity {
  readonly siteId: typeof STARTING_SITE.id
  readonly kind: 'mineral' | 'biological'
  readonly archetype: 'banded-outcrop' | 'plate-colony'
}
const surface = (id: string, x: number, y: number, appearance: Entity2D['appearance'], halfWidth: number, halfHeight: number): Entity2D => ({
  id, anchor: { frame: 'planet:fixture', point: point2(x, y) }, layer: 'terrain', appearance,
  footprint: { halfWidth, halfHeight },
})
export function planetaryEntities(): Entity2D[] {
  return [
    { id: 'terrain:site', anchor: { frame: 'planet:fixture', point: point2(0, 0) }, layer: 'terrain', appearance: 'ground' },
    surface('rock:west', -10, -3, 'rock', 1.7, 1.4),
    surface('rock:east', 11, 5, 'rock', 1.7, 1.4),
    surface('rock:north', -3, -10, 'rock', 1.7, 1.4),
    surface('rock:south', -12, 10, 'rock', 1.7, 1.4),
    surface('rock:rim', 13, -7, 'rock', 1.7, 1.4),
    surface('stones:west', -14, 4, 'stones', .5, .4),
    surface('stones:north', -7, -6, 'stones', .5, .4),
    surface('stones:east', 8, -8, 'stones', .5, .4),
    surface('stones:south', 7, 10, 'stones', .5, .4),
    { ...surface('source:landing:mineral:01', -9, 0, 'mineral', 1.2, .9),
      source: { siteId: STARTING_SITE.id, kind: 'mineral', archetype: 'banded-outcrop' },
      interaction: { label: 'Banded mineral outcrop', action: 'Inspect', reachM: 2.3 } },
    { ...surface('source:landing:biology:01', -5, 6, 'flora', .9, .85),
      source: { siteId: STARTING_SITE.id, kind: 'biological', archetype: 'plate-colony' },
      interaction: { label: 'Alien plate colony', action: 'Inspect', reachM: 1.9 } },
    surface('flora:west', -14, -1, 'flora', .9, .85),
    surface('flora:south', -9, 9, 'flora', .9, .85),
    surface('flora:east', 10, -3, 'flora', .9, .85),
  ]
}
