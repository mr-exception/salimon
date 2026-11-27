import { SUIT_ID, type GearInstance } from '../equipment.ts'
import { gearCondition } from './gearCondition.ts'
import { ScoutCharacter } from './ScoutCharacter.tsx'
import { GatheringToolArtwork, ModuleArtwork } from './EquipmentParts.tsx'

export function EquipmentIcon({ gear }: { gear?: GearInstance }) {
  const id = gear?.definitionId ?? 'gathering-tool'
  const condition = gear ? gearCondition(gear.remaining) : 'maintained'
  const art = id === SUIT_ID
    ? <ScoutCharacter art={{ suited: true, condition, modules: [], tool: false, seated: false, dynamicShadow: false }} shadow={false} />
    : gear ? <ModuleArtwork definitionId={id} condition={condition} /> : <GatheringToolArtwork />
  const scale = gear && id !== SUIT_ID ? 101 : gear ? 235 : 365
  return <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256"
    className="equipment-art-icon" aria-hidden="true" focusable="false"
    data-equipment-icon={id} data-icon-condition={condition}>
    <g transform={`translate(121 126) matrix(.87 .28 -.44 .76 0 0) scale(${scale})`}>
      <g transform="translate(.06 .14)" style={{ filter: "brightness(.42)" }}>{art}</g>
      {art}
    </g>
  </svg>
}
