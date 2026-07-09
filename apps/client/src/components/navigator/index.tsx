import { useEffect, useRef, useState } from 'react';
import Phaser from 'phaser';
import {
  BASE_SPACESHIP_CONFIG,
  getSpaceshipProximityTelemetry,
  setSpaceshipTargetFallingSpeed,
  startSpaceshipFallingSpeedControl,
  stopSpaceshipFallingSpeedControl,
  subscribeToWorld,
  type SpaceshipProximityTelemetry,
  useSpaceshipFallingSpeedControl,
} from '@store';
import type { World } from '@repo/types';
import { formatAngle, formatDistance, formatSpeed } from '../../utils';
import { BodyContextMenu } from './body-context-menu';
import {
  Scene,
  type BodyContextMenuRequest,
  type TargetDirectionPreview,
} from './game/scene';
import { MAX_ZOOM, MIN_ZOOM } from './game/scene/configure-input';
import { SearchPanel, type SearchResult } from './search-panel';
import style from './style.module.css';

const SCALE_WIDTH_PX = 200;
const TELEMETRY_UPDATE_INTERVAL_MS = 100;

function formatScaleDistance(zoom: number) {
  return formatDistance(SCALE_WIDTH_PX / zoom);
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
  const fallingSpeedControl = useSpaceshipFallingSpeedControl();
  const [zoomLevel, setZoomLevel] = useState(1);
  const [world, setWorld] = useState<World>({
    planets: [],
    stars: [],
  });
  const [worldLoadState, setWorldLoadState] = useState<
    'loading' | 'ready' | 'error'
  >('loading');
  const [contextMenu, setContextMenu] = useState<BodyContextMenuRequest | null>(
    null,
  );
  const [targetPreview, setTargetPreview] =
    useState<TargetDirectionPreview | null>(null);
  const [proximityTelemetry, setProximityTelemetry] =
    useState<SpaceshipProximityTelemetry>();
  const [isProximityExpanded, setIsProximityExpanded] = useState(false);
  const [pendingFallingSpeed, setPendingFallingSpeed] = useState(
    fallingSpeedControl.targetSpeedMetersPerSecond,
  );

  useEffect(() => {
    sceneRef.current?.setMeasuringActive(isMeasuring);
  }, [isMeasuring]);

  useEffect(() => {
    let lastUpdate = 0;
    const updateTelemetry = () => {
      const now = performance.now();
      if (now - lastUpdate < TELEMETRY_UPDATE_INTERVAL_MS) return;

      lastUpdate = now;
      setProximityTelemetry(getSpaceshipProximityTelemetry());
    };

    updateTelemetry();
    return subscribeToWorld((updatedWorld) => {
      setWorld({
        planets: updatedWorld.planets,
        stars: updatedWorld.stars,
      });
      updateTelemetry();
    });
  }, []);

  useEffect(() => {
    if (!gameHostRef.current) return;

    const scene = new Scene(
      setZoomLevel,
      setContextMenu,
      undefined,
      (engineIsRunning) => onSpaceshipEngineChange?.(engineIsRunning),
      (preview) => setTargetPreview(preview ?? null),
      onTargetDirectionSelected,
      (error) => setWorldLoadState(error ? 'error' : 'ready'),
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
    sceneRef.current?.navigateTo(body.name, body.renderZoomLevel * 10);
  };

  const recenterOnSpaceship = () => {
    setContextMenu(null);
    sceneRef.current?.recenterOnSpaceship();
  };

  const toggleAlwaysVisible = () => {
    if (!contextMenu) return;

    sceneRef.current?.toggleAlwaysVisible(contextMenu.name);
    setContextMenu(null);
  };

  const toggleFallingSpeedControl = () => {
    if (fallingSpeedControl.active) {
      stopSpaceshipFallingSpeedControl();
      setPendingFallingSpeed(
        BASE_SPACESHIP_CONFIG.crashVelocityThresholdMetersPerSecond,
      );
      return;
    }
    if (!proximityTelemetry) return;
    startSpaceshipFallingSpeedControl(
      proximityTelemetry.bodyName,
      pendingFallingSpeed,
    );
  };

  const adjustFallingSpeed = (difference: number) => {
    const targetSpeed = Math.max(
      0,
      fallingSpeedControl.targetSpeedMetersPerSecond + difference,
    );
    setSpaceshipTargetFallingSpeed(targetSpeed);
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
                  ? 'Collapse falling speed controls'
                  : 'Expand falling speed controls'
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
          {isProximityExpanded && (
            <div className={style.fallingSpeedControls}>
              <label htmlFor="target-falling-speed">
                <span>Target falling speed</span>
                <span className={style.fallingSpeedInput}>
                  <input
                    id="target-falling-speed"
                    type="number"
                    min="0"
                    step="1"
                    disabled={fallingSpeedControl.active}
                    value={
                      fallingSpeedControl.active
                        ? fallingSpeedControl.targetSpeedMetersPerSecond
                        : pendingFallingSpeed
                    }
                    onChange={(event) => {
                      const speed = event.currentTarget.valueAsNumber;
                      if (Number.isFinite(speed)) {
                        setPendingFallingSpeed(Math.max(0, speed));
                      }
                    }}
                  />
                  <span>m/s</span>
                </span>
              </label>
              {fallingSpeedControl.active && (
                <div
                  className={style.fallingSpeedAdjustments}
                  aria-label="Adjust target falling speed"
                >
                  {[10, 25, 50, 100].map((amount) => (
                    <div key={amount}>
                      <button
                        type="button"
                        onClick={() => adjustFallingSpeed(-amount)}
                      >
                        −{amount}
                      </button>
                      <button
                        type="button"
                        onClick={() => adjustFallingSpeed(amount)}
                      >
                        +{amount}
                      </button>
                    </div>
                  ))}
                </div>
              )}
              <button
                className={style.fallingSpeedToggle}
                data-active={fallingSpeedControl.active}
                type="button"
                onClick={toggleFallingSpeedControl}
              >
                {fallingSpeedControl.active ? 'Deactivate' : 'Activate'}
              </button>
            </div>
          )}
        </aside>
      )}
      {contextMenu && (
        <BodyContextMenu
          request={contextMenu}
          onDismiss={() => setContextMenu(null)}
          onToggleAlwaysVisible={toggleAlwaysVisible}
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
