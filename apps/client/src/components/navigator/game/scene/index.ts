import Phaser from 'phaser';
import {
  advanceWorld,
  getSpaceshipActiveThrustVector,
  getBodyWorldVelocity,
  getSpaceshipAttachedBodyName,
  getSpaceshipMotionState,
  getSpaceshipWorldVelocity,
  isSpaceshipEngineRunning,
  refreshWorldViewport,
  setActiveWorldBodyNames,
  setSpaceshipTargetDirection,
  setSpaceshipHeading,
  startSpaceshipThrustersFeature,
  stopSpaceshipActiveFeature,
  spaceshipState,
  WORLD_VIEWPORT_REFRESH_INTERVAL_MS,
} from '@store';
import type { Planet as PlanetData, Star as StarData } from '@repo/types';
import {
  formatLightDistance,
  formatSiValue,
  formatSpeed,
} from '../../../../utils';
import {
  DEFAULT_RENDER_ORIGIN_NAME,
  getRenderOriginWorldPosition,
  getRenderPositionFromOrigin,
  getWorldPositionFromRenderPosition,
  offsetRenderOrigin,
  setRenderOriginName,
} from '../get-render-position';
import {
  getPlanetPatternTextureKey,
  Planet,
  PLANET_PATTERN_TEXTURE_SIZE,
  PLANET_PATTERN_VARIANT_COUNT,
} from '../planet';
import { SPACESHIP_TEXTURE_KEY, type Spaceship } from '../spaceship';
import {
  getStarPatternTextureKey,
  Star,
  STAR_PATTERN_TEXTURE_SIZE,
  STAR_PATTERN_VARIANT_COUNT,
} from '../star';
import { configureCamera, isInsideWorld } from './configure-camera';
import { configureInput, MAX_ZOOM, MIN_ZOOM } from './configure-input';
import { drawVisibleWorld } from './draw-visible-world';
import { renderWorld } from './render-world';

type RenderedBodyPosition = {
  x: number;
  y: number;
};

export type BodyContextMenuRequest = {
  x: number;
  y: number;
  name: string;
  kind: 'Planet' | 'Star';
  alwaysVisible: boolean;
};

export type BodyDetailsRequest =
  | {
      kind: 'Planet';
      body: PlanetData;
      systemName: string;
      velocity: { x: number; y: number };
    }
  | {
      kind: 'Star';
      body: StarData;
      systemName: string;
      velocity: { x: number; y: number };
    };

export type TargetDirectionPreview = {
  x: number;
  y: number;
  angle: number;
  distance: number;
};

const VIEWPORT_LABEL_OBJECT_LIMIT = 20;
type ViewportLabelMode = 'force' | 'suppress' | 'zoom';
type NameLabelCandidate = {
  priority: number;
  bounds: Phaser.Geom.Rectangle;
  hide: () => void;
};
type RulerPoint = {
  x: number;
  y: number;
};
const BLACK_HOLE_VIEWPORT_RADIUS_PX = 16;
const NAME_LABEL_COLLISION_PADDING_PX = 3;
const MEASUREMENT_ARROW_LENGTH_PX = 72;
const MEASUREMENT_ARROW_HEAD_PX = 7;
const MEASUREMENT_ARROW_GAP_PX = 8;
const MEASUREMENT_ARROW_COLOR = 0x22d3ee;
const RULER_COLOR = 0xfbbf24;
const RULER_POINT_RADIUS_PX = 4;
const RULER_LABEL_OFFSET_PX = 10;
const PREDICTION_COLOR = 0xa78bfa;
const ENGINE_START_RESPONSE_TIMEOUT_MS = 5_000;
const WORLD_VIEWPORT_REQUEST_DEBOUNCE_MS = 500;

function isCanceledRequest(error: unknown) {
  if (!error || typeof error !== 'object') return false;

  const maybeCanceled = error as { code?: unknown; name?: unknown };
  return (
    maybeCanceled.code === 'ERR_CANCELED' ||
    maybeCanceled.name === 'CanceledError' ||
    maybeCanceled.name === 'AbortError'
  );
}

export class Scene extends Phaser.Scene {
  protected dragging = false;
  protected lastPointer = new Phaser.Math.Vector2();
  protected readonly onZoomChange?: (zoom: number) => void;
  protected readonly onBodyContextMenu?: (
    request: BodyContextMenuRequest,
  ) => void;
  protected readonly onBodyDetails?: (request: BodyDetailsRequest) => void;
  protected readonly onSpaceshipTurnChange?: (
    remainingDegrees: number,
    isTurning: boolean,
  ) => void;
  protected readonly onSpaceshipEngineChange?: (
    isRunning: boolean,
    speed: number,
  ) => void;
  protected readonly onTargetDirectionPreview?: (
    preview?: TargetDirectionPreview,
  ) => void;
  protected readonly onTargetDirectionSelected?: () => void;
  readonly onWorldLoadComplete?: (error?: unknown) => void;
  protected readonly onWorldViewportLoadingChange?: (
    isLoading: boolean,
  ) => void;
  protected planetData: PlanetData[] = [];
  protected starData: StarData[] = [];
  protected planets: Planet[] = [];
  protected stars: Star[] = [];
  protected spaceship?: Spaceship;
  protected grid?: Phaser.GameObjects.Graphics;
  private measurementGraphics?: Phaser.GameObjects.Graphics;
  private rulerGraphics?: Phaser.GameObjects.Graphics;
  private rulerLabel?: Phaser.GameObjects.Text;
  private predictionGraphics?: Phaser.GameObjects.Graphics;
  private readonly measurementLabels = new Map<
    string,
    Phaser.GameObjects.Text
  >();
  private measuringActive = false;
  private rulerActive = false;
  private rulerStartPoint?: RulerPoint;
  private rulerEndPoint?: RulerPoint;
  private rulerPreviewPoint?: RulerPoint;
  private predictionSeconds?: number;
  protected lastViewportKey = '';
  protected unsubscribeFromWorld?: () => void;
  private readonly planetDataByName = new Map<string, PlanetData>();
  private readonly starDataByName = new Map<string, StarData>();
  private readonly planetByName = new Map<string, Planet>();
  private readonly starByName = new Map<string, Star>();
  private cameraLockedBodyName?: string;
  private lastReportedZoom = Number.NaN;
  private lastReportedTurnDegrees?: number;
  private lastReportedTurnState = false;
  private lastActiveBodiesViewportKey = '';
  private lastVisibilityViewportKey = '';
  private lastViewportLabelMode?: ViewportLabelMode;
  private spaceshipTurn?: Phaser.Tweens.Tween;
  private spaceshipEngineRunning = false;
  private engineStartPendingUntil = 0;
  private selectingTargetDirection = false;
  private viewportRefreshTimer?: number;
  private viewportRefreshDebounceTimer?: number;
  private viewportRefreshPromise?: Promise<unknown>;
  private viewportRefreshAbortController?: AbortController;
  private hasPendingViewportRefresh = false;
  private readonly alwaysVisibleBodies = new Set<string>();

