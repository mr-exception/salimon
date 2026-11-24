import { translate } from './coordinates.ts'
import { relativeAnchor, rotate2, viewToWorld, type Anchor, type Point2 } from './topDownFrames.ts'
import type { TopDownState } from './topDownRuntime.ts'

export interface WorldCamera { readonly anchor: Anchor; readonly pixelsPerM: number; readonly panned?: boolean }
export const CAMERA_MIN_SCALE = 14, CAMERA_MAX_SCALE = 64, CAMERA_PAN_RADIUS_M = 40

/** Presentation-only operations. SVG screen CTM converts CSS pixels first. */
export function zoomWorldCamera(state: TopDownState, deltaPixels: number): TopDownState {
  if (!Number.isFinite(deltaPixels)) return state
  const pixelsPerM = Math.max(CAMERA_MIN_SCALE, Math.min(CAMERA_MAX_SCALE, state.camera.pixelsPerM * Math.exp(-Math.max(-1000, Math.min(1000, deltaPixels)) * .0015)))
  return { ...state, camera: { ...state.camera, pixelsPerM } }
}
export function followWorldCamera(state: TopDownState): TopDownState {
  return state.camera.panned ? { ...state, camera: { ...state.camera, panned: false } } : state
}
export function panWorldCamera(state: TopDownState, svgDelta: Point2): TopDownState {
  if (!svgDelta.every(Number.isFinite)) return state
  const player = state.entities.find(e => e.id === 'person:player')!.anchor
  const delta = viewToWorld([-svgDelta[0] / state.camera.pixelsPerM, -svgDelta[1] / state.camera.pixelsPerM], state.frames)
  const offset = relativeAnchor(state.camera.anchor, player, state.frames)
  const requested: Point2 = [offset[0] + delta[0], offset[1] + delta[1]]
  const factor = Math.min(1, CAMERA_PAN_RADIUS_M / Math.max(1e-12, Math.hypot(...requested)))
  const local = rotate2([requested[0] * factor, requested[1] * factor], player.frame === 'ship:scout' ? -state.frames.ship.angle : 0)
  return { ...state, camera: { ...state.camera, panned: true, anchor: { ...player, point: translate(player.point, [...local, 0]) } } }
}

/** One captured pointer; all terminal DOM events use the same cancellation. */
export class CameraDrag {
  private pointer: { id: number; point: Point2 } | null = null
  get id() { return this.pointer?.id ?? null }
  begin(id: number, point: Point2) { if (this.pointer) return false; this.pointer = { id, point }; return true }
  move(id: number, point: Point2): Point2 | null {
    if (this.pointer?.id !== id) return null
    const delta: Point2 = [point[0] - this.pointer.point[0], point[1] - this.pointer.point[1]]
    this.pointer = { id, point }; return delta
  }
  cancel(id = this.id) { if (this.pointer?.id !== id) return null; const captured = this.id; this.pointer = null; return captured }
}
