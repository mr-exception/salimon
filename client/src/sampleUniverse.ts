export type Vec3 = readonly [number, number, number]
export type BodyKind = 'star' | 'planet' | 'moon' | 'asteroid'
export interface Body {
  readonly id: string
  readonly name: string
  readonly kind: BodyKind
  readonly positionLs: Vec3
  readonly radiusKm: number
}
export const SCANNER_RADIUS_LS = 100_000
export const SCAN_INTERVAL_SECONDS = 5
export const SAMPLE_BODIES: readonly Body[] = [
  { id: 'nacre', name: 'Nacre', kind: 'star', positionLs: [-530, 90, -210], radiusKm: 510_000 },
  { id: 'vesper', name: 'Vesper', kind: 'planet', positionLs: [180, -18, -260], radiusKm: 6_200 },
  { id: 'vesper-moon', name: 'Vesper Moon', kind: 'moon', positionLs: [197, -8, -288], radiusKm: 1_400 },
  { id: 'caligo', name: 'Caligo', kind: 'planet', positionLs: [790, 160, 480], radiusKm: 9_000 },
  { id: 'relay', name: 'Uncharted asteroid', kind: 'asteroid', positionLs: [-125, -55, 115], radiusKm: 35 }
]
export const add = (a: Vec3, b: Vec3): Vec3 => [a[0]+b[0],a[1]+b[1],a[2]+b[2]]
export const sub = (a: Vec3, b: Vec3): Vec3 => [a[0]-b[0],a[1]-b[1],a[2]-b[2]]
export const scale = (a: Vec3, k: number): Vec3 => [a[0]*k,a[1]*k,a[2]*k]
export const length = (a: Vec3) => Math.hypot(...a)
export const normal = (a: Vec3): Vec3 => { const l = length(a); return l>1e-10 ? scale(a,1/l) : [0,0,-1] }
