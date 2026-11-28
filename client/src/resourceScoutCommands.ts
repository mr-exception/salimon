import { SCOUT_SAFE_ENVIRONMENT, samplePlanetEnvironment, bodyEnvironmentSummary } from './planetEnvironment.ts'
import { surfaceRegion, localToSurface, type SurfaceLocation } from './surfaceCoordinates.ts'
import { planetDefinition, geologicalDefinition } from './planetDefinition.ts'
import { SAMPLE_BODIES } from './sampleUniverse.ts'
import { scoutLandingSurface } from './scoutLocation.ts'
import { resolveSurface, siteEntities, validateSurfaceClearance } from './surfaceSite.ts'
import { toggleDoor } from './scoutDoor.ts'
import { transitionScout as transitionScoutLocation, takeoffDenial, type ScoutAction } from './scoutLocation.ts'
import { survivalPolicy } from './survival.ts'
import { ORIGIN, relativeM } from './coordinates.ts'
import { controlsCockpit, LOCAL_PLAYER, type CockpitState } from './cockpit.ts'
import { synchronizeScout, PLAYER_ID } from './topDownRuntime.ts'
import { type ResourceSave } from './resourceCatalog.ts'
import { materializeSite, captureResourceSites } from './resourceSaveLocation.ts'
import type { ResourceSession } from './resourceSession.ts'
type CommitResourceSave = (next: ResourceSave) => boolean
type SessionState = Pick<ResourceSession, 'activeSourceId' | 'cancel' | 'error' | 'feedback' | 'snapshot' | 'survival' | 'world'>

export function scoutCommandDenial(session: SessionState, revision = session.snapshot.revision, cockpitSession = session.world.cockpit, actorId = LOCAL_PLAYER, takeoff = true): string | null {
  if (session.error) return session.error
  if (revision !== session.snapshot.revision) return 'Expedition changed. Retry with the current revision.'
  if (session.world.entities.some(e => (e.id === PLAYER_ID || e.layer === 'people') && e.anchor.frame !== 'ship:scout')) return 'All occupants must board the Scout before launch.'
  if (cockpitSession !== session.world.cockpit) return 'Cockpit session expired. Re-enter the cockpit.'
  if (!session.world.cockpit.powered) return 'Cockpit has no power.'
  if (!session.world.cockpit.enabled || session.world.player.mode !== 'seated' || !controlsCockpit(session.world.cockpit, actorId)) return 'An authorized owner must pilot from the cockpit seat.'
  if (!survivalPolicy(session.survival).canWork) return 'Pilot cannot work in the current survival condition.'
  if (session.activeSourceId !== null) return 'Stop gathering before starting a Scout maneuver.'
  return takeoff ? takeoffDenial(session.world.scoutLocation) : null
}

/** Read-only publication for both target telemetry and physical admission. */
export function bodyDefinition(session: SessionState, bodyId: string) {
  const plan = session.world.scoutLocation.landing
  if (plan?.surface.bodyId === bodyId) return planetDefinition(bodyId, plan.surface.definition)
  const saved = Object.values(session.snapshot.sites).find(s => s.address.bodyId === bodyId)
  return saved ? planetDefinition(bodyId, saved.address.definition) : geologicalDefinition(bodyId)
}

/** Physical frame owns safety; a door or empty Core never changes interior sampling. */
export function currentEnvironment(session: SessionState, world = session.world) {
  const player = world.entities.find(e => e.id === PLAYER_ID)!
  if (player.anchor.frame === 'ship:scout') return SCOUT_SAFE_ENVIRONMENT
  const chart = player.anchor.chart ?? surfaceRegion(world.scoutLocation.surface, world.scoutLocation.surface.definition)
  return environmentAt(session, localToSurface(chart, relativeM(player.anchor.point, ORIGIN).slice(0, 2) as [number, number]))
}

/** Raw world-domain inputs. Scanner authorization belongs to the presentation consumer (#921). */
export function environmentAt(session: SessionState, location: SurfaceLocation) {
  return samplePlanetEnvironment(location, bodyDefinition(session, location.bodyId), session.snapshot.environmentPublication)
}

export function environmentSummary(session: SessionState, bodyId: string) {
  return bodyEnvironmentSummary(bodyId, bodyDefinition(session, bodyId), session.snapshot.environmentPublication)
}

/** Approved controlled migration: absent ref is legacy v1. A body's first
   * binding wins for the whole save, preventing seams between visited regions. */
