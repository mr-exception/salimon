import { ResourceSession } from '../src/resourceSession.ts'
import { createTopDownState, PLAYER_ID } from '../src/topDownRuntime.ts'
import { geologicalDefinition } from '../src/planetDefinition.ts'
import { localToSurface, surfaceRegion } from '../src/surfaceCoordinates.ts'
import { regionDeposits } from '../src/planetDeposits.ts'
import { point2, walkingFrames } from '../src/topDownFrames.ts'
import { resolveSurface, siteEntities } from '../src/surfaceSite.ts'
import { surfaceScoutLocation } from '../src/scoutLocation.ts'
export function depositFixture(index = 0) {
  const definition = geologicalDefinition('vesper-moon')
  const origin = surfaceRegion({ bodyId: definition.bodyId, radialM: [definition.radiusM + 5, 0, 0] }, definition)
  const region = surfaceRegion(localToSurface(origin, [index * 64, 0]), definition), deposit = regionDeposits(region)[0]
  const [x,y] = deposit?.entity.anchor.point.offsetM ?? [0,0]
  const address = { ...localToSurface(origin, [22528, 1000]), yaw: 0, definition: {bodyId: definition.bodyId,version: definition.version,digest: definition.digest} }
  const site = resolveSurface(address), scoutLocation = surfaceScoutLocation(address), base = createTopDownState('exterior', true)
  const anchor = { frame: 'planet:fixture' as const, chart: region, point: point2(x + 1.5, y) }
  const world = { ...base, scoutLocation, frames: walkingFrames(site.frames, surfaceRegion(address, definition), region), camera: {...base.camera, anchor},
    entities: [...base.entities.filter(e => e.layer !== 'terrain').map(e => e.id === PLAYER_ID ? {...e,anchor} : e), ...siteEntities(site)] }
  const session = new ResourceSession({getItem: () => null,setItem: () => {}}, world, undefined, undefined, () => Date.parse('2026-11-04T00:00:00Z'))
  if (session.error) throw Error(session.error)
  return {raw: JSON.stringify(session.snapshot, (_,v) => typeof v === 'bigint' ? String(v) : v), deposit}
}
