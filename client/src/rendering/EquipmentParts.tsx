import { GEAR_DEFINITIONS } from '../equipment.ts'
import { gearCondition } from './gearCondition.ts'

const metal = '#acaa98'
const shadow = '#344950'
const edge = '#596d70'

export function ModuleArtwork({ definitionId, condition }: {
  definitionId: string
  condition: ReturnType<typeof gearCondition>
}) {
  const definition = GEAR_DEFINITIONS[definitionId]
  if (!definition?.hazard) return null
  const { hazard, tier = 1 } = definition
  const housing = condition === 'deteriorated' ? '#847866' : metal
  return <g stroke={shadow} strokeWidth=".12" strokeLinejoin="round" strokeLinecap="round">
    {hazard === 'temperature' && <>
      <path d="M-.86 -.49 -.62 -.87H.64L.87 -.49V.56L.62 .83H-.66L-.86 .56Z" fill={housing} />
      <path d="M-.68 -.52H.67V.54H-.68Z" fill={edge} />
      <path d="M-.54 -.39H.54M-.54 -.11H.54M-.54 .17H.54M-.54 .44H.54" stroke="#d2c9aa" />
      <path d="M-.7 -.68H.68" stroke="#a68c64" />
    </>}
    {hazard === 'radiation' && <>
      <path d="M-.83 -.62 -.46 -.86H.60L.83 -.53V.63L.54 .85H-.62L-.83 .54Z" fill={housing} />
      <path d="M-.57 -.43 -.33 -.62H.44L.61 -.39V.34L.37 .57H-.44L-.57 .34Z" fill="#728184" />
      <path d="M-.37 -.24 -.13 -.41H.32L.43 -.18V.18L.22 .37H-.29L-.37 .15Z" fill="#c5c6b6" />
    </>}
    {hazard === 'poison' && <>
      <path d="M-.77 -.45 -.63 -.78H.63L.77 -.45V.58L.47 .86H-.47L-.77 .58Z" fill={housing} />
      <rect x="-.60" y="-.51" width=".47" height="1.04" rx=".19" fill="#747b68" />
      <rect x=".13" y="-.51" width=".47" height="1.04" rx=".19" fill="#747b68" />
      <path d="M-.49 -.28h.23m-.23 .48h.23m.52 -.48h.23m-.23 .48h.23" stroke="#c5c6b0" />
    </>}
    {Array.from({ length: tier }, (_, index) =>
      <path key={index} d={`M${-.26 + .23 * index} .64v.14`} stroke="#e3d6b1" strokeWidth=".13" />)}
    {condition !== 'maintained' && <path d="M.57 .34l.17 -.23" stroke="#e4c5a0" />}
    {(condition === 'deteriorated' || condition === 'broken') && <path d="M-.73 .33l.20 -.18 .16 .2" fill="none" />}
    {condition === 'broken' && <path d="M-.43 -.63 -.16 -.31 -.29 -.10 .12 .16 .07 .60" fill="none" />}
  </g>
}

export function GatheringToolArtwork() {
  return <g stroke="#283a41" strokeLinecap="round" strokeLinejoin="round" strokeWidth=".027">
    <path d="M-.025 .16 -.017 -.185H.031L.034 .16Z" fill="#778488" />
    <path d="M-.035 .045H.046V.155L.024 .19H-.024Z" fill="#a1835f" />
    <path d="M-.060 -.22 -.031 -.292 .083 -.298 .161 -.238 .135 -.191 .069 -.208 .016 -.176Z" fill="#b6b9a7" />
    <path d="M-.039 -.226 .063 -.263 .125 -.234" fill="none" stroke="#65777a" strokeWidth=".018" />
    <path d="M-.012 -.071v.071" stroke="#bcc5bb" strokeWidth=".015" />
  </g>
}
