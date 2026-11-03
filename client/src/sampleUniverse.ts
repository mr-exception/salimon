import { LIGHT_SECOND_M, ORIGIN, translate, type Position } from './coordinates.ts'
import { type Vec3 } from './math.ts'
export { add, sub, scale, length, normal, type Vec3 } from './math.ts'
export type BodyKind = 'star' | 'planet' | 'moon' | 'asteroid'
export interface Body {
  readonly id: string
  readonly name: string
  readonly kind: BodyKind
  readonly position: Position
  readonly radiusKm: number
}
export const SCANNER_RADIUS_LS = 100_000
export const SCAN_INTERVAL_SECONDS = 5
const samples: readonly {id:string;name:string;kind:BodyKind;positionLs:Vec3;radiusKm:number}[] = [
  { id: 'nacre', name: 'Nacre', kind: 'star', positionLs: [-530, 90, -210], radiusKm: 510_000 },
  { id: 'vesper', name: 'Vesper', kind: 'planet', positionLs: [180, -18, -260], radiusKm: 6_200 },
  { id: 'vesper-moon', name: 'Vesper Moon', kind: 'moon', positionLs: [197, -8, -288], radiusKm: 1_400 },
  { id: 'caligo', name: 'Caligo', kind: 'planet', positionLs: [790, 160, 480], radiusKm: 9_000 },
  { id: 'relay', name: 'Uncharted asteroid', kind: 'asteroid', positionLs: [-125, -55, 115], radiusKm: 35 }
]

export const SAMPLE_BODIES: readonly Body[] = samples.map(({positionLs,...body})=>({...body,position:translate(ORIGIN,positionLs.map(v=>v*LIGHT_SECOND_M) as [number,number,number])}))
