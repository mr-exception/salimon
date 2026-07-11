import { useEffect, useRef, useState } from 'react';
import Phaser from 'phaser';
import {
  getSpaceshipProximityTelemetry,
  subscribeToWorld,
  useInventory,
  type SpaceshipProximityTelemetry,
} from '@store';
import { INVENTORY_MATERIALS } from '@repo/types';
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
const INVENTORY_MATERIAL_LABELS = {
  iron: 'Iron',
  silicates: 'Silicates',
  ice: 'Ice',
  silver: 'Silver',
  carbon: 'Carbon',
  gold: 'Gold',
  hydrogen: 'Hydrogen',
  nitrogen: 'Nitrogen',
} as const;

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
  const inventory = useInventory();

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
      <aside className={style.inventoryPanel} aria-label="Mined materials">
        <header>
          <span>Inventory</span>
          <small>Mined materials</small>
        </header>
        <dl>
          {INVENTORY_MATERIALS.map((material) => (
            <div key={material}>
              <dt>{INVENTORY_MATERIAL_LABELS[material]}</dt>
              <dd>{inventory[material]}</dd>
            </div>
          ))}
        </dl>
      </aside>
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