function bindLandingRevision(session: SessionState, action: ScoutAction): ScoutAction {
  if (action.kind !== 'begin-landing') return action
  const body = SAMPLE_BODIES.find(b => b.id === action.bodyId)
  if (!body) return action
  const surface = scoutLandingSurface(session.world.scoutLocation, action, body)
  const saved = Object.values(session.snapshot.sites).find(s => s.address.bodyId === body.id)
  if (!saved) return action
  const { definition: _reference, ...coordinates } = surface
  return { ...action, surface: { ...coordinates, ...(saved.address.definition ? { definition: saved.address.definition } : {}) } }
}

/** Read-only admission uses the command's domain/site validation, including saved obstacles. */
export function scoutActionDenial(session: SessionState, action: ScoutAction, revision = session.snapshot.revision, cockpitSession = session.world.cockpit, actorId = LOCAL_PLAYER): string | null {
  const denial = scoutCommandDenial(session, revision, cockpitSession, actorId, action.kind === 'begin-takeoff')
  if (denial) return denial
  try {
    previewScoutAction(session, action)
    return null
  } catch (error) { return error instanceof Error ? error.message : 'Scout transition rejected.' }
}

function previewScoutAction(session: SessionState, action: ScoutAction) {
  const location = session.world.scoutLocation
  const sites = captureResourceSites(session.world, session.snapshot.sites)
  const next = transitionScoutLocation(location, bindLandingRevision(session, action))
  if (action.kind === 'begin-landing' || action.kind === 'begin-takeoff') {
    const site = resolveSurface(action.kind === 'begin-takeoff' ? location.surface : next.surface)
    validateSurfaceClearance(site, siteEntities(site, sites[site.id]))
  }
  return next
}

export function landingPreview(session: SessionState, bodyId: string): { denial: string | null; surface?: SurfaceLocation } {
  const location = session.world.scoutLocation
  const site = location.landing?.surface ?? (location.binding ? location.surface : undefined)
  const surface = site?.bodyId === bodyId ? site : undefined
  const denial = scoutCommandDenial(session, session.snapshot.revision, session.world.cockpit, LOCAL_PLAYER, false)
  if (denial) return { denial, surface }
  try {
    return { denial: null, surface: previewScoutAction(session, { kind: 'begin-landing', bodyId }).surface }
  } catch (error) {
    return { denial: error instanceof Error ? error.message : 'Scout transition rejected.', surface }
  }
}

/** Explicit commands are revision-bound and authorized against the live seat. */
export function transitionScout(session: SessionState, commit: CommitResourceSave, syncDeposits: () => void, action: ScoutAction, revision: number, cockpitSession: CockpitState, actorId = LOCAL_PLAYER): boolean {
  const deny = (reason: string) => { session.feedback = reason; return false }
  const admission = scoutActionDenial(session, action, revision, cockpitSession, actorId)
  if (admission) return deny(admission)
  const previous = session.world, location = session.world.scoutLocation, before = { ...location }
  try {
    const sites = captureResourceSites(session.world, session.snapshot.sites)
    if (action.kind === 'begin-takeoff') validateSurfaceClearance(resolveSurface(location.surface), siteEntities(resolveSurface(location.surface), sites[location.siteId]))
    const next = transitionScoutLocation(location, bindLandingRevision(session, action))
    if (action.kind === 'begin-landing') validateSurfaceClearance(resolveSurface(next.surface), siteEntities(resolveSurface(next.surface), sites[next.siteId]))
    session.cancel()
    Object.assign(location, next)
    session.world = materializeSite(synchronizeScout(session.world), sites)
    syncDeposits()
    if (!commit({ ...session.snapshot, sites })) { Object.assign(location, before); session.world = previous; return false }
    session.feedback = ''
    session.world = { ...session.world, feedback: `Scout: ${location.ship.mode}.` }
    return true
  } catch (error) { Object.assign(location, before); session.world = previous; session.feedback = error instanceof Error ? error.message : 'Scout transition rejected.'; return false }
}

/** The real E/mouse door action uses the same atomic save as flight. */
export function toggleBoardingDoor(session: SessionState, commit: CommitResourceSave, revision: number): boolean {
  if (session.error || session.survival.health === 0) return false
  if (revision !== session.snapshot.revision) { session.feedback = 'Expedition changed. Retry with the current revision.'; return false }
  const previous = session.world, location = previous.scoutLocation, before = location.boardingDoor
  const next = toggleDoor(previous)
  if (next.scoutLocation.boardingDoor === before) { session.feedback = next.feedback; return false }
  location.boardingDoor = next.scoutLocation.boardingDoor
  session.world = { ...next, scoutLocation: location }
  if (!commit(session.snapshot)) { location.boardingDoor = before; session.world = previous; return false }
  session.activeSourceId = null
  session.feedback = next.feedback
  return true
}
