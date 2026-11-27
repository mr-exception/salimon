import type { EquipmentState } from '../equipment.ts'
import { gearCondition } from './gearCondition.ts'
import { GatheringToolArtwork, ModuleArtwork } from './EquipmentParts.tsx'
import type { TopDownState } from '../topDownRuntime.ts'

export interface CharacterArt {
  suited: boolean
  condition: ReturnType<typeof gearCondition> | undefined
  modules: Array<{ slot: number; definitionId: string; condition: ReturnType<typeof gearCondition> }>
  tool: boolean
  seated: boolean
  dynamicShadow: boolean
}

export function characterArt(equipment: EquipmentState | undefined, tool: boolean,
  mode: TopDownState['player']['mode'], dynamicShadow: boolean): CharacterArt {
  const suit = equipment?.instances.find(item => item.location.kind === 'equipped')
  const suited = !equipment || !!suit
  const modules = equipment?.instances.flatMap<CharacterArt['modules'][number]>(item =>
    item.location.kind === 'installed' && item.location.suitId === suit?.id
      ? [{ slot: item.location.slot, definitionId: item.definitionId, condition: gearCondition(item.remaining) }]
      : []) ?? []
  return { suited, condition: suit ? gearCondition(suit.remaining) : suited ? 'maintained' : undefined,
    modules, tool, seated: mode === 'seated', dynamicShadow }
}

export function ScoutCharacter({ art, stride = 0, shadow = true }: { art: CharacterArt; stride?: number; shadow?: boolean }) {
  const { suited, condition, modules, tool, seated, dynamicShadow } = art
  return <g strokeLinejoin="round" strokeLinecap="round">
    {shadow && <ellipse data-character-shadow="" cx={dynamicShadow ? '.13' : '0'} cy=".225" rx=".34" ry=".235"
      fill="#18242a" opacity={dynamicShadow ? '.26' : '.19'} pointerEvents="none" />}
    {[-1, 1].map(side => <g key={side} transform={`translate(${side * .16} ${side * stride})`}>
      <path d={seated ? 'M-.085 .02H.085L.07 -.25H-.07Z' : 'M-.09 .11H.09L.075 .45H-.075Z'}
        fill={suited ? '#55666b' : '#68737c'} stroke="#283a40" strokeWidth=".035" />
      <path d={seated ? 'M-.075 -.22H.075' : 'M-.075 .42H.075'} stroke="#25363e" strokeWidth=".09" />
    </g>)}
    <path d="M-.29 -.12L-.23 -.24H.23L.29 -.12 .22 .25H-.22Z"
      fill={suited ? '#a8a18d' : '#536974'} stroke="#2d3e43" strokeWidth=".042" />
    {[-1, 1].map(side => <g key={side} transform={`translate(${side * .30} ${-side * stride * .65})`}>
      <path d="M0 -.11L.018 .20" stroke="#344650" strokeWidth=".145" />
      <path d="M0 -.12L.008 .02" stroke={suited ? '#c8c2ad' : '#657983'} strokeWidth=".13" />
      {side === 1 && tool && <g transform="translate(.055 .14) rotate(14)">
        <GatheringToolArtwork />
      </g>}
    </g>)}
    {suited && <path d="M-.20 .064L-.17 .24H.17L.20 .064Z" fill="#575f60" stroke="#303e43" strokeWidth=".025" />}
    {modules.map(item => <g key={item.slot}
      transform={`translate(${item.slot === 0 ? -.103 : .103} .215) scale(.078 .070)`}>
      <ModuleArtwork definitionId={item.definitionId} condition={item.condition} />
    </g>)}
    <path d="M-.164 -.22Q0 -.37 .164 -.22L.153 -.014Q0 .121 -.153 -.014Z"
      fill={suited ? '#e2ddc9' : '#916a53'} stroke="#394a50" strokeWidth=".037" />
    {suited && <path d="M-.137 -.219Q0 -.307 .137 -.219L.119 -.14Q0 -.186 -.119 -.14Z"
      fill={condition === 'broken' ? '#657679' : '#76aeb3'} stroke="#3f5b61" strokeWidth=".023" />}
    {condition && condition !== 'maintained' && <g fill="none" stroke="#72634f" strokeWidth=".024">
      <path d="M-.128 -.078l.054 .036m.212 .171l.039 -.036" />
      {(condition === 'deteriorated' || condition === 'broken') && <path d="M.11 -.114l-.052 .078 .036 .063m-.32 .076l.08 .023" />}
      {condition === 'broken' && <path d="M.08 -.24l-.059 .067 .027 .029 -.05 .05" stroke="#32454b" />}
    </g>}
  </g>
}
