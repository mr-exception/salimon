/** Metre-scale physical surface queries. No sprite, camera or renderer imports. */
export interface SurfaceObstacle { readonly id: string; readonly x: number; readonly y: number; readonly halfWidth: number; readonly halfHeight: number }
export const WALK_RADIUS_M = .46
export interface SurfaceProbe {
  readonly x: number; readonly y: number
  readonly radiusM?: number
  readonly motion?: readonly [number, number]
  readonly landingYaw?: number
}
function rotate(x: number, y: number, angle: number): readonly [number, number] {
  return [Math.cos(angle) * x - Math.sin(angle) * y, Math.sin(angle) * x + Math.cos(angle) * y]
}
// Swept point versus expanded box: tangent/away motion remains legal and long
// steps cannot tunnel. Shared by existing walking and canonical surface sampling.
function hitFraction(origin: readonly [number, number], delta: readonly [number, number], w: number, h: number): number {
  let enter = -Infinity, leave = Infinity
  for (let axis = 0; axis < 2; axis++) {
    const extent = axis === 0 ? w : h
    if (Math.abs(delta[axis]!) < 1e-12) {
      if (Math.abs(origin[axis]!) >= extent - 1e-10) return 1
    } else {
      const a = (-extent - origin[axis]!) / delta[axis]!, b = (extent - origin[axis]!) / delta[axis]!
      enter = Math.max(enter, Math.min(a, b)); leave = Math.min(leave, Math.max(a, b))
    }
  }
  return enter < leave && leave > 1e-10 && enter >= -1e-10 && enter < 1 ? Math.max(0, enter - 1e-9) : 1
}
/** The current approved terrain is flat, solid and undeformed. Obstacles are
 * physical authored boxes or validated save deltas, never decorative pixels. */
export function samplePhysicalSurface(probe: SurfaceProbe, obstacles: readonly SurfaceObstacle[]) {
  const radius = probe.radiusM ?? 0, motion = probe.motion ?? [0, 0], yaw = probe.landingYaw ?? 0
  if (![probe.x, probe.y, radius, ...motion, yaw].every(Number.isFinite) || radius < 0 || motion.length !== 2) throw Error('Invalid physical surface probe')
  let collision = false, moveFraction = 1, landingClear = true
  const c = Math.abs(Math.cos(yaw)), s = Math.abs(Math.sin(yaw))
  for (const object of obstacles) {
    const { halfWidth: w, halfHeight: h } = object
    if (![object.x, object.y, w, h].every(Number.isFinite) || w <= 0 || h <= 0) throw Error('Invalid physical surface obstacle')
    const dx = object.x - probe.x, dy = object.y - probe.y
    collision ||= Math.abs(dx) < w + radius && Math.abs(dy) < h + radius
    moveFraction = Math.min(moveFraction, hitFraction([-dx, -dy], motion, w + radius, h + radius))
    if (probe.landingYaw !== undefined) {
      const [lx, ly] = rotate(dx, dy, -yaw)
      const physical = Math.hypot(Math.max(0, Math.abs(dx) - w), Math.max(0, Math.abs(dy) - h)) < 5
      const overlap = (cx: number, cy: number, hw: number, hh: number) => {
        const [wx, wy] = rotate(cx, cy, yaw)
        return Math.abs(lx - cx) < hw + c*w + s*h && Math.abs(ly - cy) < hh + s*w + c*h
          && Math.abs(dx - wx) < w + c*hw + s*hh && Math.abs(dy - wy) < h + s*hw + c*hh
      }
      if (physical || overlap(0, 0, 4.2, 7.5) || overlap(0, 7, 1.5, 1.5)) landingClear = false
    }
  }
  return { heightM: 0, slopeRadians: 0, solid: true, collision, moveFraction, landingClear: probe.landingYaw === undefined ? null : landingClear } as const
}