  constructor(
    onZoomChange?: (zoom: number) => void,
    onBodyContextMenu?: (request: BodyContextMenuRequest) => void,
    onBodyDetails?: (request: BodyDetailsRequest) => void,
    onSpaceshipTurnChange?: (
      remainingDegrees: number,
      isTurning: boolean,
    ) => void,
    onSpaceshipEngineChange?: (isRunning: boolean, speed: number) => void,
    onTargetDirectionPreview?: (preview?: TargetDirectionPreview) => void,
    onTargetDirectionSelected?: () => void,
    onWorldLoadComplete?: (error?: unknown) => void,
    onWorldViewportLoadingChange?: (isLoading: boolean) => void,
  ) {
    super('navigation');
    this.onZoomChange = onZoomChange;
    this.onBodyContextMenu = onBodyContextMenu;
    this.onBodyDetails = onBodyDetails;
    this.onSpaceshipTurnChange = onSpaceshipTurnChange;
    this.onSpaceshipEngineChange = onSpaceshipEngineChange;
    this.onTargetDirectionPreview = onTargetDirectionPreview;
    this.onTargetDirectionSelected = onTargetDirectionSelected;
    this.onWorldLoadComplete = onWorldLoadComplete;
    this.onWorldViewportLoadingChange = onWorldViewportLoadingChange;
  }

  protected configureCamera = configureCamera;
  protected configureInput = configureInput;
  protected drawVisibleWorld = drawVisibleWorld;
  protected renderWorld = renderWorld;

  preload() {
    this.load.svg(SPACESHIP_TEXTURE_KEY, '/spaceship.svg');
    for (
      let variant = 0;
      variant < PLANET_PATTERN_VARIANT_COUNT;
      variant += 1
    ) {
      this.load.svg(
        getPlanetPatternTextureKey(variant),
        `/planets/${variant}.svg`,
        {
          width: PLANET_PATTERN_TEXTURE_SIZE,
          height: PLANET_PATTERN_TEXTURE_SIZE,
        },
      );
    }
    for (let variant = 0; variant < STAR_PATTERN_VARIANT_COUNT; variant += 1) {
      this.load.svg(
        getStarPatternTextureKey(variant),
        `/stars/${variant}.svg`,
        {
          width: STAR_PATTERN_TEXTURE_SIZE,
          height: STAR_PATTERN_TEXTURE_SIZE,
        },
      );
    }
  }

