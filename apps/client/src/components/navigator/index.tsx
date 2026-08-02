import { useEffect, useRef, useState } from 'react';
import Phaser from 'phaser';
import { type SpaceshipProximityTelemetry } from '@store';
import {
  formatAngle,
  formatDistance,
  formatSiValue,
  formatSpeed,
} from '../../utils';
import { SearchDialog, type SearchResult } from '../search';
import { BodyContextMenu } from './body-context-menu';
import {
  Scene,
  type BodyContextMenuRequest,
  type BodyDetailsRequest,
  type TargetDirectionPreview,
} from './game/scene';
import { MAX_ZOOM, MIN_ZOOM } from './game/scene/configure-input';
import style from './style.module.css';

const SCALE_WIDTH_PX = 200;
function formatScaleDistance(zoom: number) {
  return formatDistance(SCALE_WIDTH_PX / zoom);
}

function formatMass(valueInKilograms: bigint) {
  return formatSiValue(Number(valueInKilograms), 'kg');
}

function formatVelocityDirection(velocity: { x: number; y: number }) {
  const speed = Math.hypot(velocity.x, velocity.y);
  if (speed === 0) return formatAngle(0);

  const degrees = (Math.atan2(velocity.y, velocity.x) * 180) / Math.PI;
  return formatAngle(Math.round((degrees + 360) % 360));
}

function proximityTelemetryChanged(
  current: SpaceshipProximityTelemetry | undefined,
  next: SpaceshipProximityTelemetry | undefined,
) {
  return (
    current?.bodyName !== next?.bodyName ||
    current?.bodyKind !== next?.bodyKind ||
    current?.surfaceDistanceMeters !== next?.surfaceDistanceMeters ||
    current?.relativeSpeedMetersPerSecond !== next?.relativeSpeedMetersPerSecond
  );
}

function getBodyDetailsTitle(details: BodyDetailsRequest) {
  return details.body.name;
}

function formatMaterials(details: BodyDetailsRequest) {
  if (details.kind !== 'Asteroid') return undefined;

  return details.body.materials
    .map(({ name, massKg }) => `${name}: ${formatSiValue(massKg, 'kg')}`)
    .join(', ');
}

function BodyDetailsDialog({
  details,
  onDismiss,
}: {
  details: BodyDetailsRequest;
  onDismiss: () => void;
}) {
  useEffect(() => {
    const dismissOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onDismiss();
    };

    window.addEventListener('keydown', dismissOnEscape);
    return () => window.removeEventListener('keydown', dismissOnEscape);
  }, [onDismiss]);

  const velocitySpeed = Math.hypot(details.velocity.x, details.velocity.y);
  const title = getBodyDetailsTitle(details);
  const materials = formatMaterials(details);

  return (
    <>
      <button
        type="button"
        className={style.bodyDetailsBackdrop}
        aria-label="Close body details"
        onClick={onDismiss}
      />
      <section
        className={style.bodyDetailsDialog}
        role="dialog"
        aria-modal="true"
        aria-labelledby="navigator-body-details-title"
      >
        <header>
          <div>
            <h2 id="navigator-body-details-title">{title}</h2>
            <span>{details.kind}</span>
          </div>
          <button type="button" aria-label="Close details" onClick={onDismiss}>
            ×
          </button>
        </header>
        <dl>
          <div>
            <dt>System</dt>
            <dd>{details.systemName}</dd>
          </div>
          <div>
            <dt>
              {details.kind === 'Asteroid' ? 'Relative speed' : 'Current speed'}
            </dt>
            <dd>{formatSpeed(velocitySpeed)}</dd>
          </div>
          <div>
            <dt>
              {details.kind === 'Asteroid' ? 'Relative direction' : 'Direction'}
            </dt>
            <dd>{formatVelocityDirection(details.velocity)}</dd>
          </div>
          <div>
            <dt>Mass</dt>
            <dd>{formatMass(details.body.mass)}</dd>
          </div>
          <div>
            <dt>Size</dt>
            <dd>{formatDistance(Number(details.body.radius) * 2)}</dd>
          </div>
          {details.kind === 'Asteroid' && (
            <div>
              <dt>Orbit surface</dt>
              <dd>{formatDistance(details.orbitSurfaceDistanceMeters)}</dd>
            </div>
          )}
          {materials && (
            <div>
              <dt>Materials</dt>
              <dd title={materials}>{materials}</dd>
            </div>
          )}
        </dl>
      </section>
    </>
  );
}

