import Phaser from 'phaser';
import {
  addInventoryMaterial,
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
  startSpaceshipTargetSpeedFeature,
  stopSpaceshipActiveFeature,
  WORLD_VIEWPORT_REFRESH_INTERVAL_MS,
} from '@store';
import type { Planet as PlanetData, Star as StarData } from '@repo/types';
import { formatSpeed } from '../../../../utils';
import {
  DEFAULT_RENDER_ORIGIN_NAME,
  getRenderPosition,
  offsetRenderOrigin,
  setRenderOriginName,
  getWorldPositionFromRenderPosition,
} from '../get-render-position';
import { Asteroid, type AsteroidMaterial } from '../asteroid';
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

export type TargetDirectionPreview = {
  x: number;
  y: number;
  angle: number;
  distance: number;
};

const VIEWPORT_LABEL_OBJECT_LIMIT = 20;
const MEASUREMENT_ARROW_LENGTH_PX = 72;
const MEASUREMENT_ARROW_HEAD_PX = 7;
const MEASUREMENT_ARROW_GAP_PX = 8;
const MEASUREMENT_ARROW_COLOR = 0x22d3ee;
const PREDICTION_COLOR = 0xa78bfa;
const ASTEROID_TARGET_COUNT = 8;
const ASTEROID_SPAWN_INTERVAL_MS = 900;
const ASTEROID_ORBIT_CLEARANCE_FACTOR = 1.05;
const ASTEROID_MIN_SCREEN_RADIUS = 9;
const ASTEROID_MAX_SCREEN_RADIUS = 18;
const ASTEROID_SCREEN_MARGIN = 36;
const ASTEROID_MIN_SCREEN_SPEED = 32;
const ASTEROID_MAX_SCREEN_SPEED = 68;
const ENGINE_START_RESPONSE_TIMEOUT_MS = 5_000;
const ASTEROID_MATERIALS: AsteroidMaterial[] = ['iron', 'silicates', 'ice'];

export class Scene extends Phaser.Scene {
  protected dragging = false;
  protected lastPointer = new Phaser.Math.Vector2();
  protected readonly onZoomChange?: (zoom: number) => void;
  protected readonly onBodyContextMenu?: (
    request: BodyContextMenuRequest,
  ) => void;
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
  protected planetData: PlanetData[] = [];
  protected starData: StarData[] = [];
  protected planets: Planet[] = [];
  protected stars: Star[] = [];
  protected asteroids: Asteroid[] = [];
  protected spaceship?: Spaceship;
  protected grid?: Phaser.GameObjects.Graphics;
  private measurementGraphics?: Phaser.GameObjects.Graphics;
  private predictionGraphics?: Phaser.GameObjects.Graphics;
  private readonly measurementLabels = new Map<
    string,
    Phaser.GameObjects.Text
  >();
  private measuringActive = false;
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
  private lastShowViewportLabels?: boolean;
  private spaceshipTurn?: Phaser.Tweens.Tween;
  private spaceshipEngineRunning = false;
  private engineStartPendingUntil = 0;
  private selectingTargetDirection = false;
  private targetDirection?: number;
  private viewportRefreshTimer?: number;
  private viewportRefreshDebounceTimer?: number;
  private viewportRefreshPromise?: Promise<unknown>;
  private hasPendingViewportRefresh = false;
  private asteroidSpawnElapsedMs = ASTEROID_SPAWN_INTERVAL_MS;
  private readonly alwaysVisibleBodies = new Set<string>();