  create() {
    setRenderOriginName(DEFAULT_RENDER_ORIGIN_NAME);
    this.configureCamera();
    this.grid = this.add.graphics().setDepth(-1);
    this.measurementGraphics = this.add.graphics().setDepth(20);
    this.rulerGraphics = this.add.graphics().setDepth(22);
    this.predictionGraphics = this.add.graphics().setDepth(19);
    this.drawVisibleWorld();
    this.configureInput();
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.unsubscribeFromWorld?.();
      this.unsubscribeFromWorld = undefined;
      window.clearInterval(this.viewportRefreshTimer);
      this.viewportRefreshTimer = undefined;
      window.clearTimeout(this.viewportRefreshDebounceTimer);
      this.viewportRefreshDebounceTimer = undefined;
      this.viewportRefreshAbortController?.abort();
      this.viewportRefreshAbortController = undefined;
    });
    void this.renderWorld();
    this.viewportRefreshTimer = window.setInterval(() => {
      this.refreshWorldFromViewportSafely();
    }, WORLD_VIEWPORT_REFRESH_INTERVAL_MS);
  }

  update(time: number, delta: number) {
    this.publishActiveWorldBodies();
    const worldElapsedSeconds = advanceWorld(delta / 1000);
    this.syncWorldPositions();
    this.planets.forEach((planet) => planet.syncRotation(worldElapsedSeconds));
    this.stars.forEach((star) => star.syncRotation(worldElapsedSeconds));
    if (getSpaceshipMotionState() === 'crashed') {
      this.spaceship?.clearTargetDirection();
    }
    const storeEngineRunning = isSpaceshipEngineRunning();
    const engineStartPending =
      this.spaceshipEngineRunning &&
      !storeEngineRunning &&
      time < this.engineStartPendingUntil;
    const spaceshipEngineRunning = storeEngineRunning || engineStartPending;
    if (storeEngineRunning) {
      this.engineStartPendingUntil = 0;
    }
    if (!this.spaceshipEngineRunning && spaceshipEngineRunning) {
      this.spaceshipEngineRunning = true;
      this.spaceship?.setThrustersActive(
        true,
        getSpaceshipActiveThrustVector(),
      );
      this.onSpaceshipEngineChange?.(true, this.getSpaceshipSpeed());
    } else if (this.spaceshipEngineRunning && !spaceshipEngineRunning) {
      this.spaceshipEngineRunning = false;
      this.spaceship?.setThrustersActive(false);
      this.onSpaceshipEngineChange?.(false, this.getSpaceshipSpeed());
    } else if (spaceshipEngineRunning) {
      this.spaceship?.setThrustersActive(
        true,
        getSpaceshipActiveThrustVector(),
      );
    }

    const camera = this.cameras.main;
    this.updateCameraLock();
    this.drawVisibleWorld();
    const zoom = camera.zoom;
    if (zoom !== this.lastReportedZoom) {
      this.lastReportedZoom = zoom;
      this.onZoomChange?.(zoom);
    }

    const visibilityViewportKey = [
      zoom,
      camera.worldView.x,
      camera.worldView.y,
      camera.worldView.width,
      camera.worldView.height,
    ].join(':');
    if (visibilityViewportKey !== this.lastVisibilityViewportKey) {
      this.lastVisibilityViewportKey = visibilityViewportKey;
      this.updateWorldVisibility();
    }
    this.drawMeasurements();
    this.drawRuler();
    this.drawPredictions();
  }

  setZoom(zoom: number) {
    const camera = this.cameras.main;
    const center = camera.midPoint.clone();

    camera.setZoom(zoom);
    camera.centerOn(center.x, center.y);
    this.lastActiveBodiesViewportKey = '';
    this.lastReportedZoom = camera.zoom;
    this.onZoomChange?.(camera.zoom);
    this.updateWorldVisibility();
  }

  setMeasuringActive(active: boolean) {
    this.measuringActive = active;
    if (!active) {
      this.measurementGraphics?.clear();
      this.measurementLabels.forEach((label) => label.setVisible(false));
    }
  }

  setRulerActive(active: boolean) {
    this.rulerActive = active;
    this.game.canvas.style.cursor = active ? 'crosshair' : 'grab';
    if (!active) this.rulerPreviewPoint = undefined;
  }

  setPrediction(active: boolean, seconds: number) {
    this.predictionSeconds =
      active && Number.isFinite(seconds) && seconds > 0 ? seconds : undefined;
    if (!this.predictionSeconds) this.predictionGraphics?.clear();
  }

  isRulerActive() {
    return this.rulerActive;
  }

  previewRulerAt(x: number, y: number) {
    if (!this.rulerActive || !this.rulerStartPoint) return;

    const point = this.getRulerPointAt(x, y);
    this.rulerPreviewPoint = point;
  }

  selectRulerPointAt(x: number, y: number) {
    if (!this.rulerActive) return;

    const point = this.getRulerPointAt(x, y);
    if (!point) return;

    if (!this.rulerStartPoint || this.rulerEndPoint) {
      this.rulerStartPoint = point;
      this.rulerEndPoint = undefined;
      this.rulerPreviewPoint = undefined;
      return;
    }

    this.rulerEndPoint = point;
    this.rulerPreviewPoint = undefined;
  }

  hideRulerPreview() {
    this.rulerPreviewPoint = undefined;
  }

  private getRulerPointAt(x: number, y: number) {
    const worldPoint = this.cameras.main.getWorldPoint(x, y);
    if (!isInsideWorld(worldPoint.x, worldPoint.y)) return undefined;

    return { x: worldPoint.x, y: worldPoint.y };
  }

  private drawRuler() {
    const graphics = this.rulerGraphics;
    if (!graphics) return;

    const start = this.rulerStartPoint;
    const end = this.rulerEndPoint ?? this.rulerPreviewPoint;
    graphics.clear();
    this.rulerLabel?.setVisible(false);
    if (!start || !end) return;

    const camera = this.cameras.main;
    const zoom = camera.zoom;
    const pointRadius = RULER_POINT_RADIUS_PX / zoom;
    const labelOffset = RULER_LABEL_OFFSET_PX / zoom;
    const distance = Math.hypot(end.x - start.x, end.y - start.y);
    const midpointX = (start.x + end.x) / 2;
    const midpointY = (start.y + end.y) / 2;

    graphics
      .lineStyle(1.5 / zoom, RULER_COLOR, this.rulerEndPoint ? 0.95 : 0.55)
      .fillStyle(RULER_COLOR, 0.95)
      .beginPath()
      .moveTo(start.x, start.y)
      .lineTo(end.x, end.y)
      .strokePath()
      .fillCircle(start.x, start.y, pointRadius)
      .fillCircle(end.x, end.y, pointRadius);

    if (!this.rulerLabel) {
      this.rulerLabel = this.add
        .text(0, 0, '', {
          color: '#fde68a',
          fontFamily: 'system-ui, sans-serif',
          fontSize: '12px',
          fontStyle: 'bold',
          stroke: '#050816',
          strokeThickness: 4,
        })
        .setDepth(23)
        .setOrigin(0.5, 1)
        .setResolution(Math.max(2, window.devicePixelRatio));
    }

    this.rulerLabel
      .setText(
        `${formatSiValue(distance, 'm')}\n${formatLightDistance(distance)}`,
      )
      .setPosition(midpointX, midpointY - labelOffset)
      .setScale(1 / zoom)
      .setVisible(true);
  }

  private drawPredictions() {
    const graphics = this.predictionGraphics;
    const seconds = this.predictionSeconds;
    if (!graphics || !seconds) return;

    const zoom = this.cameras.main.zoom;
    graphics.clear();
    graphics.lineStyle(1.5 / zoom, PREDICTION_COLOR, 0.72);

    const drawPrediction = (
      x: number,
      y: number,
      velocity: { x: number; y: number },
    ) => {
      const targetX = x + velocity.x * seconds;
      const targetY = y + velocity.y * seconds;
      const markerRadius = 6 / zoom;
      graphics.lineBetween(x, y, targetX, targetY);
      graphics.strokeCircle(targetX, targetY, markerRadius);
      graphics.lineBetween(
        targetX - markerRadius * 1.5,
        targetY,
        targetX + markerRadius * 1.5,
        targetY,
      );
      graphics.lineBetween(
        targetX,
        targetY - markerRadius * 1.5,
        targetX,
        targetY + markerRadius * 1.5,
      );
    };

    this.planets.forEach((planet) => {
      if (planet.visible) {
        drawPrediction(planet.x, planet.y, getBodyWorldVelocity(planet.name));
      }
    });
    this.stars.forEach((star) => {
      if (star.visible) {
        drawPrediction(star.x, star.y, getBodyWorldVelocity(star.name));
      }
    });
    if (this.spaceship?.visible) {
      drawPrediction(
        this.spaceship.x,
        this.spaceship.y,
        getSpaceshipWorldVelocity(),
      );
    }
  }

  navigateTo(name: string, zoom: number) {
    this.focusOn(name, zoom, true);
  }

  recenterOnSpaceship(animate = true) {
    this.focusOn(DEFAULT_RENDER_ORIGIN_NAME, MAX_ZOOM, animate);
  }

  rotateSpaceship(degrees: number) {
    if (
      !this.spaceship ||
      this.spaceshipTurn ||
      this.spaceshipEngineRunning ||
      !Number.isFinite(degrees) ||
      degrees === 0
    ) {
      return;
    }

    setRenderOriginName(DEFAULT_RENDER_ORIGIN_NAME);
    this.syncWorldPositions();
    this.cameraLockedBodyName = DEFAULT_RENDER_ORIGIN_NAME;
    this.cameras.main.panEffect.reset();
    this.cameras.main.setAngle(0);

    const turn = { heading: this.spaceship.spaceship.heading };
    const targetHeading = turn.heading + degrees;
    this.reportSpaceshipTurn(degrees, true);
    this.spaceshipTurn = this.tweens.add({
      targets: turn,
      heading: targetHeading,
      duration: Math.abs(degrees) * 1_000,
      ease: 'Linear',
      onUpdate: () => {
        setSpaceshipHeading(turn.heading);
        this.reportSpaceshipTurn(targetHeading - turn.heading, true);
        this.updateCameraLock();
      },
      onComplete: () => {
        setSpaceshipHeading(targetHeading);
        this.spaceshipTurn = undefined;
        this.reportSpaceshipTurn(0, false);
      },
    });
  }

  getSpaceshipSpeed() {
    return Number(this.spaceship?.spaceship.speed ?? 0n);
  }

  startThrusters(thrusters: { powerPercent: number; active: boolean }[]) {
    if (
      !this.spaceship ||
      !Array.isArray(thrusters) ||
      thrusters.length === 0
    ) {
      return;
    }

    try {
      startSpaceshipThrustersFeature(thrusters);
    } catch (error) {
      console.error('Failed to start thrusters feature', error);
      return;
    }

    this.spaceshipEngineRunning = true;
    this.engineStartPendingUntil =
      this.time.now + ENGINE_START_RESPONSE_TIMEOUT_MS;
    this.spaceship.setThrustersActive(true, getSpaceshipActiveThrustVector());
    this.onSpaceshipEngineChange?.(true, this.getSpaceshipSpeed());
  }

  stopEngines() {
    if (!this.spaceshipEngineRunning) return;

    try {
      stopSpaceshipActiveFeature();
    } catch (error) {
      console.error('Failed to stop active spaceship feature', error);
      return;
    }

    this.spaceshipEngineRunning = false;
    this.engineStartPendingUntil = 0;
    this.spaceship?.setThrustersActive(false);
    this.onSpaceshipEngineChange?.(false, this.getSpaceshipSpeed());
  }

  private reportSpaceshipTurn(remainingDegrees: number, isTurning: boolean) {
    const displayedDegrees =
      remainingDegrees === 0
        ? 0
        : Math.sign(remainingDegrees) * Math.ceil(Math.abs(remainingDegrees));
    if (
      displayedDegrees === this.lastReportedTurnDegrees &&
      isTurning === this.lastReportedTurnState
    ) {
      return;
    }

    this.lastReportedTurnDegrees = displayedDegrees;
    this.lastReportedTurnState = isTurning;
    this.onSpaceshipTurnChange?.(displayedDegrees, isTurning);
  }

  private focusOn(name: string, zoom: number, animate: boolean) {
    const camera = this.cameras.main;
    const targetZoom = Phaser.Math.Clamp(zoom, MIN_ZOOM, MAX_ZOOM);

    setRenderOriginName(name);
    this.ensureRenderedBody(name);
    this.syncWorldPositions();
    this.cameraLockedBodyName = name;
    this.lastActiveBodiesViewportKey = '';
    camera.setAngle(0);
    if (animate) {
      camera.pan(0, 0, 700, 'Sine.easeInOut', true);
      camera.zoomTo(targetZoom, 700, 'Sine.easeInOut', true);
    } else {
      camera.centerOn(0, 0);
      camera.setZoom(targetZoom);
      this.onZoomChange?.(camera.zoom);
    }
    this.updateWorldVisibility();
  }

  releaseCameraLock() {
    this.cameraLockedBodyName = undefined;
    this.cameras.main.panEffect.reset();
    this.cameras.main.zoomEffect.reset();
  }

  rebaseRenderOriginAtCameraCenter() {
    const camera = this.cameras.main;
    camera.preRender();
    const center = camera.midPoint;

    offsetRenderOrigin(center.x, center.y);
    this.syncWorldPositions();
    camera.centerOn(0, 0);
    this.lastViewportKey = '';
    this.lastActiveBodiesViewportKey = '';
    this.lastVisibilityViewportKey = '';
    this.updateWorldVisibility();
  }

  lockCameraOn(name: string) {
    this.cameraLockedBodyName = name;
    this.updateCameraLock();
  }

  openBodyContextMenuAt(x: number, y: number) {
    const camera = this.cameras.main;
    const star = this.stars
      .toReversed()
      .find((candidate) => candidate.containsScreenPoint(x, y, camera));
    const planet = star
      ? undefined
      : this.planets
          .toReversed()
          .find((candidate) => candidate.containsScreenPoint(x, y, camera));
    const body: PlanetData | StarData | undefined =
      star?.star ?? planet?.planet;
    if (!body) return;

    this.onBodyContextMenu?.({
      name: body.name,
      kind: star ? 'Star' : 'Planet',
      x: Phaser.Math.Clamp(x, 8, Math.max(8, this.scale.width - 230)),
      y: Phaser.Math.Clamp(y, 8, Math.max(8, this.scale.height - 110)),
      alwaysVisible: this.alwaysVisibleBodies.has(body.name),
    });
  }

  openBodyDetailsAt(x: number, y: number) {
    const camera = this.cameras.main;
    const star = this.stars
      .toReversed()
      .find((candidate) => candidate.containsScreenPoint(x, y, camera));
    if (star) {
      this.onBodyDetails?.({
        kind: 'Star',
        body: star.star,
        systemName: star.star.name,
        velocity: getBodyWorldVelocity(star.star.name),
      });
      return;
    }

    const planet = this.planets
      .toReversed()
      .find((candidate) => candidate.containsScreenPoint(x, y, camera));
    if (!planet) return;

    this.onBodyDetails?.({
      kind: 'Planet',
      body: planet.planet,
      systemName: this.getSystemName(planet.planet),
      velocity: getBodyWorldVelocity(planet.planet.name),
    });
  }

  private getSystemName(body: PlanetData | StarData) {
    let systemName = body.name;
    let centerName = body.orbitalCenter;
    const visitedNames = new Set<string>();

    while (centerName && !visitedNames.has(centerName)) {
      visitedNames.add(centerName);
      systemName = centerName;
      centerName =
        this.planetDataByName.get(centerName)?.orbitalCenter ??
        this.starDataByName.get(centerName)?.orbitalCenter ??
        null;
    }

    return systemName;
  }

  setTargetDirectionSelectionActive(active: boolean) {
    this.selectingTargetDirection = active;
    this.game.canvas.style.cursor = active ? 'crosshair' : 'grab';
    if (!active) this.onTargetDirectionPreview?.(undefined);
  }

  private drawMeasurements() {
    const graphics = this.measurementGraphics;
    if (!graphics || !this.measuringActive) return;

    const camera = this.cameras.main;
    const zoom = camera.zoom;
    const displayedNames = new Set<string>();
    graphics.clear();
    graphics.lineStyle(1.5 / zoom, MEASUREMENT_ARROW_COLOR, 0.9);

    const drawMeasurement = (
      name: string,
      x: number,
      y: number,
      radius: number,
      velocity: { x: number; y: number },
    ) => {
      const speed = Math.hypot(velocity.x, velocity.y);
      const angle = speed > 0 ? Math.atan2(velocity.y, velocity.x) : 0;
      const directionX = Math.cos(angle);
      const directionY = Math.sin(angle);
      const startDistance = radius + MEASUREMENT_ARROW_GAP_PX / zoom;
      const endDistance = startDistance + MEASUREMENT_ARROW_LENGTH_PX / zoom;
      const startX = x + directionX * startDistance;
      const startY = y + directionY * startDistance;
      const endX = x + directionX * endDistance;
      const endY = y + directionY * endDistance;
      const headLength = MEASUREMENT_ARROW_HEAD_PX / zoom;

      graphics
        .beginPath()
        .moveTo(startX, startY)
        .lineTo(endX, endY)
        .lineTo(
          endX - Math.cos(angle - Math.PI / 4) * headLength,
          endY - Math.sin(angle - Math.PI / 4) * headLength,
        )
        .moveTo(endX, endY)
        .lineTo(
          endX - Math.cos(angle + Math.PI / 4) * headLength,
          endY - Math.sin(angle + Math.PI / 4) * headLength,
        )
        .strokePath();

      let label = this.measurementLabels.get(name);
      if (!label) {
        label = this.add
          .text(0, 0, '', {
            color: '#67e8f9',
            fontFamily: 'system-ui, sans-serif',
            fontSize: '12px',
            fontStyle: 'bold',
            stroke: '#050816',
            strokeThickness: 3,
          })
          .setDepth(21)
          .setOrigin(0, 0.5)
          .setResolution(Math.max(2, window.devicePixelRatio));
        this.measurementLabels.set(name, label);
      }

      label
        .setText(formatSpeed(speed))
        .setOrigin(directionX < 0 ? 1 : 0, 0.5)
        .setPosition(
          endX + directionX * (MEASUREMENT_ARROW_GAP_PX / zoom),
          endY + directionY * (MEASUREMENT_ARROW_GAP_PX / zoom),
        )
        .setScale(1 / zoom)
        .setVisible(true);
      displayedNames.add(name);
    };

    this.planets.forEach((planet) => {
      if (!planet.visible) return;
      drawMeasurement(
        planet.name,
        planet.x,
        planet.y,
        Number(planet.planet.radius),
        getBodyWorldVelocity(planet.name),
      );
    });
    this.stars.forEach((star) => {
      if (!star.visible) return;
      drawMeasurement(
        star.name,
        star.x,
        star.y,
        Number(star.star.radius),
        getBodyWorldVelocity(star.name),
      );
    });
    if (this.spaceship?.visible) {
      drawMeasurement(
        this.spaceship.name,
        this.spaceship.x,
        this.spaceship.y,
        Number(this.spaceship.spaceship.radius),
        getSpaceshipWorldVelocity(),
      );
    }

    this.measurementLabels.forEach((label, name) => {
      if (!displayedNames.has(name)) label.setVisible(false);
    });
  }

  isTargetDirectionSelectionActive() {
    return this.selectingTargetDirection;
  }

  previewTargetDirectionAt(x: number, y: number) {
    const target = this.getTargetDirectionAt(x, y);
    this.onTargetDirectionPreview?.(target?.preview);
  }

  selectTargetDirectionAt(x: number, y: number) {
    const target = this.getTargetDirectionAt(x, y);
    if (!target) return;

    this.setTargetDirection(target.direction);
    this.setTargetDirectionSelectionActive(false);
    this.onTargetDirectionSelected?.();
  }

  hideTargetDirectionPreview() {
    this.onTargetDirectionPreview?.(undefined);
  }

  setTargetDirection(direction: number) {
    if (!Number.isFinite(direction)) return;

    setSpaceshipTargetDirection(direction);
    this.spaceship?.setTargetDirection(direction);
  }

  private getTargetDirectionAt(x: number, y: number) {
    if (!this.spaceship) return undefined;

    const worldPoint = this.cameras.main.getWorldPoint(x, y);
    if (!isInsideWorld(worldPoint.x, worldPoint.y)) return undefined;

    const deltaX = worldPoint.x - this.spaceship.x;
    const deltaY = worldPoint.y - this.spaceship.y;
    const distance = Math.hypot(deltaX, deltaY);
    if (distance === 0) return undefined;

    const direction = Math.atan2(deltaY, deltaX);
    const velocity = getSpaceshipWorldVelocity();
    const currentDirection = Math.atan2(velocity.y, velocity.x);
    const angleDegrees = ((direction - currentDirection) * 180) / Math.PI;
    const angle = (angleDegrees + 360) % 360;

    return {
      direction,
      preview: { x, y, angle, distance },
    };
  }

  toggleAlwaysVisible(name: string) {
    if (this.alwaysVisibleBodies.has(name)) {
      this.alwaysVisibleBodies.delete(name);
    } else {
      this.alwaysVisibleBodies.add(name);
    }

    this.lastActiveBodiesViewportKey = '';
    this.updateWorldVisibility();
  }

  protected updateWorldVisibility() {
    const camera = this.cameras.main;
    if (!camera) return;
    const viewportLabelMode = this.getViewportLabelMode();
    this.lastViewportLabelMode = viewportLabelMode;

    this.planets.forEach((planet) => {
      planet.setRenderVisibility(
        camera.zoom,
        camera.worldView,
        this.alwaysVisibleBodies.has(planet.planet.name),
        viewportLabelMode,
      );
      planet.setVisible(planet.visible && isInsideWorld(planet.x, planet.y));
    });
    this.stars.forEach((star) => {
      star.setRenderVisibility(
        camera.zoom,
        camera.worldView,
        this.alwaysVisibleBodies.has(star.star.name),
        viewportLabelMode,
      );
      star.setVisible(star.visible && isInsideWorld(star.x, star.y));
    });
    this.spaceship?.setRenderVisibility(camera.zoom, camera.worldView);
    if (this.spaceship) {
      this.snapSpaceshipToSurface();
      this.spaceship.setVisible(
        this.spaceship.intersectsViewport(camera.worldView) &&
          isInsideWorld(this.spaceship.x, this.spaceship.y),
      );
    }
    this.applyNameLabelVisibility(viewportLabelMode);
    if (viewportLabelMode !== 'suppress' || this.alwaysVisibleBodies.size > 1) {
      this.resolveNameLabelCollisions();
    }
  }

  protected syncWorldPositions(bodyNames?: ReadonlySet<string>) {
    const originPosition = getRenderOriginWorldPosition();
    if (!bodyNames) {
      this.planets.forEach((planet) => planet.syncPosition(originPosition));
      this.stars.forEach((star) => star.syncPosition(originPosition));
      this.spaceship?.syncPosition();
      this.snapSpaceshipToSurface();
      return;
    }

    bodyNames.forEach((bodyName) => {
      this.planetByName.get(bodyName)?.syncPosition(originPosition);
      this.starByName.get(bodyName)?.syncPosition(originPosition);
    });
    if (this.spaceship && bodyNames.has(this.spaceship.spaceship.name)) {
      this.spaceship.syncPosition();
    }
    this.snapSpaceshipToSurface();
  }

  private snapSpaceshipToSurface() {
    const referenceName = getSpaceshipAttachedBodyName();
    if (!this.spaceship || !referenceName) return;

    const planet = this.planetByName.get(referenceName);
    if (planet) {
      this.spaceship.snapToSurface(planet, Number(planet.planet.radius));
      return;
    }

    const star = this.starByName.get(referenceName);
    if (star) {
      this.spaceship.snapToSurface(star, Number(star.star.radius));
    }
  }

  protected updateChangedWorldVisibility(bodyNames: ReadonlySet<string>) {
    const camera = this.cameras.main;
    if (!camera) return;
    const viewportLabelMode = this.getViewportLabelMode();
    if (viewportLabelMode !== this.lastViewportLabelMode) {
      this.updateWorldVisibility();
      return;
    }

    bodyNames.forEach((bodyName) => {
      const planet = this.planetByName.get(bodyName);
      if (!planet) return;

      planet.setRenderVisibility(
        camera.zoom,
        camera.worldView,
        this.alwaysVisibleBodies.has(planet.planet.name),
        viewportLabelMode,
      );
      planet.setVisible(planet.visible && isInsideWorld(planet.x, planet.y));
    });
    bodyNames.forEach((bodyName) => {
      const star = this.starByName.get(bodyName);
      if (!star) return;

      star.setRenderVisibility(
        camera.zoom,
        camera.worldView,
        this.alwaysVisibleBodies.has(star.star.name),
        viewportLabelMode,
      );
      star.setVisible(star.visible && isInsideWorld(star.x, star.y));
    });
    if (this.spaceship && bodyNames.has(this.spaceship.spaceship.name)) {
      this.spaceship.setRenderVisibility(camera.zoom, camera.worldView);
      this.snapSpaceshipToSurface();
      this.spaceship.setVisible(
        this.spaceship.intersectsViewport(camera.worldView) &&
          isInsideWorld(this.spaceship.x, this.spaceship.y),
      );
    }
    this.applyNameLabelVisibility(viewportLabelMode);
    if (viewportLabelMode !== 'suppress' || this.alwaysVisibleBodies.size > 1) {
      this.resolveNameLabelCollisions();
    }
  }

  private publishActiveWorldBodies() {
    const camera = this.cameras.main;
    if (!camera) return;

    camera.preRender();

    const viewport = camera.worldView;
    const viewportKey = [
      viewport.x,
      viewport.y,
      viewport.width,
      viewport.height,
      camera.zoom,
      this.alwaysVisibleBodies.size,
    ].join(':');
    if (viewportKey === this.lastActiveBodiesViewportKey) return;

    this.lastActiveBodiesViewportKey = viewportKey;

    const activeBodyNames = new Set<string>();
    const renderedBodyNames = new Set<string>();
    const bodyPositionByName = this.getBodyPositionByName();
    const viewportBodyNames = this.getViewportBodyNames(bodyPositionByName);

    if (viewportBodyNames.length < VIEWPORT_LABEL_OBJECT_LIMIT) {
      viewportBodyNames.forEach((bodyName) => {
        activeBodyNames.add(bodyName);
        renderedBodyNames.add(bodyName);
      });
    }

    this.planetData.forEach((planet) => {
      const position = bodyPositionByName.get(planet.name);
      const radius = this.getPlanetViewportRadius(planet, camera.zoom);
      const canRenderShape =
        this.alwaysVisibleBodies.has(planet.name) ||
        this.shouldRenderPlanetShape(planet, camera.zoom);
      const intersectsViewport =
        position &&
        this.bodyBoundsIntersectViewport(
          position.x,
          position.y,
          radius,
          viewport,
        );
      if (
        this.alwaysVisibleBodies.has(planet.name) ||
        this.bodyMotionAreaIntersectsViewport(
          planet,
          radius,
          viewport,
          camera.zoom,
          bodyPositionByName,
        )
      ) {
        activeBodyNames.add(planet.name);
      }
      if (canRenderShape && intersectsViewport) {
        renderedBodyNames.add(planet.name);
        activeBodyNames.add(planet.name);
      }
    });
    this.starData.forEach((star) => {
      const position = bodyPositionByName.get(star.name);
      if (!position) return;

      if (
        this.alwaysVisibleBodies.has(star.name) ||
        this.bodyBoundsIntersectViewport(
          position.x,
          position.y,
          Number(star.radius),
          viewport,
        )
      ) {
        activeBodyNames.add(star.name);
        renderedBodyNames.add(star.name);
      }
    });
    if (this.cameraLockedBodyName) {
      activeBodyNames.add(this.cameraLockedBodyName);
      renderedBodyNames.add(this.cameraLockedBodyName);
    }
    const attachedBodyName = getSpaceshipAttachedBodyName();
    if (attachedBodyName) activeBodyNames.add(attachedBodyName);
    if (this.spaceship) activeBodyNames.add(this.spaceship.spaceship.name);

    if (this.reconcileRenderedBodies(renderedBodyNames)) {
      this.lastVisibilityViewportKey = '';
    }
    setActiveWorldBodyNames(activeBodyNames);
  }

  private bodyMotionAreaIntersectsViewport(
    body: PlanetData,
    radius: number,
    viewport: Phaser.Geom.Rectangle,
    zoom: number,
    bodyPositionByName: Map<string, RenderedBodyPosition>,
  ) {
    if (body.type !== 'blackhole' && zoom < body.minZoomRenderShape) {
      return false;
    }

    const position = bodyPositionByName.get(body.name);
    if (!position) return false;

    const center = body.orbitalCenter
      ? bodyPositionByName.get(body.orbitalCenter)
      : undefined;
    if (!center) {
      return this.bodyBoundsIntersectViewport(
        position.x,
        position.y,
        radius,
        viewport,
      );
    }

    const orbitalRadius = Phaser.Math.Distance.Between(
      position.x,
      position.y,
      center.x,
      center.y,
    );

    return this.circleIntersectsViewport(
      center.x,
      center.y,
      orbitalRadius + radius,
      viewport,
    );
  }

  private getViewportLabelMode(): ViewportLabelMode {
    const viewportBodyCount = this.getViewportBodyNames(
      this.getBodyPositionByName(),
    ).length;

    if (viewportBodyCount < VIEWPORT_LABEL_OBJECT_LIMIT) return 'force';
    if (viewportBodyCount > VIEWPORT_LABEL_OBJECT_LIMIT) return 'suppress';

    return 'zoom';
  }

  private applyNameLabelVisibility(viewportLabelMode: ViewportLabelMode) {
    const zoom = this.cameras.main.zoom;

    this.stars.forEach((star) => {
      star.setNameLabelVisible(
        star.visible &&
          (this.alwaysVisibleBodies.has(star.star.name) ||
            viewportLabelMode === 'force' ||
            (viewportLabelMode === 'zoom' &&
              zoom >= star.star.renderZoomLevel)),
      );
    });

    this.planets.forEach((planet) => {
      planet.setNameLabelVisible(
        planet.visible &&
          (this.alwaysVisibleBodies.has(planet.planet.name) ||
            viewportLabelMode === 'force' ||
            (viewportLabelMode === 'zoom' &&
              zoom >= planet.planet.renderZoomLevel)),
      );
    });
  }

  private resolveNameLabelCollisions() {
    const camera = this.cameras.main;
    if (!camera) return;

    const candidates = [
      ...this.stars.map((star) => ({
        bodyName: star.star.name,
        priority: 0,
        bounds: star.getLabelScreenBounds(camera),
        hide: () => star.setNameLabelVisible(false),
      })),
      ...this.planets.map((planet) => ({
        bodyName: planet.planet.name,
        priority: this.getPlanetLabelPriority(planet.planet),
        bounds: planet.getLabelScreenBounds(camera),
        hide: () => planet.setNameLabelVisible(false),
      })),
    ]
      .filter(
        (candidate): candidate is NameLabelCandidate & { bodyName: string } =>
          Boolean(candidate.bounds),
      )
      .sort(
        (a, b) =>
          a.priority - b.priority || a.bodyName.localeCompare(b.bodyName),
      );

    const acceptedBounds: Phaser.Geom.Rectangle[] = [];
    candidates.forEach((candidate) => {
      const bounds = this.padScreenBounds(
        candidate.bounds,
        NAME_LABEL_COLLISION_PADDING_PX,
      );
      if (
        acceptedBounds.some((accepted) =>
          Phaser.Geom.Intersects.RectangleToRectangle(bounds, accepted),
        )
      ) {
        candidate.hide();
        return;
      }

      acceptedBounds.push(bounds);
    });
  }

  private padScreenBounds(bounds: Phaser.Geom.Rectangle, padding: number) {
    return new Phaser.Geom.Rectangle(
      bounds.x - padding,
      bounds.y - padding,
      bounds.width + padding * 2,
      bounds.height + padding * 2,
    );
  }

  private getPlanetLabelPriority(body: PlanetData) {
    if (body.type === 'moon') return 2;

    return 1;
  }

  private getViewportBodyNames(
    bodyPositionByName: Map<string, RenderedBodyPosition>,
  ) {
    const camera = this.cameras.main;
    const viewport = camera.worldView;
    const planets = this.planetData
      .filter((body) => {
        if (!this.shouldRenderPlanetShape(body, camera.zoom)) return false;

        const position = bodyPositionByName.get(body.name);
        if (!position) return false;

        return this.bodyBoundsIntersectViewport(
          position.x,
          position.y,
          this.getPlanetViewportRadius(body, camera.zoom),
          viewport,
        );
      })
      .map((body) => body.name);
    const stars = this.starData
      .filter((body) => {
        const position = bodyPositionByName.get(body.name);
        if (!position) return false;

        return this.bodyBoundsIntersectViewport(
          position.x,
          position.y,
          Number(body.radius),
          viewport,
        );
      })
      .map((body) => body.name);

    return [...planets, ...stars];
  }

  private getPlanetViewportRadius(body: PlanetData, zoom: number) {
    const radius = Number(body.radius);
    if (body.type !== 'blackhole') return radius;

    return Math.max(radius, BLACK_HOLE_VIEWPORT_RADIUS_PX / zoom);
  }

  private shouldRenderPlanetShape(body: PlanetData, zoom: number) {
    return body.type === 'blackhole' || zoom >= body.minZoomRenderShape;
  }

  private bodyBoundsIntersectViewport(
    x: number,
    y: number,
    radius: number,
    viewport: Phaser.Geom.Rectangle,
  ) {
    return (
      x + radius >= viewport.left &&
      x - radius <= viewport.right &&
      y + radius >= viewport.top &&
      y - radius <= viewport.bottom
    );
  }

  private circleIntersectsViewport(
    x: number,
    y: number,
    radius: number,
    viewport: Phaser.Geom.Rectangle,
  ) {
    const closestX = Phaser.Math.Clamp(x, viewport.left, viewport.right);
    const closestY = Phaser.Math.Clamp(y, viewport.top, viewport.bottom);
    return (
      Phaser.Math.Distance.Squared(x, y, closestX, closestY) <= radius ** 2
    );
  }

  private getBodyPositionByName() {
    const bodyPositionByName = new Map<string, RenderedBodyPosition>();
    const originPosition = getRenderOriginWorldPosition();
    this.planetData.forEach((planet) => {
      bodyPositionByName.set(
        planet.name,
        getRenderPositionFromOrigin(planet.position, originPosition),
      );
    });
    this.starData.forEach((star) => {
      bodyPositionByName.set(
        star.name,
        getRenderPositionFromOrigin(star.position, originPosition),
      );
    });
    if (this.spaceship) {
      bodyPositionByName.set(this.spaceship.name, this.spaceship);
    }
    return bodyPositionByName;
  }

  protected rebuildRenderedBodyIndexes() {
    this.planetByName.clear();
    this.starByName.clear();
    this.planets.forEach((planet) =>
      this.planetByName.set(planet.name, planet),
    );
    this.stars.forEach((star) => this.starByName.set(star.name, star));
  }

  protected setWorldBodyData(planets: PlanetData[], stars: StarData[]) {
    const planetDataByName = new Map(
      planets.map((planet) => [planet.name, planet]),
    );
    const starDataByName = new Map(stars.map((star) => [star.name, star]));

    this.planets = this.planets.filter((planet) => {
      const nextPlanet = planetDataByName.get(planet.name);
      if (nextPlanet) {
        Object.assign(planet.planet, nextPlanet);
        planetDataByName.set(planet.name, planet.planet);
        return true;
      }

      this.planetByName.delete(planet.name);
      planet.destroy();
      return false;
    });
    this.stars = this.stars.filter((star) => {
      const nextStar = starDataByName.get(star.name);
      if (nextStar) {
        Object.assign(star.star, nextStar);
        starDataByName.set(star.name, star.star);
        return true;
      }

      this.starByName.delete(star.name);
      star.destroy();
      return false;
    });

    this.planetData = planets.map(
      (planet) => planetDataByName.get(planet.name) ?? planet,
    );
    this.starData = stars.map((star) => starDataByName.get(star.name) ?? star);
    this.planetDataByName.clear();
    this.starDataByName.clear();
    this.planetData.forEach((planet) =>
      this.planetDataByName.set(planet.name, planet),
    );
    this.starData.forEach((star) => this.starDataByName.set(star.name, star));
  }

  async refreshWorldFromViewport(signal?: AbortSignal) {
    const camera = this.cameras.main;
    camera.preRender();
    const viewport = camera.worldView;
    const topLeft = getWorldPositionFromRenderPosition(
      viewport.left,
      viewport.top,
    );
    const bottomRight = getWorldPositionFromRenderPosition(
      viewport.right,
      viewport.bottom,
    );
    const requiredBodyNames = spaceshipState.position.relativeTo
      ? [spaceshipState.position.relativeTo]
      : undefined;
    this.onWorldViewportLoadingChange?.(true);
    const world = await refreshWorldViewport({
      x1:
        topLeft.x < bottomRight.x
          ? topLeft.x.toString()
          : bottomRight.x.toString(),
      x2:
        topLeft.x > bottomRight.x
          ? topLeft.x.toString()
          : bottomRight.x.toString(),
      y1:
        topLeft.y < bottomRight.y
          ? topLeft.y.toString()
          : bottomRight.y.toString(),
      y2:
        topLeft.y > bottomRight.y
          ? topLeft.y.toString()
          : bottomRight.y.toString(),
      zoom: camera.zoom,
      requiredBodyNames,
      signal,
    }).finally(() => {
      this.onWorldViewportLoadingChange?.(false);
    });

    if (signal?.aborted || !this.sys.isActive()) return world;

    this.setWorldBodyData(world.planets, world.stars);
    this.syncWorldPositions();
    this.lastActiveBodiesViewportKey = '';
    this.lastVisibilityViewportKey = '';
    this.updateWorldVisibility();
    return world;
  }

  queueViewportWorldRefresh(delayMs = WORLD_VIEWPORT_REQUEST_DEBOUNCE_MS) {
    window.clearTimeout(this.viewportRefreshDebounceTimer);
    this.viewportRefreshDebounceTimer = window.setTimeout(() => {
      this.viewportRefreshDebounceTimer = undefined;
      this.refreshWorldFromViewportSafely();
    }, delayMs);
  }

  private refreshWorldFromViewportSafely() {
    if (this.viewportRefreshPromise) {
      this.hasPendingViewportRefresh = true;
      this.viewportRefreshAbortController?.abort();
      return;
    }

    const abortController = new AbortController();
    this.viewportRefreshAbortController = abortController;
    this.viewportRefreshPromise ??= this.refreshWorldFromViewport(
      abortController.signal,
    )
      .catch((error: unknown) => {
        if (isCanceledRequest(error)) return;

        console.error('Failed to refresh viewport world data', error);
        this.onWorldLoadComplete?.(error);
      })
      .finally(() => {
        if (this.viewportRefreshAbortController === abortController) {
          this.viewportRefreshAbortController = undefined;
        }
        this.viewportRefreshPromise = undefined;
        if (this.hasPendingViewportRefresh) {
          this.hasPendingViewportRefresh = false;
          this.queueViewportWorldRefresh();
        }
      });
  }

  private reconcileRenderedBodies(activeBodyNames: ReadonlySet<string>) {
    let changed = false;
    activeBodyNames.forEach((bodyName) => {
      changed = this.ensureRenderedBody(bodyName) || changed;
    });

    this.planets = this.planets.filter((planet) => {
      if (
        activeBodyNames.has(planet.planet.name) ||
        this.alwaysVisibleBodies.has(planet.planet.name)
      ) {
        return true;
      }

      this.planetByName.delete(planet.planet.name);
      planet.destroy();
      changed = true;
      return false;
    });

    this.stars = this.stars.filter((star) => {
      if (
        activeBodyNames.has(star.star.name) ||
        this.alwaysVisibleBodies.has(star.star.name)
      ) {
        return true;
      }

      this.starByName.delete(star.star.name);
      star.destroy();
      changed = true;
      return false;
    });

    return changed;
  }

  private ensureRenderedBody(name: string) {
    if (this.planetByName.has(name) || this.starByName.has(name)) return false;

    const planetData = this.planetDataByName.get(name);
    if (planetData) {
      const planet = new Planet(this, planetData);
      this.planets.push(planet);
      this.planetByName.set(planet.name, planet);
      return true;
    }

    const starData = this.starDataByName.get(name);
    if (starData) {
      const star = new Star(this, starData);
      this.stars.push(star);
      this.starByName.set(star.name, star);
      return true;
    }

    return false;
  }

  private updateCameraLock() {
    if (!this.cameraLockedBodyName) return;

    const camera = this.cameras.main;
    if (camera.panEffect.isRunning) return;

    const body =
      this.planetByName.get(this.cameraLockedBodyName) ??
      this.starByName.get(this.cameraLockedBodyName) ??
      (this.spaceship?.name === this.cameraLockedBodyName
        ? this.spaceship
        : undefined);
    if (!body) return;

    camera.centerOn(body.x, body.y);
  }
}
