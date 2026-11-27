import { GEAR_DEFINITIONS, type EquipmentState } from '../equipment.ts'
import { gearCondition } from './gearCondition.ts'
import type { TopDownState } from '../topDownRuntime.ts'

export function ScoutCharacter({ equipment, tool, mode, tick, reducedMotion }: {
  equipment: EquipmentState | undefined
  tool: boolean
  mode: TopDownState['player']['mode']
  tick: number
  reducedMotion: boolean
}) {
  const suit = equipment?.instances.find(item => item.location.kind === 'equipped')
  const suited = !equipment || !!suit
  const condition = suit ? gearCondition(suit.remaining) : suited ? 'maintained' : undefined
  const stride = mode === 'walking' && !reducedMotion ? .13 * Math.sin(tick * Math.PI / 12) : 0
  const seated = mode === 'seated'
  const modules = equipment?.instances.filter(item =>
    item.location.kind === 'installed' && item.location.suitId === suit?.id)
  return <g data-character="" data-suit-condition={condition} data-suit-equipped={String(suited)}
    strokeLinejoin="round" strokeLinecap="round">
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
      {side === 1 && tool && <g data-held-tool="" transform="translate(.055 .14) rotate(14)">
        <path d="M0 .16V-.26" stroke="#263941" strokeWidth=".07" />
        <path d="M-.04 -.25Q.09 -.35 .17 -.23L.133 -.16Q.06 -.22 -.04 -.18Z" fill="#ad9874"
          stroke="#353e40" strokeWidth=".03" />
      </g>}
    </g>)}
    {suited && <path d="M-.20 .064L-.17 .24H.17L.20 .064Z" fill="#575f60" stroke="#303e43" strokeWidth=".025" />}
    {(modules ?? []).map(item => {
      if (item.location.kind !== 'installed') return null
      const definition = GEAR_DEFINITIONS[item.definitionId]
      const moduleCondition = gearCondition(item.remaining)
      return <g key={item.id} data-character-module={item.definitionId}
        data-module-condition={moduleCondition} transform={`translate(${item.location.slot === 0 ? -.103 : .103} .215)`}>
        <rect x="-.072" y="-.065" width=".144" height=".13" rx=".031" fill={moduleCondition === 'deteriorated' ? '#80725e' : '#a7a38c'} stroke="#263840" strokeWidth=".021" />
        <path d={definition?.hazard === 'temperature' ? 'M-.045 -.031v.062M0 -.031v.062M.045 -.031v.062'
          : definition?.hazard === 'radiation' ? 'M-.041 -.021h.082v.042h-.082Z' : 'M-.035 -.03v.06M.035 -.03v.06'}
          fill="none" stroke="#46565a" strokeWidth=".021" />
        {moduleCondition !== 'maintained' && <path d="M.036 .051l.023 -.025" stroke="#dcc9a6" strokeWidth=".014" />}
      </g>
    })}
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
