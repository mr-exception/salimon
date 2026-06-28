import { useEffect, useRef, useState } from 'react';
import Phaser from 'phaser';
import {
  BASE_SPACESHIP_CONFIG,
  getSpaceshipProximityTelemetry,
  loadWorld,
  setSpaceshipTargetFallingSpeed,
  startSpaceshipFallingSpeedControl,
  stopSpaceshipFallingSpeedControl,
  subscribeToWorld,
  type SpaceshipProximityTelemetry,
  useSpaceshipFallingSpeedControl,
  useSetTimeSpeed,
  useTimeSpeed,
} from '@store';
import type { World } from '@types';
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
  isSelectingTargetDirection?: boolean;
  onSceneChange?: (scene: Scene | null) => void;
  onSpaceshipEngineChange?: (isRunning: boolean) => void;
  onTargetDirectionSelected?: () => void;
};

export function Navigator({
  isSelectingTargetDirection = false,
  onSceneChange,
  onSpaceshipEngineChange,
  onTargetDirectionSelected,
}: NavigatorProps) {
  const gameHostRef = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<Scene>(null);
  const timeSpeed = useTimeSpeed();
  const setTimeSpeed = useSetTimeSpeed();
  const fallingSpeedControl = useSpaceshipFallingSpeedControl();
  const [zoomLevel, setZoomLevel] = useState(1);
  const [world, setWorld] = useState<World>({
    planets: [],
    stars: [],
  });
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
    let lastUpdate = 0;
    const updateTelemetry = () => {
      const now = performance.now();
      if (now - lastUpdate < TELEMETRY_UPDATE_INTERVAL_MS) return;

      lastUpdate = now;
      setProximityTelemetry(getSpaceshipProximityTelemetry());
    };

    updateTelemetry();
    return subscribeToWorld(updateTelemetry);
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
    );
    sceneRef.current = scene;
    onSceneChange?.(scene);
    const game = new Phaser.Game({
      type: Phaser.AUTO,
      parent: gameHostRef.current,
      backgroundColor: '#050816',
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

    void loadWorld()
      .then((loadedWorld) => {
        setWorld(loadedWorld);
        setProximityTelemetry(getSpaceshipProximityTelemetry());
      })
      .catch(() => {
        // The scene reports world-loading errors; keep search unavailable.
      });

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
      <div className={style.timeControls}>
        <span>Time speed</span>
        <div
          className={style.timeSpeedButtons}
          role="group"
          aria-label="Time speed"
        >
          {[1, 10, 100].map((speed) => (
            <button
              key={speed}
              type="button"
              aria-pressed={timeSpeed === speed}
              onClick={() => setTimeSpeed(speed)}
            >
              ×{speed}
            </button>
          ))}
        </div>
      </div>
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
