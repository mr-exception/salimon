import { AUTHORED_SURFACE } from './authoredSurface.ts'
import { point2, type Anchor } from './topDownFrames.ts'
import type { Entity2D } from './topDownRuntime.ts'

// Authored fictional living-world sample, NOT astronomy or global generation.
// Stable v1 planet-fixed anchors are independent of camera, ship and visit time.
export const STARTING_SITE = {
  id: AUTHORED_SURFACE.siteId, version: 1,
  spawn: { frame: 'planet:fixture', point: point2(...AUTHORED_SURFACE.spawn) } satisfies Anchor,
  shipOrigin: point2(AUTHORED_SURFACE.ship[0], AUTHORED_SURFACE.ship[1]), shipAngle: AUTHORED_SURFACE.ship[2],
} as const
export interface SourceIdentity {
  readonly siteId: typeof STARTING_SITE.id
  readonly kind: 'mineral' | 'biological'
  readonly archetype: 'banded-outcrop' | 'plate-colony'
}
const surface = (id: string, appearance: Entity2D['appearance']): Entity2D => {
  const physical = AUTHORED_SURFACE.objects.find(o => o[0] === id)
  if (!physical) throw Error('Missing authored physical surface object')
  const [, x, y, halfWidth, halfHeight] = physical
  return { id, anchor: { frame: 'planet:fixture', point: point2(x, y) }, layer: 'terrain', appearance, footprint: { halfWidth, halfHeight } }
}
export function planetaryEntities(): Entity2D[] {
  return [
    { id: 'terrain:site', anchor: { frame: 'planet:fixture', point: point2(0, 0) }, layer: 'terrain', appearance: 'ground' },
    surface('rock:west', 'rock'),
    surface('rock:east', 'rock'),
    surface('rock:north', 'rock'),
    surface('rock:south', 'rock'),
    surface('rock:rim', 'rock'),
    surface('stones:west', 'stones'),
    surface('stones:north', 'stones'),
    surface('stones:east', 'stones'),
    surface('stones:south', 'stones'),
    { ...surface('source:landing:mineral:01', 'mineral'),
      source: { siteId: STARTING_SITE.id, kind: 'mineral', archetype: 'banded-outcrop' },
      interaction: { label: 'Banded mineral outcrop', action: 'Inspect', reachM: 2.3 } },
    { ...surface('source:landing:biology:01', 'flora'),
      source: { siteId: STARTING_SITE.id, kind: 'biological', archetype: 'plate-colony' },
      interaction: { label: 'Alien plate colony', action: 'Inspect', reachM: 1.9 } },
    surface('flora:west', 'flora'),
    surface('flora:south', 'flora'),
    surface('flora:east', 'flora'),
  ]
}
