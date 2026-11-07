import { compassDirection } from './orientationCompass.ts'
import type { ObservationSnapshot } from './observationCamera.ts'
import { forward, type ShipState } from './simulation.ts'

export function OrientationCompass({ ship, view }: { ship: ShipState; view: ObservationSnapshot }) {
  const heading = compassDirection(forward(ship.yaw, ship.pitch), view)
  const velocity = compassDirection(ship.velocityMps, view)
  const description = (marker: typeof heading) => marker ? `${marker.rear ? 'rear' : 'front'}, right ${marker.x.toFixed(1)}, up ${(-marker.y).toFixed(1)}, depth ${marker.depth.toFixed(2)}` : 'Stationary'
  return <figure className="orientation-compass" aria-label="3D orientation compass">
    <svg viewBox="-55 -55 110 110" role="img" aria-label={`Camera cyan at center. Heading yellow: ${description(heading)}. Velocity green: ${description(velocity)}. Solid front, hollow rear.`}>
      <circle r="43" fill="var(--ui-void, #09151d)" stroke="var(--ui-muted, #587480)"/>
      <ellipse rx="43" ry="14" fill="none" stroke="var(--ui-surface, #38515f)"/>
      <path d="M-43 0H43 M0-43V43" stroke="var(--ui-surface, #38515f)"/>
      {velocity && <circle data-compass="velocity" data-rear={velocity.rear} cx={velocity.x} cy={velocity.y} r="8" fill={velocity.rear ? 'var(--ui-void, #09151d)' : 'var(--ui-velocity, #b6f5a2)'} stroke="var(--ui-velocity, #b6f5a2)" strokeWidth="2" strokeDasharray={velocity.rear ? '2 2' : undefined}/>}
      {heading && <path data-compass="heading" data-rear={heading.rear} d="M0-5 5 4H-5Z" transform={`translate(${heading.x} ${heading.y})`} fill={heading.rear ? 'var(--ui-void, #09151d)' : 'var(--ui-heading, #ffe36a)'} stroke="var(--ui-heading, #ffe36a)" strokeWidth="1.5"/>}
      <circle r="1.6" fill="var(--ui-active, #60e8ff)"/>
      <text y="-47" textAnchor="middle" fill="var(--ui-muted, #abc2cd)" fontSize="7">UP</text>
    </svg>
    <figcaption><span>Cyan view</span> · <b>Yellow heading</b><br/><em>{velocity ? 'Green velocity' : 'Stationary'}</em> · solid front / hollow rear</figcaption>
  </figure>
}