  constructor(
    onZoomChange?: (zoom: number) => void,
    onBodyContextMenu?: (request: BodyContextMenuRequest) => void,
    onSpaceshipTurnChange?: (
      remainingDegrees: number,
      isTurning: boolean,
    ) => void,
    onSpaceshipEngineChange?: (isRunning: boolean, speed: number) => void,
    onTargetDirectionPreview?: (preview?: TargetDirectionPreview) => void,
    onTargetDirectionSelected?: () => void,
    onWorldLoadComplete?: (error?: unknown) => void,
  ) {
    super('navigation');
    this.onZoomChange = onZoomChange;
    this.onBodyContextMenu = onBodyContextMenu;
    this.onSpaceshipTurnChange = onSpaceshipTurnChange;
    this.onSpaceshipEngineChange = onSpaceshipEngineChange;
    this.onTargetDirectionPreview = onTargetDirectionPreview;
    this.onTargetDirectionSelected = onTargetDirectionSelected;
    this.onWorldLoadComplete = onWorldLoadComplete;
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
    this.predictionGraphics = this.add.graphics().setDepth(19);
    this.drawVisibleWorld();
    this.configureInput();
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.unsubscribeFromWorld?.();
      this.unsubscribeFromWorld = undefined;
      this.clearAsteroids();
      window.clearInterval(this.viewportRefreshTimer);
      this.viewportRefreshTimer = undefined;
      window.clearTimeout(this.viewportRefreshDebounceTimer);
      this.viewportRefreshDebounceTimer = undefined;
    });
    void this.renderWorld();
    this.viewportRefreshTimer = window.setInterval(() => {
      this.refreshWorldFromViewportSafely();
    }, WORLD_VIEWPORT_REFRESH_INTERVAL_MS);
  }

  update(time: number, delta: number) {
    const worldElapsedSeconds = advanceWorld(delta / 1000);
    this.syncWorldPositions();
    this.lastActiveBodiesViewportKey = '';
    this.lastVisibilityViewportKey = '';
    this.publishActiveWorldBodies();
    this.planets.forEach((planet) => planet.syncRotation(worldElapsedSeconds));
    this.stars.forEach((star) => star.syncRotation(worldElapsedSeconds));
    if (getSpaceshipMotionState() === 'crashed') {
      this.spaceship?.clearTargetDirection();
      this.targetDirection = undefined;
    }
    this.updateAsteroids(delta);
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
    this.drawPredictions();
  }

  mineAsteroidAt(x: number, y: number) {
    const camera = this.cameras.main;
    if (!this.isMaxZoomedIn()) return false;

    const worldPoint = camera.getWorldPoint(x, y);
    const asteroid = this.asteroids
      .toReversed()
      .find(
        (candidate) =>
          Phaser.Math.Distance.Between(
            worldPoint.x,
            worldPoint.y,
            candidate.x,
            candidate.y,
          ) <=
          (candidate.radius * 1.35) / camera.zoom,
      );
    if (!asteroid) return false;

    addInventoryMaterial(asteroid.payload.material, asteroid.payload.amount);
    this.asteroids = this.asteroids.filter(
      (candidate) => candidate !== asteroid,
    );
    this.tweens.add({
      targets: asteroid,
      alpha: 0,
      scale: asteroid.scale * 1.8,
      duration: 120,
      ease: 'Quad.easeOut',
      onComplete: () => asteroid.destroy(),
    });
    return true;
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

  setPrediction(active: boolean, seconds: number) {
    this.predictionSeconds =
      active && Number.isFinite(seconds) && seconds > 0 ? seconds : undefined;
    if (!this.predictionSeconds) this.predictionGraphics?.clear();
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

  private updateAsteroids(deltaMs: number) {
    const camera = this.cameras.main;
    const deltaSeconds = deltaMs / 1000;
    const viewport = this.getShipMaxZoomViewport();
    const margin = ASTEROID_SCREEN_MARGIN / MAX_ZOOM;
    const isMaxZoomedIn = this.isMaxZoomedIn();

    this.asteroids.forEach((asteroid) =>
      asteroid.update(deltaSeconds, camera.zoom),
    );
    this.asteroids = this.asteroids.filter((asteroid) => {
      const keep = !asteroid.isPastViewport(viewport, margin);
      if (!keep) asteroid.destroy();
      return keep;
    });

    if (!isMaxZoomedIn || !this.canSpawnAsteroids()) {
      this.asteroidSpawnElapsedMs = ASTEROID_SPAWN_INTERVAL_MS;
      this.clearAsteroids();
      return;
    }

    this.asteroidSpawnElapsedMs += deltaMs;
    if (
      this.asteroids.length >= ASTEROID_TARGET_COUNT ||
      this.asteroidSpawnElapsedMs < ASTEROID_SPAWN_INTERVAL_MS
    ) {
      return;
    }

    this.asteroidSpawnElapsedMs = 0;
    this.spawnAsteroid(viewport);
  }

  private spawnAsteroid(viewport: Phaser.Geom.Rectangle) {
    const direction = this.getAsteroidDriftDirection();
    const perpendicular = new Phaser.Math.Vector2(-direction.y, direction.x);
    const edge = this.getAsteroidSpawnEdge(viewport, direction);
    const x = edge.x + perpendicular.x * edge.crossOffset;
    const y = edge.y + perpendicular.y * edge.crossOffset;
    const radius = Phaser.Math.FloatBetween(
      ASTEROID_MIN_SCREEN_RADIUS,
      ASTEROID_MAX_SCREEN_RADIUS,
    );
    const material = Phaser.Math.RND.pick(ASTEROID_MATERIALS);
    const amount = Math.round(radius * Phaser.Math.FloatBetween(1.6, 3.2));
    const screenSpeed = Phaser.Math.FloatBetween(
      ASTEROID_MIN_SCREEN_SPEED,
      ASTEROID_MAX_SCREEN_SPEED,
    );
    const wobble = Phaser.Math.FloatBetween(-10, 10) / MAX_ZOOM;
    const velocity = new Phaser.Math.Vector2(
      direction.x * (screenSpeed / MAX_ZOOM) + perpendicular.x * wobble,
      direction.y * (screenSpeed / MAX_ZOOM) + perpendicular.y * wobble,
    );

    this.asteroids.push(
      new Asteroid(this, x, y, radius, { material, amount }, velocity),
    );
  }

  private getAsteroidSpawnEdge(
    viewport: Phaser.Geom.Rectangle,
    direction: Phaser.Math.Vector2,
  ) {
    const margin = ASTEROID_SCREEN_MARGIN / MAX_ZOOM;
    const spawnOnVerticalEdge =
      Phaser.Math.FloatBetween(
        0,
        Math.abs(direction.x) + Math.abs(direction.y),
      ) < Math.abs(direction.x);

    if (spawnOnVerticalEdge && Math.abs(direction.x) > 1e-8) {
      return {
        x: direction.x > 0 ? viewport.left - margin : viewport.right + margin,
        y: Phaser.Math.FloatBetween(viewport.top, viewport.bottom),
        crossOffset: 0,
      };
    }

    if (Math.abs(direction.y) > 1e-8) {
      return {
        x: Phaser.Math.FloatBetween(viewport.left, viewport.right),
        y: direction.y > 0 ? viewport.top - margin : viewport.bottom + margin,
        crossOffset: 0,
      };
    }

    return {
      x: viewport.left - margin,
      y: Phaser.Math.FloatBetween(viewport.top, viewport.bottom),
      crossOffset: 0,
    };
  }

  private getAsteroidDriftDirection() {
    const velocity = getSpaceshipWorldVelocity();
    const speed = Math.hypot(velocity.x, velocity.y);
    if (speed > 0) {
      return new Phaser.Math.Vector2(-velocity.x / speed, -velocity.y / speed);
    }

    const headingRadians =
      (this.spaceship?.spaceship.heading ?? 0) * (Math.PI / 180);
    return new Phaser.Math.Vector2(
      -Math.sin(headingRadians),
      Math.cos(headingRadians),
    ).normalize();
  }

  private getShipMaxZoomViewport() {
    const width = this.scale.width / MAX_ZOOM;
    const height = this.scale.height / MAX_ZOOM;
    const shipX = this.spaceship?.x ?? 0;
    const shipY = this.spaceship?.y ?? 0;

    return new Phaser.Geom.Rectangle(
      shipX - width / 2,
      shipY - height / 2,
      width,
      height,
    );
  }

  private isMaxZoomedIn() {
    return this.cameras.main.zoom >= MAX_ZOOM * 0.999;
  }

  private canSpawnAsteroids() {
    if (!this.spaceship || getSpaceshipMotionState() !== 'flying') return false;

    const shipX = this.spaceship.x;
    const shipY = this.spaceship.y;
    return [...this.planetData, ...this.starData].every((body) => {
      const position = getRenderPosition(body.position);

      return (
        Phaser.Math.Distance.Between(
          shipX,
          shipY,
          Number(position.x),
          Number(position.y),
        ) >
        Number(body.radius) * ASTEROID_ORBIT_CLEARANCE_FACTOR
      );
    });
  }

  private clearAsteroids() {
    this.asteroids.forEach((asteroid) => asteroid.destroy());
    this.asteroids = [];
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

  startEngines(targetSpeed: number, maximumThrustPercent: number) {
    if (
      !this.spaceship ||
      this.spaceshipEngineRunning ||
      !Number.isFinite(targetSpeed) ||
      targetSpeed < 0 ||
      !Number.isFinite(maximumThrustPercent) ||
      maximumThrustPercent <= 0 ||
      maximumThrustPercent > 100
    ) {
      return;
    }

    try {
      startSpaceshipTargetSpeedFeature(
        targetSpeed,
        maximumThrustPercent,
        this.targetDirection,
      );
    } catch (error) {
      console.error('Failed to start target speed feature', error);
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

    this.targetDirection = direction;
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
    const showViewportLabels = this.shouldShowViewportLabels();
    this.lastShowViewportLabels = showViewportLabels;

    this.planets.forEach((planet) => {
      planet.setRenderVisibility(
        camera.zoom,
        camera.worldView,
        this.alwaysVisibleBodies.has(planet.planet.name),
        showViewportLabels,
      );
      planet.setVisible(planet.visible && isInsideWorld(planet.x, planet.y));
    });
    this.stars.forEach((star) => {
      star.setRenderVisibility(
        camera.zoom,
        camera.worldView,
        this.alwaysVisibleBodies.has(star.star.name),
        showViewportLabels,
      );
      star.setVisible(star.visible && isInsideWorld(star.x, star.y));
    });
    this.spaceship?.setRenderVisibility(camera.zoom, camera.worldView);
    if (this.spaceship) {
      this.spaceship.setVisible(
        this.spaceship.visible &&
          isInsideWorld(this.spaceship.x, this.spaceship.y),
      );
    }
    this.snapSpaceshipToSurface();
  }

  protected syncWorldPositions(bodyNames?: ReadonlySet<string>) {
    if (!bodyNames) {
      this.planets.forEach((planet) => planet.syncPosition());
      this.stars.forEach((star) => star.syncPosition());
      this.spaceship?.syncPosition();
      this.snapSpaceshipToSurface();
      return;
    }

    bodyNames.forEach((bodyName) => {
      this.planetByName.get(bodyName)?.syncPosition();
      this.starByName.get(bodyName)?.syncPosition();
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
    const showViewportLabels = this.shouldShowViewportLabels();
    if (showViewportLabels !== this.lastShowViewportLabels) {
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
        showViewportLabels,
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
        showViewportLabels,
      );
      star.setVisible(star.visible && isInsideWorld(star.x, star.y));
    });
    if (this.spaceship && bodyNames.has(this.spaceship.spaceship.name)) {
      this.spaceship.setRenderVisibility(camera.zoom, camera.worldView);
      this.spaceship.setVisible(
        this.spaceship.visible &&
          isInsideWorld(this.spaceship.x, this.spaceship.y),
      );
    }
    this.snapSpaceshipToSurface();
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
    const bodyPositionByName = this.getBodyPositionByName();
    const viewportBodyNames = this.getViewportBodyNames(bodyPositionByName);

    if (viewportBodyNames.length < VIEWPORT_LABEL_OBJECT_LIMIT) {
      viewportBodyNames.forEach((bodyName) => activeBodyNames.add(bodyName));
    }

    this.planetData.forEach((planet) => {
      if (
        this.alwaysVisibleBodies.has(planet.name) ||
        this.bodyMotionAreaIntersectsViewport(
          planet,
          Number(planet.radius),
          viewport,
          camera.zoom,
          bodyPositionByName,
        )
      ) {
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
      }
    });
    if (this.cameraLockedBodyName)
      activeBodyNames.add(this.cameraLockedBodyName);
    if (this.spaceship) activeBodyNames.add(this.spaceship.spaceship.name);

    if (this.reconcileRenderedBodies(activeBodyNames)) {
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
    if (zoom < body.shapeRenderZoomLevel) return false;

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

  private shouldShowViewportLabels() {
    return (
      this.getViewportBodyNames(this.getBodyPositionByName()).length <
      VIEWPORT_LABEL_OBJECT_LIMIT
    );
  }

  private getViewportBodyNames(
    bodyPositionByName: Map<string, RenderedBodyPosition>,
  ) {
    const viewport = this.cameras.main.worldView;
    return [...this.planetData, ...this.starData]
      .filter((body) => {
        const position = bodyPositionByName.get(body.name);
        return position && viewport.contains(position.x, position.y);
      })
      .map((body) => body.name);
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
    this.planetData.forEach((planet) => {
      bodyPositionByName.set(planet.name, getRenderPosition(planet.position));
    });
    this.starData.forEach((star) => {
      bodyPositionByName.set(star.name, getRenderPosition(star.position));
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
    this.planetData = planets;
    this.starData = stars;
    this.planetDataByName.clear();
    this.starDataByName.clear();
    planets.forEach((planet) => this.planetDataByName.set(planet.name, planet));
    stars.forEach((star) => this.starDataByName.set(star.name, star));
  }

  async refreshWorldFromViewport() {
    const camera = this.cameras.main;
    camera.preRender();
    const viewport = camera.worldView;
    const center = getWorldPositionFromRenderPosition(
      viewport.centerX,
      viewport.centerY,
    );
    const radius = BigInt(
      Math.ceil(Math.hypot(viewport.width, viewport.height) / 2),
    );
    const world = await refreshWorldViewport({
      x: center.x.toString(),
      y: center.y.toString(),
      radius: radius.toString(),
    });

    this.setWorldBodyData(world.planets, world.stars);
    this.syncWorldPositions();
    this.lastActiveBodiesViewportKey = '';
    this.lastVisibilityViewportKey = '';
    this.updateWorldVisibility();
    return world;
  }

  queueViewportWorldRefresh(delayMs = 250) {
    window.clearTimeout(this.viewportRefreshDebounceTimer);
    this.viewportRefreshDebounceTimer = window.setTimeout(() => {
      this.viewportRefreshDebounceTimer = undefined;
      this.refreshWorldFromViewportSafely();
    }, delayMs);
  }

  private refreshWorldFromViewportSafely() {
    if (this.viewportRefreshPromise) {
      this.hasPendingViewportRefresh = true;
      return;
    }

    this.viewportRefreshPromise ??= this.refreshWorldFromViewport()
      .catch((error: unknown) => {
        console.error('Failed to refresh viewport world data', error);
        this.onWorldLoadComplete?.(error);
      })
      .finally(() => {
        this.viewportRefreshPromise = undefined;
        if (this.hasPendingViewportRefresh) {
          this.hasPendingViewportRefresh = false;
          this.refreshWorldFromViewportSafely();
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
