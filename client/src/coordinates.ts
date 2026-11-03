import { type Vec3 } from './math.ts'
export const LIGHT_SECOND_M = 299_792_458
// Integer cells never pass through a global Number. Offsets retain sub-mm precision.
export const CELL_SIZE_M = 2 ** 40
export interface Position { readonly cell: readonly [bigint,bigint,bigint]; readonly offsetM: Vec3 }
export const ORIGIN: Position = {cell:[0n,0n,0n],offsetM:[0,0,0]}
export function translate(position: Position, displacementM: Vec3): Position {
  const cell = [...position.cell] as [bigint,bigint,bigint]
  const offset = displacementM.map((d,i) => {
    const value = position.offsetM[i]+d
    if (!Number.isFinite(value)) throw new RangeError('Non-finite position displacement')
    const carry = Math.floor((value+CELL_SIZE_M/2)/CELL_SIZE_M)
    if (!Number.isSafeInteger(carry)) throw new RangeError('Translate in bounded local segments')
    cell[i] += BigInt(carry)
    return value-carry*CELL_SIZE_M
  }) as [number,number,number]
  return {cell,offsetM:offset}
}
// Only relative cell differences are narrowed; nearby objects retain local precision
// even when both cell indices are far beyond Number.MAX_SAFE_INTEGER.
export function relativeM(target: Position, origin: Position): Vec3 {
  return target.cell.map((c,i)=>Number(c-origin.cell[i])*CELL_SIZE_M+(target.offsetM[i]-origin.offsetM[i])) as [number,number,number]
}
export const radarPosition = (target: Position, ship: Position): Vec3 => relativeM(target,ship).map(v=>v/LIGHT_SECOND_M) as [number,number,number]
