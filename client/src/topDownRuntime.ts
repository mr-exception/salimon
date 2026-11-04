import { translate } from './coordinates.ts'
import { point2, relativeAnchor, rotate2, type Anchor, type Point2, type WorldFrames } from './topDownFrames.ts'

export const LAYERS = ['terrain', 'hull', 'interior', 'people', 'props', 'cues'] as const
export type Layer = typeof LAYERS[number]
export type Appearance = 'ground' | 'rock' | 'hull' | 'floor' | 'player' | 'desk' | 'bed' | 'core' | 'cue'
export interface Entity2D { readonly id: string; readonly anchor: Anchor; readonly layer: Layer; readonly appearance: Appearance }
export interface TopDownState {
  readonly tick: number
  readonly frames: WorldFrames
  readonly entities: readonly Entity2D[]
  readonly camera: { readonly anchor: Anchor; readonly pixelsPerM: number }
}
export const STEP_2D = 1 / 60
export const PLAYER_SPEED_MPS = 2.4
export const PLAYER_ID = 'person:player'
const entity = (id: string, frame: Anchor['frame'], x: number, y: number, layer: Layer, appearance: Appearance): Entity2D =>
  ({ id, anchor: { frame, point: point2(x, y) }, layer, appearance })

export function createTopDownState(): TopDownState {
  const entities = [
    entity('terrain:site', 'planet:fixture', 0, 0, 'terrain', 'ground'),
    entity('rock:west', 'planet:fixture', -10, -3, 'terrain', 'rock'),
    entity('rock:east', 'planet:fixture', 11, 5, 'terrain', 'rock'),
    entity('ship:scout', 'ship:scout', 0, 0, 'hull', 'hull'),
    entity('scout:floor', 'ship:scout', 0, 0, 'interior', 'floor'),
    entity(PLAYER_ID, 'ship:scout', 0, 2, 'people', 'player'),
    entity('scout:desk', 'ship:scout', 0, -5, 'props', 'desk'),
    entity('scout:bed', 'ship:scout', -2.2, .5, 'props', 'bed'),
    entity('scout:core', 'ship:scout', 2.3, 3.5, 'props', 'core'),
    entity('scout:desk-cue', 'ship:scout', 0, -3.5, 'cues', 'cue'),
  ]
  return {
    tick: 0,
    frames: {
      planet: { id: 'planet:fixture', universeOrigin: { cell: [10n ** 24n, -(10n ** 24n), 0n], offsetM: [0, 0, 0] }, angle: .2 },
      ship: { id: 'ship:scout', parent: 'planet:fixture', origin: point2(2, -1), angle: -.12 },
    },
    entities,
    camera: { anchor: entities.find(e => e.id === PLAYER_ID)!.anchor, pixelsPerM: 32 },
  }
}
// One scene and one fixed update boundary. The player stays in its current frame;
// frame transfer, collision, furniture interaction and navigation seating follow later.
export function stepTopDown(state: TopDownState, direction: Point2): TopDownState {
  const [x, y] = direction
  if (![x, y].every(Number.isFinite)) throw new RangeError('Expected finite movement')
  const length = Math.hypot(x, y), scale = length > 0 ? PLAYER_SPEED_MPS * STEP_2D / Math.max(1, length) : 0
  const entities = state.entities.map(e => {
    if (e.id !== PLAYER_ID || length === 0) return e
    const localDirection = e.anchor.frame === state.frames.ship.id ? rotate2([x, y], -state.frames.ship.angle) : direction
    const moved = translate(e.anchor.point, [localDirection[0] * scale, localDirection[1] * scale, 0])
    // Bounded interior walk fixture: no implied cross-frame passage through hull.
    return { ...e, anchor: { ...e.anchor, point: point2(Math.max(-1.1, Math.min(1.1, moved.offsetM[0])), Math.max(-2.6, Math.min(4.4, moved.offsetM[1]))) } }
  })
  return { ...state, tick: state.tick + 1, entities, camera: { ...state.camera, anchor: entities.find(e => e.id === PLAYER_ID)!.anchor } }
}
export interface DrawItem { readonly entity: Entity2D; readonly positionM: Point2; readonly angle: number }
export function drawList(state: TopDownState): DrawItem[] {
  return [...state.entities].sort((a, b) => LAYERS.indexOf(a.layer) - LAYERS.indexOf(b.layer)
    || Number(b.appearance === 'ground') - Number(a.appearance === 'ground')
    || (a.id < b.id ? -1 : a.id > b.id ? 1 : 0))
    .map(entity => ({ entity, positionM: relativeAnchor(entity.anchor, state.camera.anchor, state.frames), angle: entity.anchor.frame === state.frames.ship.id ? state.frames.ship.angle : 0 }))
}
export class TopDownClock {
  private remainder = 0
  advance(state: TopDownState, seconds: number, direction: Point2): TopDownState {
    if (!Number.isFinite(seconds) || seconds < 0) throw new RangeError('Expected nonnegative finite time')
    this.remainder += Math.min(seconds, .25)
    while (this.remainder + 1e-12 >= STEP_2D) {
      state = stepTopDown(state, direction)
      this.remainder = Math.max(0, this.remainder - STEP_2D)
    }
    return state
  }
  reset() { this.remainder = 0 }
}
