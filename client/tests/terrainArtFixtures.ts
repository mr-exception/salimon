import {ResourceSession} from '../src/resourceSession.ts'
import {createTopDownState,PLAYER_ID} from '../src/topDownRuntime.ts'
import {surfaceScoutLocation} from '../src/scoutLocation.ts'
import {resolveSurface,siteEntities,startingSurface} from '../src/surfaceSite.ts'
import {surfaceRegion,localToSurface,surfaceToLocal} from '../src/surfaceCoordinates.ts'
import {geologicalDefinition} from '../src/planetDefinition.ts'
import {normal,scale,type Vec3} from '../src/math.ts'
import {point2,walkingFrames} from '../src/topDownFrames.ts'

export const ART_FIXTURES=[
  {name:'barren',bodyId:'caligo',direction:[1,0,0],offset:[0,0]},
  {name:'crater-highlands',bodyId:'vesper',direction:[-3,-3,0],offset:[0,0]},
  {name:'ice',bodyId:'vesper-moon',direction:[0,1,0],offset:[0,0]},
  {name:'fractured',bodyId:'caligo',direction:[1,1,1],offset:[0,0]},
] as const
export function terrainArtSave(bodyId:string,direction:Vec3,offset:readonly[number,number]=[0,0]) {
  const {version,digest}=geologicalDefinition(bodyId),definition={bodyId,version,digest},base={bodyId,radialM:scale(normal(direction),geologicalDefinition(bodyId).radiusM+5)}
  const region=surfaceRegion(base,definition),physical=localToSurface(region,offset)
  const address=bodyId==='vesper' ? startingSurface() : {...localToSurface(region,[19500,0]),yaw:0,definition}
  const site=resolveSurface(address),scoutLocation=surfaceScoutLocation(address),chart=surfaceRegion(address,definition)
  const p=surfaceToLocal(physical,region),anchor={frame:'planet:fixture' as const,point:point2(...p),chart:region},world=createTopDownState()
  const session=new ResourceSession({getItem:()=>null,setItem:()=>{}},{...world,scoutLocation,frames:walkingFrames(site.frames,chart,region),camera:{...world.camera,anchor},entities:[...world.entities.filter(e=>e.layer!=='terrain').map(e=>e.id===PLAYER_ID?{...e,anchor}:e),...siteEntities(site)]},undefined,undefined,()=>Date.parse('2026-11-04T00:00:00Z'))
  if(session.error)throw Error(session.error)
  return JSON.stringify(session.snapshot,(_,v)=>typeof v==='bigint'?String(v):v)
}
