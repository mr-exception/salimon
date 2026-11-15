import { sampleGeology } from './planetGeology.ts'
import { planetDefinition, type PlanetReference } from './planetDefinition.ts'
import { neighboringRegions, surfaceRegion, type SurfaceLocation, type SurfaceRegion } from './surfaceCoordinates.ts'

export const STREAM_REGION_LIMIT = 32
export const STREAM_SAMPLE_LIMIT = 2048
export const STREAM_FRAME_BUDGET = 2048
export type GroundSample = Pick<ReturnType<typeof sampleGeology>, 'groundType' | 'grade' | 'slopeRadians' | 'heightM'>
const keyOrder = (a: string, b: string) => a < b ? -1 : a > b ? 1 : 0
interface Entry {region: SurfaceRegion; samples: Map<string, GroundSample>; used: number}
/** Disposable immutable baseline only: source journals and saves never enter
 * this cache. Physical player selects neighbourhood, independent of viewport. */
export class RegionStream {
  private entries = new Map<string,Entry>()
  private sequence = 0
  private retained = new Set<string>()
  private pending: SurfaceRegion[] = []
  private reference?: PlanetReference
  readonly stats = {generated:0,evicted:0,hits:0,frameGenerated:0,regions:0,samples:0,pending:0}
  select(location: SurfaceLocation, reference: PlanetReference) {
    planetDefinition(location.bodyId,reference)
    if (this.reference?.digest !== reference.digest || this.reference.bodyId !== reference.bodyId) {
      this.entries.clear(); this.pending=[]; this.retained.clear()
    }
    this.reference=reference
    const centre=surfaceRegion(location,reference)
    const rings = new Map([[centre.id,centre]])
    let frontier=[centre]
    for(let ring=0;ring<3;ring++) {
      const next:SurfaceRegion[]=[]
      for(const region of frontier) for(const n of neighboringRegions(region)) if(!rings.has(n.id)) {rings.set(n.id,n);next.push(n)}
      frontier=next
    }
    this.retained=new Set(rings.keys())
    this.pending=[...rings.values()].filter(r=>!this.entries.has(r.id)).sort((a,b)=>keyOrder(a.id,b.id))
    // Synchronous capped work: replace rather than accumulate stale requests.
    for(const [key] of this.entries) if(!this.retained.has(key)) {this.entries.delete(key);this.stats.evicted++}
    this.stats.frameGenerated=0;this.stats.pending=this.pending.length
  }
  private entry(region:SurfaceRegion):Entry {
    let entry=this.entries.get(region.id)
    if(!entry) {
      if(this.entries.size>=STREAM_REGION_LIMIT) {
        const oldest=[...this.entries.entries()].sort((a,b)=>Number(this.retained.has(a[0]))-Number(this.retained.has(b[0])) || a[1].used-b[1].used || keyOrder(a[0],b[0]))[0]!
        this.entries.delete(oldest[0]);this.stats.evicted++
      }
      entry={region,samples:new Map(),used:0};this.entries.set(region.id,entry)
    }
    entry.used=++this.sequence
    return entry
  }
  sample(location:SurfaceLocation):GroundSample {
    if(!this.reference) throw Error('Select a physical streaming neighbourhood first')
    const region=surfaceRegion(location,this.reference),entry=this.entry(region)
    const key=location.radialM.join(':')
    const cached=entry.samples.get(key)
    if(cached) {this.stats.hits++;return cached}
    if(this.stats.frameGenerated>=STREAM_FRAME_BUDGET) throw Error('Streaming frame budget exceeded')
    const sample:GroundSample=this.reference.version===2 ? sampleGeology(location,this.reference) : {groundType:'authored-ground',grade:0,slopeRadians:0,heightM:0}
    if(entry.samples.size>=STREAM_SAMPLE_LIMIT) entry.samples.delete(entry.samples.keys().next().value!)
    entry.samples.set(key,Object.freeze(sample));this.stats.generated++;this.stats.frameGenerated++
    return sample
  }
  prefetch() {
    for(const region of this.pending) {
      if(this.stats.frameGenerated>=STREAM_FRAME_BUDGET) break
      this.sample({bodyId:region.bodyId,radialM:region.centre})
    }
    this.stats.regions=this.entries.size
    this.stats.samples=[...this.entries.values()].reduce((sum,e)=>sum+e.samples.size,0)
    this.stats.pending=0;this.pending=[]
  }
}
