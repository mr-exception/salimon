import { point2, type Anchor } from './topDownFrames.ts'
import type { Entity2D } from './topDownRuntime.ts'

// Metres in ship-local space. These footprints are also the visible wall/prop
// geometry; no roof overlay or second interior scene exists.
const prop = (id: string, x: number, y: number, appearance: Entity2D['appearance'], halfWidth: number, halfHeight: number, label?: string, reachM = 1.9): Entity2D => ({
  id, anchor: { frame: 'ship:scout', point: point2(x, y) }, layer: 'props', appearance,
  footprint: { halfWidth, halfHeight },
  ...(label ? { interaction: { label, action: 'Inspect', reachM } } : {}),
})
export const SCOUT_ANCHORS: Readonly<Record<string, Anchor>> = {
  cockpit: { frame: 'ship:scout', point: point2(0, -5) },
  chair: { frame: 'ship:scout', point: point2(0, -3.95) },
  core: { frame: 'ship:scout', point: point2(1.9, 1.8) },
  fabricator: { frame: 'ship:scout', point: point2(-1.85, 3.9) },
  storage: { frame: 'ship:scout', point: point2(1.85, 4.25) },
  entry: { frame: 'ship:scout', point: point2(0, 5.5) },
}
export const STORAGE_CAPACITY = { massG: 50_000, volumeL: 50 } as const
export const SCOUT_ENTRY_HALF_WIDTH_M = 1.5
export function scoutEntities(): Entity2D[] {
  return [
    prop('scout:desk', 0, -5, 'desk', 1.5, .5, 'Cockpit desk'),
    prop('scout:chair', 0, -3.95, 'chair', .35, .35, 'Cockpit chair', 1),
    prop('scout:bed', -2.2, .5, 'bed', .45, 1.2),
    prop('scout:core', 1.9, 1.8, 'core', .4, .7, 'Core'),
    prop('scout:fabricator', -1.85, 3.9, 'fabricator', .6, .65, 'Fabricator'),
    prop('scout:storage', 1.85, 4.25, 'storage', .55, .7, 'Storage · 50 kg / 50 L'),
    prop('fixture:crate', 0, -.6, 'crate', .6, .5),
    ...[
      ['wall:port', -2.65, .5, .15, 5], ['wall:starboard', 2.65, .5, .15, 5],
      ['wall:nose-port', -1.85, -5.3, .15, .95], ['wall:nose-starboard', 1.85, -5.3, .15, .95],
      ['wall:shoulder-port', -2.2, -4.5, .45, .15], ['wall:shoulder-starboard', 2.2, -4.5, .45, .15],
      ['wall:cockpit-left', -1.65, -2.3, .85, .12], ['wall:cockpit-right', 1.65, -2.3, .85, .12],
      ['wall:nose', 0, -6.1, 1.7, .15],
      ['wall:aft-port', -2, 5.5, .5, .15], ['wall:aft-starboard', 2, 5.5, .5, .15],
      ['wall:partition-left', -1.9, 2.7, .6, .12], ['wall:partition-right', 1.9, 2.7, .6, .12],
    ].map(([id, x, y, hw, hh]) => prop(id as string, x as number, y as number, 'wall', hw as number, hh as number)),
    // Permanently open 2.6 m cargo passage and 3 m aft entry. Door jambs above
    // remain physical; the painted threshold has no hidden collision box.
    { id: 'scout:entry', anchor: SCOUT_ANCHORS.entry!, layer: 'interior', appearance: 'threshold' },
  ]
}
