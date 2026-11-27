import type { Entity2D } from '../topDownRuntime.ts'
import type { Point2 } from '../topDownFrames.ts'

type Wall = Entity2D & { footprint: NonNullable<Entity2D['footprint']> }
const isWall = (entity: Entity2D): entity is Wall =>
  entity.appearance === 'wall' && entity.anchor.frame === 'ship:scout' && !!entity.footprint
const joinToleranceM = .065

export type WallJoins = readonly [start: boolean, end: boolean]
export function scoutWallJoins(entities: readonly Entity2D[]) {
  const walls = entities.filter(isWall)
  const joins = new Map<string, WallJoins>()
  for (const wall of walls) {
    const [x, y] = wall.anchor.point.offsetM
    const { halfWidth, halfHeight } = wall.footprint
    const horizontal = halfWidth >= halfHeight
    const attached = (sign: number) => {
      const endX = horizontal ? x + sign * halfWidth : x
      const endY = horizontal ? y : y + sign * halfHeight
      return walls.some(other => {
        if (other.id === wall.id) return false
        const [ox, oy] = other.anchor.point.offsetM
        return Math.abs(endX - ox) <= other.footprint.halfWidth + (horizontal ? 0 : halfWidth) + joinToleranceM &&
          Math.abs(endY - oy) <= other.footprint.halfHeight + (horizontal ? halfHeight : 0) + joinToleranceM
      })
    }
    joins.set(wall.id, [attached(-1), attached(1)])
  }
  return joins
}

export function isScoutModule(entity: Entity2D) {
  return entity.anchor.frame === 'ship:scout' &&
    ['hull', 'floor', 'wall', 'door', 'threshold'].includes(entity.appearance)
}

/** Estimate overlap of the player's visible torso with foreground artwork. */
export function scoutArtObstructs(entity: Entity2D, playerRelativeM: Point2) {
  if (entity.anchor.frame !== 'ship:scout' || !entity.footprint ||
      !['wall', 'bed', 'desk', 'core', 'storage', 'fabricator', 'crate'].includes(entity.appearance)) return false
  const [x, y] = playerRelativeM
  const { halfWidth, halfHeight } = entity.footprint
  return y < 0 && y >= -halfHeight - .47 && Math.abs(x) <= halfWidth + .21
}