type NavigatorProps = {
  isMeasuring?: boolean;
  isMeasurementRelativeToSpaceship?: boolean;
  isMeasurementVelocityAxesSeparated?: boolean;
  isRulerActive?: boolean;
  isSearchOpen?: boolean;
  isSelectingTargetDirection?: boolean;
  onCloseSearch?: () => void;
  onSceneChange?: (scene: Scene | null) => void;
  onSpaceshipEngineChange?: (isRunning: boolean) => void;
  onTargetDirectionSelected?: () => void;
};

export function Navigator({
  isMeasuring = false,
  isMeasurementRelativeToSpaceship = false,
  isMeasurementVelocityAxesSeparated = false,
  isRulerActive = false,
  isSearchOpen = false,
  isSelectingTargetDirection = false,
  onCloseSearch,
  onSceneChange,
  onSpaceshipEngineChange,
  onTargetDirectionSelected,
}: NavigatorProps) {
  const gameHostRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<Scene>(null);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [worldLoadState, setWorldLoadState] = useState<
    'loading' | 'ready' | 'error'
  >('loading');
  const [isWorldViewportLoading, setIsWorldViewportLoading] = useState(false);
  const [contextMenu, setContextMenu] = useState<BodyContextMenuRequest | null>(
    null,
  );
  const [bodyDetails, setBodyDetails] = useState<BodyDetailsRequest | null>(
    null,
  );
  const [targetPreview, setTargetPreview] =
    useState<TargetDirectionPreview | null>(null);
  const [proximityTelemetry, setProximityTelemetry] =
    useState<SpaceshipProximityTelemetry>();
  const [isProximityExpanded, setIsProximityExpanded] = useState(false);
  const [showAsteroids, setShowAsteroids] = useState(true);

  useEffect(() => {
    sceneRef.current?.setMeasuringActive(isMeasuring);
  }, [isMeasuring]);

  useEffect(() => {
    sceneRef.current?.setMeasurementRelativeToSpaceship(
      isMeasurementRelativeToSpaceship,
    );
  }, [isMeasurementRelativeToSpaceship]);

  useEffect(() => {
    sceneRef.current?.setMeasurementVelocityAxesSeparated(
      isMeasurementVelocityAxesSeparated,
    );
  }, [isMeasurementVelocityAxesSeparated]);

  useEffect(() => {
    sceneRef.current?.setRulerActive(isRulerActive);
  }, [isRulerActive]);

  useEffect(() => {
    sceneRef.current?.setAsteroidsVisible(showAsteroids);
  }, [showAsteroids]);

  useEffect(() => {
    if (!gameHostRef.current) return;

    const scene = new Scene(
      setZoomLevel,
      setContextMenu,
      setBodyDetails,
      undefined,
      (engineIsRunning) => onSpaceshipEngineChange?.(engineIsRunning),
      (preview) => setTargetPreview(preview ?? null),
      onTargetDirectionSelected,
      (error) => setWorldLoadState(error ? 'error' : 'ready'),
      setIsWorldViewportLoading,
      (nextTelemetry) =>
        setProximityTelemetry((current) =>
          proximityTelemetryChanged(current, nextTelemetry)
            ? nextTelemetry
            : current,
        ),
    );
    sceneRef.current = scene;
    onSceneChange?.(scene);
    const game = new Phaser.Game({
      type: Phaser.AUTO,
      parent: gameHostRef.current,
      backgroundColor: '#050816',
      fps: {
        forceSetTimeOut: true,
        // Preserve real elapsed time when background tabs are throttled.
        smoothStep: false,
      },
      scale: {
        mode: Phaser.Scale.RESIZE,
        width: '100%',
        height: '100%',
      },
      scene,
    });
    // Keep the online simulation running while the document is in the background.
    game.events.off(Phaser.Core.Events.HIDDEN);

    return () => {
      sceneRef.current = null;
      onSceneChange?.(null);
      game.destroy(true);
    };
  }, [onSceneChange, onSpaceshipEngineChange, onTargetDirectionSelected]);

  const navigateTo = (result: SearchResult) => {
    setContextMenu(null);
    setBodyDetails(null);
    sceneRef.current?.clearSelectedBodyDetails();
    sceneRef.current?.navigateTo(result.name, result.navigationZoom);
  };

  const recenterOnSpaceship = () => {
    setContextMenu(null);
    setBodyDetails(null);
    sceneRef.current?.clearSelectedBodyDetails();
    sceneRef.current?.recenterOnSpaceship();
  };

  const dismissBodyDetails = () => {
    setBodyDetails(null);
    sceneRef.current?.clearSelectedBodyDetails();
  };

  const toggleAlwaysVisible = () => {
    if (!contextMenu) return;

    sceneRef.current?.toggleAlwaysVisible(contextMenu.name);
    setContextMenu(null);
  };

  return (
    <section className={style.container} aria-label="Planet navigation map">
      <div className={style.gameHost} ref={gameHostRef} />
      {worldLoadState !== 'ready' && (
        <div
          className={style.worldLoadingOverlay}
          role={worldLoadState === 'error' ? 'alert' : 'status'}
          aria-live="polite"
        >
          {worldLoadState === 'loading' ? (
            <>
              <span className={style.worldLoadingSpinner} aria-hidden="true" />
              <span>Loading star systems…</span>
            </>
          ) : (
            <span>Unable to load star systems.</span>
          )}
        </div>
      )}
      {isWorldViewportLoading && worldLoadState === 'ready' && (
        <div
          className={style.worldViewportLoadingIndicator}
          role="status"
          aria-live="polite"
        >
          <span className={style.worldLoadingSpinner} aria-hidden="true" />
          <span>Loading systems</span>
        </div>
      )}
      {isSearchOpen && (
        <SearchDialog
          onClose={onCloseSearch ?? (() => {})}
          onSelect={navigateTo}
        />
      )}
      {proximityTelemetry && (
        <aside
          className={style.proximityTelemetry}
          data-expanded={isProximityExpanded}
          aria-label={`Spaceship telemetry relative to ${proximityTelemetry.bodyName}`}
        >
          <header>
            <div>
              <span>{proximityTelemetry.bodyName}</span>
              <small>{proximityTelemetry.bodyKind} proximity</small>
            </div>
            <button
              className={style.proximityExpandButton}
              type="button"
              aria-expanded={isProximityExpanded}
              aria-label={
                isProximityExpanded
                  ? 'Collapse proximity telemetry'
                  : 'Expand proximity telemetry'
              }
              onClick={() => setIsProximityExpanded((expanded) => !expanded)}
            >
              {isProximityExpanded ? '›' : '‹'}
            </button>
          </header>
          <dl>
            <div>
              <dt>Surface</dt>
              <dd>
                {formatDistance(proximityTelemetry.surfaceDistanceMeters)}
              </dd>
            </div>
            <div>
              <dt>Relative speed</dt>
              <dd>
                {formatSpeed(proximityTelemetry.relativeSpeedMetersPerSecond)}
              </dd>
            </div>
          </dl>
        </aside>
      )}
      {contextMenu && (
        <BodyContextMenu
          request={contextMenu}
          onDismiss={() => setContextMenu(null)}
          onToggleAlwaysVisible={toggleAlwaysVisible}
        />
      )}
      {bodyDetails && (
        <BodyDetailsDialog
          details={bodyDetails}
          onDismiss={dismissBodyDetails}
        />
      )}
      {isSelectingTargetDirection && targetPreview && (
        <output
          className={style.targetDirectionPreview}
          style={{ left: targetPreview.x - 8, top: targetPreview.y + 12 }}
        >
          <span>{formatAngle(Math.round(targetPreview.angle) % 360)}</span>
          <span>{formatDistance(targetPreview.distance)}</span>
        </output>
      )}
      <div className={style.navigationControls}>
        <label className={style.asteroidToggle}>
          <input
            type="checkbox"
            checked={showAsteroids}
            onChange={(event) => setShowAsteroids(event.currentTarget.checked)}
          />
          <span>Asteroids</span>
        </label>
        <button
          className={style.recenterButton}
          type="button"
          onClick={recenterOnSpaceship}
        >
          Recenter on spaceship
        </button>
        <div className={style.zoomMeter}>
          <label htmlFor="navigator-zoom-level">
            {SCALE_WIDTH_PX} px ={' '}
            <output>{formatScaleDistance(zoomLevel)}</output>
          </label>
          <meter
            id="navigator-zoom-level"
            min={Math.log10(MIN_ZOOM)}
            max={Math.log10(MAX_ZOOM)}
            value={Math.log10(zoomLevel)}
          />
        </div>
      </div>
    </section>
  );
}
