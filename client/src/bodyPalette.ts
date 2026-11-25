import type { Body } from './sampleUniverse.ts'
import type { Vec3 } from './math.ts'

/** Shared display-space pigments. These are art swatches, not new geology. */
export const SURFACE_PIGMENTS = {
  lowland: [179, 160, 132], highland: [133, 131, 118], impact: [160, 140, 119],
  fracture: [103, 108, 105], ice: [187, 211, 216],
} as const
const channel = (v: number) => {
  const c = v / 255
  return c <= .04045 ? c / 12.92 : ((c + .055) / 1.055) ** 2.4
}
const linear = (rgb: Vec3): Vec3 => [channel(rgb[0]), channel(rgb[1]), channel(rgb[2])]
export const OPTICAL_ICE = linear(SURFACE_PIGMENTS.ice)

/** Body.kind is the only material classification in the current catalog.
 * Do not infer ice, gas, atmosphere or oceans from names, radii or IDs. */
export function bodyPalette(kind: Body['kind']): { color: Vec3; accent: Vec3 } {
  switch (kind) {
    case 'planet': return { color: linear(SURFACE_PIGMENTS.lowland), accent: linear(SURFACE_PIGMENTS.highland) }
    case 'moon': return { color: linear(SURFACE_PIGMENTS.highland), accent: linear(SURFACE_PIGMENTS.impact) }
    case 'asteroid': return { color: linear(SURFACE_PIGMENTS.fracture), accent: linear(SURFACE_PIGMENTS.impact) }
    case 'star': return { color: [1, .66, .26], accent: [1, .9, .6] }
  }
}
