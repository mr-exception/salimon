import { useRef, type CSSProperties, type PointerEvent } from 'react';
import { INITIAL_SPACESHIP_FUEL_KNS, useSpaceshipFuelKns } from '@store';
import coreUrl from './core.svg';
import style from './style.module.css';
import thrusterUrl from './thruster.svg';

const HULL_COLUMNS = 20;
const HULL_ROWS = 20;
const HULL_TILE_COUNT = HULL_COLUMNS * HULL_ROWS;

type PanState = {
  pointerId: number;
  x: number;
  y: number;
};

export function SpaceshipHull() {
  const panStateRef = useRef<PanState | undefined>(undefined);
  const fuelKns = useSpaceshipFuelKns();
  const coreCharge = Math.min(
    1,
    Math.max(0, fuelKns / INITIAL_SPACESHIP_FUEL_KNS),
  );

  const startPanning = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;

    event.currentTarget.setPointerCapture(event.pointerId);
    panStateRef.current = {
      pointerId: event.pointerId,
      x: event.clientX,
      y: event.clientY,
    };
    event.currentTarget.dataset.panning = 'true';
  };

  const pan = (event: PointerEvent<HTMLDivElement>) => {
    const panState = panStateRef.current;
    if (!panState || panState.pointerId !== event.pointerId) return;

    event.currentTarget.scrollBy(
      panState.x - event.clientX,
      panState.y - event.clientY,
    );
    panState.x = event.clientX;
    panState.y = event.clientY;
  };

  const stopPanning = (event: PointerEvent<HTMLDivElement>) => {
    if (panStateRef.current?.pointerId !== event.pointerId) return;

    panStateRef.current = undefined;
    delete event.currentTarget.dataset.panning;
  };

  return (
    <section className={style.container} aria-label="Spaceship hull">
      <div
        className={style.viewport}
        aria-label="Pannable spaceship hull"
        onPointerDown={startPanning}
        onPointerMove={pan}
        onPointerUp={stopPanning}
        onPointerCancel={stopPanning}
        tabIndex={0}
      >
        <div className={style.hull} role="grid" aria-label="Spaceship floor">
          {Array.from({ length: HULL_TILE_COUNT }, (_, index) => (
            <div
              className={style.floorTile}
              key={index}
              role="gridcell"
              aria-label={`Floor tile ${Math.floor(index / HULL_COLUMNS) + 1}, ${(index % HULL_COLUMNS) + 1}`}
            />
          ))}
          <figure
            className={style.core}
            aria-label={`Core, ${Math.round(coreCharge * 100)}% fuel`}
            style={{ '--core-charge': coreCharge } as CSSProperties}
          >
            <img src={coreUrl} alt="" />
            <figcaption>Core</figcaption>
          </figure>
          {(['top', 'right', 'bottom', 'left'] as const).map((edge) => (
            <figure
              className={`${style.thruster} ${style[edge]}`}
              aria-label={`${edge} thruster`}
              key={edge}
            >
              <img src={thrusterUrl} alt="" />
            </figure>
          ))}
        </div>
      </div>
    </section>
  );
}
