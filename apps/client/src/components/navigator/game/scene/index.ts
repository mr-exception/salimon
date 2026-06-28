import Phaser from 'phaser';
import {
  advanceWorld,
  didSpaceshipBurnReachTarget,
  getSpaceshipAttachedBodyName,
  getSpaceshipBurnAcceleration,
  getSpaceshipMotionState,
  getSpaceshipVelocity,
  isSpaceshipEngineRunning,
  resolveSpaceshipPlanetCollision,
  setActiveWorldBodyNames,
  setSpaceshipTargetDirection,
  setSpaceshipHeading,
  setSpaceshipManualThrust,
  startSpaceshipEngines,
  stopSpaceshipEngines,
} from '@store';
import type { Planet as PlanetData, Star as StarData } from '@types';
import {
  DEFAULT_RENDER_ORIGIN_NAME,
  getRenderPosition,
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
  getPlanetNameFromPhysicsLabel,
  SPACESHIP_PHYSICS_LABEL,
} from '../physics';
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
  protected planetData: PlanetData[] = [];
  protected starData: StarData[] = [];
  protected planets: Planet[] = [];
  protected stars: Star[] = [];
  protected spaceship?: Spaceship;
  protected grid?: Phaser.GameObjects.Graphics;
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
  private selectingTargetDirection = false;
  private targetDirection?: number;
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
  ) {
    super('navigation');
    this.onZoomChange = onZoomChange;
    this.onBodyContextMenu = onBodyContextMenu;
    this.onSpaceshipTurnChange = onSpaceshipTurnChange;
    this.onSpaceshipEngineChange = onSpaceshipEngineChange;
    this.onTargetDirectionPreview = onTargetDirectionPreview;
    this.onTargetDirectionSelected = onTargetDirectionSelected;
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
    this.drawVisibleWorld();
    this.configureInput();
    this.matter.world.on(
      Phaser.Physics.Matter.Events.COLLISION_START,
      this.handleMatterCollision,
    );
    this.events.once(Phaser.Scenes.Events.SHUTDOWN, () => {
      this.matter.world.off(
        Phaser.Physics.Matter.Events.COLLISION_START,
        this.handleMatterCollision,
      );
      this.unsubscribeFromWorld?.();
      this.unsubscribeFromWorld = undefined;
    });
    void this.renderWorld();
  }

  update(_time: number, delta: number) {
    this.publishActiveWorldBodies();
    const worldElapsedSeconds = advanceWorld(delta / 1000);
    this.planets.forEach((planet) => planet.syncRotation(worldElapsedSeconds));
    this.stars.forEach((star) => star.syncRotation(worldElapsedSeconds));
    if (getSpaceshipMotionState() === 'crashed') {
      this.spaceship?.clearTargetDirection();
      this.targetDirection = undefined;
    }
    const spaceshipEngineRunning = isSpaceshipEngineRunning();
    if (!this.spaceshipEngineRunning && spaceshipEngineRunning) {
      this.spaceshipEngineRunning = true;
      this.spaceship?.setThrustersActive(true, getSpaceshipBurnAcceleration());
      this.onSpaceshipEngineChange?.(true, this.getSpaceshipSpeed());
    } else if (this.spaceshipEngineRunning && !spaceshipEngineRunning) {
      this.spaceshipEngineRunning = false;
      this.spaceship?.setThrustersActive(false);
      if (didSpaceshipBurnReachTarget()) {
        this.spaceship?.clearTargetDirection();
        this.targetDirection = undefined;
        setSpaceshipTargetDirection(undefined);
      }
      this.onSpaceshipEngineChange?.(false, this.getSpaceshipSpeed());
    } else if (spaceshipEngineRunning) {
      this.spaceship?.setThrustersActive(true, getSpaceshipBurnAcceleration());
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
  }

  private readonly handleMatterCollision = (
    event: Phaser.Physics.Matter.Events.CollisionStartEvent,
  ) => {
    event.pairs.forEach(({ bodyA, bodyB }) => {
      const labels = [bodyA.label, bodyB.label];
      if (!labels.includes(SPACESHIP_PHYSICS_LABEL)) return;

      const planetName = labels
        .map(getPlanetNameFromPhysicsLabel)
        .find((name) => name !== undefined);
      if (planetName) resolveSpaceshipPlanetCollision(planetName);
    });
  };

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

    const currentSpeed = this.getSpaceshipSpeed();
    if (
      !startSpaceshipEngines(
        targetSpeed,
        maximumThrustPercent,
        this.targetDirection,
      )
    ) {
      return;
    }

    this.spaceshipEngineRunning = true;
    this.spaceship.setThrustersActive(true, getSpaceshipBurnAcceleration());
    this.onSpaceshipEngineChange?.(true, currentSpeed);
  }

  stopEngines() {
    if (!this.spaceshipEngineRunning || !stopSpaceshipEngines()) return;

    this.spaceshipEngineRunning = false;
    this.spaceship?.setThrustersActive(false);
    this.onSpaceshipEngineChange?.(false, this.getSpaceshipSpeed());
  }

  setManualThrust(
    direction: { x: number; y: number } | undefined,
    power: number,
  ) {
    setSpaceshipManualThrust(direction, power);
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
    const velocity = getSpaceshipVelocity();
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

    this.reconcileRenderedBodies(activeBodyNames);
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

  private reconcileRenderedBodies(activeBodyNames: ReadonlySet<string>) {
    activeBodyNames.forEach((bodyName) => this.ensureRenderedBody(bodyName));

    this.planets = this.planets.filter((planet) => {
      if (
        activeBodyNames.has(planet.planet.name) ||
        this.alwaysVisibleBodies.has(planet.planet.name)
      ) {
        return true;
      }

      this.planetByName.delete(planet.planet.name);
      planet.destroy();
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
      return false;
    });
  }

  private ensureRenderedBody(name: string) {
    if (this.planetByName.has(name) || this.starByName.has(name)) return;

    const planetData = this.planetDataByName.get(name);
    if (planetData) {
      const planet = new Planet(this, planetData);
      this.planets.push(planet);
      this.planetByName.set(planet.name, planet);
      return;
    }

    const starData = this.starDataByName.get(name);
    if (starData) {
      const star = new Star(this, starData);
      this.stars.push(star);
      this.starByName.set(star.name, star);
    }
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
