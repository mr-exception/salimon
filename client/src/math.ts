export type Vec3 = readonly [number, number, number]
export const add = (a: Vec3, b: Vec3): Vec3 => [a[0]+b[0], a[1]+b[1], a[2]+b[2]]
export const sub = (a: Vec3, b: Vec3): Vec3 => [a[0]-b[0], a[1]-b[1], a[2]-b[2]]
export const scale = (a: Vec3, k: number): Vec3 => [a[0]*k, a[1]*k, a[2]*k]
export const length = (a: Vec3) => Math.hypot(...a)
export const normal = (a: Vec3): Vec3 => { const l = length(a); return l > 0 ? scale(a, 1/l) : [0,0,-1] }
export const dot = (a: Vec3, b: Vec3) => a[0]*b[0]+a[1]*b[1]+a[2]*b[2]
export const cross = (a: Vec3, b: Vec3): Vec3 => [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]

/** Rodrigues rotation in a fixed angular-velocity frame. */
export function rotateVector(v: Vec3, omega: Vec3, seconds: number): Vec3 {
  const speed = length(omega)
  if (speed === 0) return v
  const axis = scale(omega, 1 / speed), angle = speed * seconds
  return add(add(scale(v, Math.cos(angle)), scale(cross(axis, v), Math.sin(angle))), scale(axis, dot(axis, v) * (1 - Math.cos(angle))))
}
