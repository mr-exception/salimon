import { useEffect, useRef, useState } from 'react';
import Phaser from 'phaser';
import {
  getSpaceshipProximityTelemetry,
  subscribeToWorld,
  type SpaceshipProximityTelemetry,
} from '@store';
import type { World } from '@repo/types';
import {
  formatAngle,
  formatDistance,
  formatSiValue,
  formatSpeed,
} from '../../utils';
import { BodyContextMenu } from './body-context-menu';
import {
  Scene,
  type BodyContextMenuRequest,
  type BodyDetailsRequest,
  type TargetDirectionPreview,
} from './game/scene';
import { MAX_ZOOM, MIN_ZOOM } from './game/scene/configure-input';
import { SearchPanel, type SearchResult } from './search-panel';
import style from './style.module.css';

const SCALE_WIDTH_PX = 200;
const TELEMETRY_UPDATE_INTERVAL_MS = 250;

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

function getBodyDetailsTitle(details: BodyDetailsRequest) {
  return details.body.name;
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
            <dt>Current speed</dt>
            <dd>{formatSpeed(velocitySpeed)}</dd>
          </div>
          <div>
            <dt>Direction</dt>
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
        </dl>
      </section>
    </>
  );
}

type NavigatorProps = {
  isMeasuring?: boolean;
  isSelectingTargetDirection?: boolean;
  onSceneChange?: (scene: Scene | null) => void;
  onSpaceshipEngineChange?: (isRunning: boolean) => void;
  onTargetDirectionSelected?: () => void;
};

export function Navigator({
  isMeasuring = false,
  isSelectingTargetDirection = false,
  onSceneChange,
  onSpaceshipEngineChange,
  onTargetDirectionSelected,
}: NavigatorProps) {
  const gameHostRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<Scene>(null);
  const [zoomLevel, setZoomLevel] = useState(1);
  const [world, setWorld] = useState<World>({
    planets: [],
    stars: [],
  });
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

  useEffect(() => {
    sceneRef.current?.setMeasuringActive(isMeasuring);
  }, [isMeasuring]);

  useEffect(() => {
    const updateTelemetry = () => {
      setProximityTelemetry(getSpaceshipProximityTelemetry());
    };

    updateTelemetry();
    const telemetryTimer = window.setInterval(
      updateTelemetry,
      TELEMETRY_UPDATE_INTERVAL_MS,
    );
    const unsubscribeFromWorld = subscribeToWorld((updatedWorld) => {
      setWorld({
        planets: updatedWorld.planets,
        stars: updatedWorld.stars,
      });
      updateTelemetry();
    });

    return () => {
      window.clearInterval(telemetryTimer);
      unsubscribeFromWorld();
    };
  }, []);

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
      physics: {
        default: 'matter',
        matter: {
          gravity: { x: 0, y: 0 },
        },
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

  const navigateTo = ({ body }: SearchResult) => {
    setContextMenu(null);
    setBodyDetails(null);
    sceneRef.current?.navigateTo(body.name, body.renderZoomLevel * 10);
  };

  const recenterOnSpaceship = () => {
    setContextMenu(null);
    setBodyDetails(null);
    sceneRef.current?.recenterOnSpaceship();
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
      <SearchPanel
        planets={world.planets}
        stars={world.stars}
        onSelect={navigateTo}
      />
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
          onDismiss={() => setBodyDetails(null)}
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
