import { Euler, Quaternion, Vector3 } from 'three'
import { relativeM, type Position } from './coordinates.ts'
import type { ShipState } from './simulation.ts'

export const DEFAULT_OBSERVATION_FOV = 75
export const MIN_OBSERVATION_FOV = 0.1
export interface ObservationSnapshot {
  readonly quaternion: readonly [number, number, number, number]
  readonly fov: number
}
export type ObservationCommand =
  | { type: 'look'; right: number; up: number }
  | { type: 'zoom'; pixels: number }
  | { type: 'reset'; heading: Pick<ShipState, 'yaw' | 'pitch'> }

/** Session-local view authority. Keep this owner outside the mounted R3F scene.
 * Angles are increments in the current screen basis, never Euler accumulators
 * or a world-up lookAt reconstruction. Crossing either pole preserves the
 * transported up/right frame; no implicit roll correction or singularity.
 */
export class ObservationCameraState {
  private orientation = new Quaternion()
  private verticalFov = DEFAULT_OBSERVATION_FOV

  constructor(heading: Pick<ShipState, 'yaw' | 'pitch'>) { this.reset(heading) }

  snapshot(): ObservationSnapshot {
    return { quaternion: this.orientation.toArray(), fov: this.verticalFov }
  }

  restore(snapshot: ObservationSnapshot): void {
    if (snapshot.quaternion.length !== 4 || !snapshot.quaternion.every(Number.isFinite) ||
        !Number.isFinite(snapshot.fov)) throw new RangeError('Invalid observation snapshot')
    const norm = Math.hypot(...snapshot.quaternion)
    if (norm < 1e-12 || !Number.isFinite(norm)) throw new RangeError('Invalid observation quaternion')
    this.orientation.fromArray(snapshot.quaternion.map(value => value / norm))
    this.verticalFov = Math.max(MIN_OBSERVATION_FOV, Math.min(DEFAULT_OBSERVATION_FOV, snapshot.fov))
  }

  reset(heading: Pick<ShipState, 'yaw' | 'pitch'>): void {
    if (![heading.yaw, heading.pitch].every(Number.isFinite)) throw new RangeError('Invalid heading')
    this.orientation.setFromEuler(new Euler(heading.pitch, heading.yaw, 0, 'YXZ'))
    this.verticalFov = DEFAULT_OBSERVATION_FOV
  }

  /** Screen fractions (right/up), scaled by optical FOV at every command. */
  look(right: number, up: number): void {
    if (![right, up].every(Number.isFinite)) return
    const radians = this.verticalFov * Math.PI / 180
    // One rotation vector in the image plane: no optical-axis roll component.
    const axis = new Vector3(up, -right, 0)
    const magnitude = axis.length()
    if (!Number.isFinite(magnitude) || magnitude === 0) return
    this.orientation.multiply(new Quaternion().setFromAxisAngle(axis.divideScalar(magnitude),
      magnitude * radians)).normalize()
  }

  zoom(pixels: number): void {
    if (!Number.isFinite(pixels)) return
    this.verticalFov = Math.max(MIN_OBSERVATION_FOV, Math.min(DEFAULT_OBSERVATION_FOV,
      this.verticalFov * Math.exp(Math.max(-100, Math.min(100, pixels * 0.001)))))
  }

  command(command: ObservationCommand): void {
    switch (command.type) {
      case 'look': this.look(command.right, command.up); break
      case 'zoom': this.zoom(command.pixels); break
      case 'reset': this.reset(command.heading); break
    }
  }
}

/** Physical origin remains split. GPU origin is always zero; render bodies
 * with relativeM(body, frame.origin), never global cell -> Number conversion.
 * Reading a ship snapshot cannot alter either ship or inertial orientation.
 */
export function observationFrame(view: ObservationCameraState, ship: Pick<ShipState, 'position'>) {
  return { origin: ship.position, ...view.snapshot() }
}
export function observationRelativePosition(target: Position, ship: Pick<ShipState, 'position'>) {
  return relativeM(target, ship.position)
}
