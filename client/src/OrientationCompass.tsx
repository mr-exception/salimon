import { compassDirection } from './orientationCompass.ts'
import type { ObservationSnapshot } from './observationCamera.ts'
import { forward, type ShipState } from './simulation.ts'

export function OrientationCompass({ ship, view }: { ship: ShipState; view: ObservationSnapshot }) {
  const heading = compassDirection(forward(ship.yaw, ship.pitch), view)
  const velocity = compassDirection(ship.velocityMps, view)
  const description = (marker: typeof heading) => marker ? `${marker.rear ? 'rear' : 'front'}, right ${marker.x.toFixed(1)}, up ${(-marker.y).toFixed(1)}, depth ${marker.depth.toFixed(2)}` : 'Stationary'
  return <figure className="orientation-compass" aria-label="3D orientation compass">
    <svg viewBox="-55 -55 110 110" role="img" aria-label={`Camera view cyan at center. Ship heading amber: ${description(heading)}. Velocity green: ${description(velocity)}. Solid front, hollow rear.`}>
      <circle r="43" fill="var(--ui-void, #09151d)" stroke="var(--ui-muted, #587480)"/>
      <ellipse rx="43" ry="14" fill="none" stroke="var(--ui-surface, #38515f)"/>
      <path d="M-43 0H43 M0-43V43" stroke="var(--ui-surface, #38515f)"/>
      {[{ key: 'heading', marker: heading, ink: 'var(--ui-heading, #ebc076)' }, { key: 'velocity', marker: velocity, ink: 'var(--ui-velocity, #b6f5a2)' }].map(({ key, marker, ink }) => marker && <line key={key} data-compass-line={key} x1="0" y1="0" x2={marker.x} y2={marker.y} stroke={ink} strokeWidth={key === 'heading' ? '2' : '1.5'} strokeDasharray={key === 'velocity' || marker.rear ? '3 2' : undefined}/>)}
      {velocity && <circle data-compass="velocity" data-rear={velocity.rear} cx={velocity.x} cy={velocity.y} r="8" fill={velocity.rear ? 'var(--ui-void, #09151d)' : 'var(--ui-velocity, #b6f5a2)'} stroke="var(--ui-velocity, #b6f5a2)" strokeWidth="2" strokeDasharray={velocity.rear ? '2 2' : undefined}/>}
      {heading && <path data-compass="heading" data-rear={heading.rear} d="M0-5 5 4H-5Z" transform={`translate(${heading.x} ${heading.y})`} fill={heading.rear ? 'var(--ui-void, #09151d)' : 'var(--ui-heading, #ebc076)'} stroke="var(--ui-heading, #ebc076)" strokeWidth="1.5"/>}
      <path d="M-3 0H3 M0-3V3" stroke="var(--ui-active, #60e8ff)" strokeWidth="1"/>
      <circle r="1.6" fill="var(--ui-active, #60e8ff)"/>
      <text y="-47" textAnchor="middle" fill="var(--ui-muted, #abc2cd)" fontSize="7">UP</text>
    </svg>
    <figcaption><span>Cyan camera</span> · <b>Amber ship</b><br/><em>{velocity ? 'Green velocity' : 'Stationary'}</em> · solid front / hollow rear</figcaption>
  </figure>
}
