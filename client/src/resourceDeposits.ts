import { depositNeighborhood, isDepositId, regionDeposits, type PlanetDeposit } from './planetDeposits.ts'
import { surfaceRegion, surfaceToLocal, localToSurface } from './surfaceCoordinates.ts'
import { ORIGIN, relativeM } from './coordinates.ts'
import { point2, relativeAnchor, rotate2 } from './topDownFrames.ts'
import { PLAYER_ID, type TopDownState } from './topDownRuntime.ts'

/** Keep unworked deposit baselines disposable and exclude the current ship footprint. */
export function synchronizeDeposits(world: TopDownState, cachedKey: string, deposits: readonly PlanetDeposit[]) {
    const binding = world.scoutLocation.binding
    const player = world.entities.find(e => e.id === PLAYER_ID)!
    const landing = binding ? surfaceRegion(world.scoutLocation.surface, world.scoutLocation.surface.definition) : null
    const chart = player.anchor.chart ?? landing
    const centre = chart && player.anchor.frame === 'planet:fixture'
      ? surfaceRegion(localToSurface(chart, relativeM(player.anchor.point, ORIGIN).slice(0, 2) as [number, number]), chart.definition) : landing
    const key = centre ? `${centre.definition.digest}:${centre.id}` : ''
    if (key !== cachedKey) {
      cachedKey = key
      deposits = centre ? depositNeighborhood(centre).flatMap(regionDeposits) : []
    }
    // Ship exclusion is a visit overlay: it never rekeys or changes initial mass.
    const entities = deposits.map(p => {
      const e = p.entity
      if (world.frames.surface || !landing) return e
      const location = localToSurface(e.anchor.chart!, e.anchor.point.offsetM.slice(0, 2) as [number, number])
      return { ...e, anchor: { ...e.anchor, chart: landing, point: point2(...surfaceToLocal(location, landing)) } }
    }).filter(e => {
      const [x, y] = rotate2(relativeAnchor(e.anchor, { frame: 'ship:scout', point: ORIGIN }, world.frames), -world.frames.ship.angle)
      return Math.abs(x) > 6 || Math.abs(y) > 10
    })
    const nextWorld = { ...world, entities: [...world.entities.filter(e => !isDepositId(e.id)), ...entities] }
    return { world: nextWorld, key, deposits }
}
