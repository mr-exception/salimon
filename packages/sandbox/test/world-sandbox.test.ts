import { describe, expect, it } from 'vitest';
import {
  DEFAULT_SPACESHIP_RADIUS_METERS,
  SANDBOX_MAX_ENGINE_THRUST_N,
  SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS,
  SANDBOX_SPACESHIP_TICK_MS,
  SandBox,
  SandboxObject,
  WorldSandbox,
} from '../src';
import type { SandboxCollisionEvent } from '../src';

describe('WorldSandbox collision detection', () => {
  it('emits one collision event when a lone spaceship intersects a lone planet', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const sandbox = new WorldSandbox();
    const collisionEvents: SandboxCollisionEvent[] = [];

    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: 5.972e24,
        radius: 1_000,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: { x: 1_100, y: 0, relativeTo: 'Earth' },
      direction: 270,
      speed: 100,
      velocity: { x: -100, y: 0 },
      motionState: 'flying',
      simulatedAt: new Date(capturedAt),
      mass: 10_000,
      radius: 200,
    });
    sandbox.onCollision((event) => collisionEvents.push(event));

    sandbox.tick(capturedAt);
    sandbox.tick(capturedAt + 1);

    expect(collisionEvents).toHaveLength(1);
    expect(collisionEvents[0]).toMatchObject({
      objectAId: WorldSandbox.getBodyObjectId('Earth'),
      objectBId: WorldSandbox.getSpaceshipObjectId('ship-1'),
      position: { x: 1_000, y: 0 },
    });
    expect(collisionEvents[0].forceN).toBeCloseTo(
      SandBox.calculateCrashForceN(5.972e24, 10_000, 100),
    );
  });

  it('launches a landed spaceship from a planet surface with thruster power and advances it for five seconds', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const planetRadiusMeters = 6_371_000;
    const planetMassKg = 5.972e24;
    const spaceshipMassKg = 10_000;
    const thrusterPowerPercent = 50;
    const elapsedSeconds = 5;
    const sandbox = new WorldSandbox();

    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: planetMassKg,
        radius: planetRadiusMeters,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: { x: planetRadiusMeters, y: 0, relativeTo: 'Earth' },
      direction: 90,
      speed: 0,
      motionState: 'landed',
      simulatedAt: new Date(capturedAt),
      mass: spaceshipMassKg,
    });

    const launchSnapshot = sandbox.startSpaceshipThrusters(
      'ship-1',
      [
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: true, powerPercent: thrusterPowerPercent },
      ],
      capturedAt,
    );
    const launchRadius =
      planetRadiusMeters +
      DEFAULT_SPACESHIP_RADIUS_METERS +
      SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS;
    const thrustAcceleration =
      (SANDBOX_MAX_ENGINE_THRUST_N * (thrusterPowerPercent / 100)) /
      spaceshipMassKg;
    let expectedVelocity = 0;
    let expectedX = launchRadius;
    for (
      let elapsedMilliseconds = 0;
      elapsedMilliseconds < elapsedSeconds * 1_000;
      elapsedMilliseconds += SANDBOX_SPACESHIP_TICK_MS
    ) {
      const stepSeconds =
        Math.min(
          SANDBOX_SPACESHIP_TICK_MS,
          elapsedSeconds * 1_000 - elapsedMilliseconds,
        ) / 1_000;
      const gravityAcceleration =
        (SandboxObject.gravitationalConstant * planetMassKg) / expectedX ** 2;
      const expectedAcceleration = thrustAcceleration - gravityAcceleration;
      expectedVelocity += expectedAcceleration * stepSeconds;
      expectedX += expectedVelocity * stepSeconds;
    }

    expect(launchSnapshot).toMatchObject({
      securityCode: 'ship-1',
      position: { x: Math.round(launchRadius).toString(), y: '0' },
      velocity: { x: 0, y: 0 },
      speed: '0',
      direction: 0,
      simulatedAt: new Date(capturedAt),
      updatedAt: new Date(capturedAt),
    });

    sandbox.tick(capturedAt + elapsedSeconds * 1_000);

    const spaceship = sandbox.getObject(
      WorldSandbox.getSpaceshipObjectId('ship-1'),
    );
    const snapshot =
      spaceship &&
      sandbox.getSpaceshipSnapshot(
        spaceship,
        capturedAt + elapsedSeconds * 1_000,
      );

    expect(spaceship?.metadata).toMatchObject({
      motionState: 'flying',
      relativeTo: 'Earth',
    });
    expect(spaceship?.activeForce).toMatchObject({
      id: 'spaceship:thrusters',
      x: SANDBOX_MAX_ENGINE_THRUST_N * (thrusterPowerPercent / 100),
      y: 0,
    });
    expect(spaceship?.activeForce?.durationMs).toBeGreaterThan(
      Number.MAX_SAFE_INTEGER - elapsedSeconds * 1_000 - 1,
    );
    expect(spaceship?.velocity?.x).toBeCloseTo(expectedVelocity);
    expect(spaceship?.velocity?.y).toBe(0);
    expect(spaceship?.position.x).toBeCloseTo(expectedX);
    expect(spaceship?.position.y).toBe(0);
    expect(snapshot).toMatchObject({
      securityCode: 'ship-1',
      position: { x: Math.round(expectedX).toString(), y: '0' },
      speed: Math.round(Math.abs(expectedVelocity)).toString(),
      direction: 90,
      simulatedAt: new Date(capturedAt + elapsedSeconds * 1_000),
      updatedAt: new Date(capturedAt + elapsedSeconds * 1_000),
    });
    expect(snapshot?.velocity.x).toBeCloseTo(expectedVelocity);
    expect(snapshot?.velocity.y).toBe(0);
  });

  it('places a landed spaceship on the right side of its planet when loaded at the center', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const planetRadiusMeters = 6_371_000;
    const sandbox = new WorldSandbox();

    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: 5.972e24,
        radius: planetRadiusMeters,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: { x: 0, y: 0, relativeTo: 'Earth' },
      direction: 0,
      speed: 0,
      motionState: 'landed',
      simulatedAt: new Date(capturedAt),
    });

    const spaceship = sandbox.getObject(
      WorldSandbox.getSpaceshipObjectId('ship-1'),
    );
    const snapshot = spaceship
      ? sandbox.getSpaceshipSnapshot(spaceship, capturedAt)
      : undefined;

    expect(spaceship?.position).toEqual({
      x: planetRadiusMeters + DEFAULT_SPACESHIP_RADIUS_METERS,
      y: 0,
    });
    expect(snapshot?.position).toEqual({
      x: (planetRadiusMeters + DEFAULT_SPACESHIP_RADIUS_METERS).toString(),
      y: '0',
      relativeTo: 'Earth',
    });
  });

  it('does not move a landed spaceship that is already clear of the planet surface when thrusters start', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const planetRadiusMeters = 6_371_000;
    const sandbox = new WorldSandbox();
    const landedRadius =
      planetRadiusMeters + DEFAULT_SPACESHIP_RADIUS_METERS + 20_000;

    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: 5.972e24,
        radius: planetRadiusMeters,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: { x: landedRadius, y: 0, relativeTo: 'Earth' },
      direction: 90,
      speed: 0,
      motionState: 'landed',
      simulatedAt: new Date(capturedAt),
      mass: 10_000,
    });

    const launchSnapshot = sandbox.startSpaceshipThrusters(
      'ship-1',
      [
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: true, powerPercent: 50 },
      ],
      capturedAt,
    );

    expect(launchSnapshot).toMatchObject({
      securityCode: 'ship-1',
      position: { x: landedRadius.toString(), y: '0' },
      velocity: { x: 0, y: 0 },
      speed: '0',
      simulatedAt: new Date(capturedAt),
      updatedAt: new Date(capturedAt),
    });
  });

  it('starts thrusters from a landed spaceship local position after its planet moves', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const planetRadiusMeters = 6_371_000;
    const landedRadius = planetRadiusMeters + DEFAULT_SPACESHIP_RADIUS_METERS;
    const sandbox = new WorldSandbox();

    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: 5.972e24,
        radius: planetRadiusMeters,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: { x: landedRadius, y: 0, relativeTo: 'Earth' },
      direction: 90,
      speed: 0,
      motionState: 'landed',
      simulatedAt: new Date(capturedAt),
      mass: 10_000,
    });

    const earth = sandbox.getObject(WorldSandbox.getBodyObjectId('Earth'));
    if (!earth) throw new Error('Earth was not loaded');
    earth.position = { x: 1_000_000, y: 500_000 };

    const launchSnapshot = sandbox.startSpaceshipThrusters(
      'ship-1',
      [
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: true, powerPercent: 50 },
      ],
      capturedAt,
    );

    expect(launchSnapshot).toMatchObject({
      securityCode: 'ship-1',
      position: {
        x: (
          landedRadius + SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS
        ).toString(),
        y: '0',
        relativeTo: 'Earth',
      },
      velocity: { x: 0, y: 0 },
      speed: '0',
      simulatedAt: new Date(capturedAt),
      updatedAt: new Date(capturedAt),
    });
    expect(
      sandbox.getObject(WorldSandbox.getSpaceshipObjectId('ship-1'))?.position,
    ).toEqual({
      x:
        earth.position.x +
        landedRadius +
        SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS,
      y: earth.position.y,
    });
  });

  it('inherits orbital velocity from a body that uses position.relativeTo as its orbit frame', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const sandbox = new WorldSandbox();

    sandbox.loadBody(
      {
        name: 'Sun',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: 1.989e30,
        radius: 696_340_000,
        updatedAt: new Date(capturedAt),
      },
      'star',
    );
    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 1_000_000_000, y: 0, relativeTo: 'Sun' },
        orbitalCenter: null,
        clockwise: true,
        speed: 30_000,
        mass: 5.972e24,
        radius: 6_371_000,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: { x: 6_371_200, y: 0, relativeTo: 'Earth' },
      direction: 90,
      speed: 0,
      motionState: 'landed',
      simulatedAt: new Date(capturedAt),
      mass: 10_000,
    });

    const earth = sandbox.getObject(WorldSandbox.getBodyObjectId('Earth'));
    const spaceship = sandbox.getObject(
      WorldSandbox.getSpaceshipObjectId('ship-1'),
    );
    const landedSnapshot =
      spaceship && sandbox.getSpaceshipSnapshot(spaceship, capturedAt);
    const launchSnapshot = sandbox.startSpaceshipThrusters(
      'ship-1',
      [
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: true, powerPercent: 100 },
      ],
      capturedAt,
    );

    expect(earth?.velocity?.x).toBeCloseTo(0);
    expect(earth?.velocity?.y).toBeCloseTo(30_000);
    expect(spaceship?.velocity?.x).toBeCloseTo(0);
    expect(spaceship?.velocity?.y).toBeCloseTo(30_000);
    expect(landedSnapshot?.velocity.x).toBeCloseTo(0);
    expect(landedSnapshot?.velocity.y).toBeCloseTo(0);
    expect(landedSnapshot?.speed).toBe('0');
    expect(launchSnapshot?.velocity.x).toBeCloseTo(0);
    expect(launchSnapshot?.velocity.y).toBeCloseTo(0);
  });

  it('keeps a launched spaceship in its planet-relative physics frame while the planet orbits', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const planetRadiusMeters = 6_371_000;
    const launchRadius =
      planetRadiusMeters +
      DEFAULT_SPACESHIP_RADIUS_METERS +
      SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS;
    const sandbox = new WorldSandbox();

    sandbox.loadBody(
      {
        name: 'Sun',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: 0,
        radius: 696_340_000,
        updatedAt: new Date(capturedAt),
      },
      'star',
    );
    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 10_000_000, y: 0, relativeTo: 'Sun' },
        orbitalCenter: null,
        clockwise: true,
        speed: 30_000,
        mass: 5.972e24,
        radius: planetRadiusMeters,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: { x: planetRadiusMeters, y: 0, relativeTo: 'Earth' },
      direction: 90,
      speed: 0,
      motionState: 'landed',
      simulatedAt: new Date(capturedAt),
      mass: 10_000,
    });

    sandbox.startSpaceshipThrusters(
      'ship-1',
      [
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: false, powerPercent: 0 },
        { active: true, powerPercent: 10 },
      ],
      capturedAt,
    );
    sandbox.tick(capturedAt + 30_000);

    const spaceship = sandbox.getObject(
      WorldSandbox.getSpaceshipObjectId('ship-1'),
    );
    const snapshot =
      spaceship && sandbox.getSpaceshipSnapshot(spaceship, capturedAt + 30_000);
    const relativePosition = snapshot?.position;
    const relativeRadius = relativePosition
      ? Math.hypot(Number(relativePosition.x), Number(relativePosition.y))
      : 0;
    const surfaceAltitude =
      relativeRadius - planetRadiusMeters - DEFAULT_SPACESHIP_RADIUS_METERS;

    expect(relativePosition?.relativeTo).toBe('Earth');
    expect(relativeRadius).toBeGreaterThan(launchRadius);
    expect(Number(relativePosition?.y)).toBe(0);
    expect(surfaceAltitude).toBeLessThan(10_000);
  });

  it('restores active spaceship thrusters when loading a persisted flying spaceship', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const planetRadiusMeters = 6_371_000;
    const spaceshipMassKg = 10_000;
    const thrusterPowerPercent = 50;
    const elapsedSeconds = 5;
    const sandbox = new WorldSandbox();

    sandbox.loadBody(
      {
        name: 'Earth',
        position: { x: 0, y: 0 },
        orbitalCenter: null,
        clockwise: true,
        speed: 0,
        mass: 5.972e24,
        radius: planetRadiusMeters,
        updatedAt: new Date(capturedAt),
      },
      'planet',
    );
    sandbox.loadSpaceship({
      securityCode: 'ship-1',
      position: {
        x: (
          planetRadiusMeters +
          DEFAULT_SPACESHIP_RADIUS_METERS +
          SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS +
          1_000
        ).toString(),
        y: '0',
        relativeTo: 'Earth',
      },
      direction: 90,
      speed: 100,
      velocity: { x: 100, y: 0 },
      motionState: 'flying',
      simulatedAt: new Date(capturedAt),
      mass: spaceshipMassKg,
      activeFeature: {
        type: 'thrusters',
        thrusters: [
          { active: false, powerPercent: 0 },
          { active: false, powerPercent: 0 },
          { active: false, powerPercent: 0 },
          { active: true, powerPercent: thrusterPowerPercent },
        ],
        elapsedSeconds: 0,
      },
    });

    const spaceship = sandbox.getObject(
      WorldSandbox.getSpaceshipObjectId('ship-1'),
    );
    const thrustAcceleration =
      (SANDBOX_MAX_ENGINE_THRUST_N * (thrusterPowerPercent / 100)) /
      spaceshipMassKg;

    expect(spaceship?.activeForce).toMatchObject({
      id: 'spaceship:thrusters',
      x: SANDBOX_MAX_ENGINE_THRUST_N * (thrusterPowerPercent / 100),
      y: 0,
    });

    sandbox.tick(capturedAt + elapsedSeconds * 1_000);

    expect(spaceship?.velocity?.x).toBeGreaterThan(
      100 + thrustAcceleration * elapsedSeconds * 0.5,
    );
    expect(spaceship?.activeForce).toBeDefined();
  });

  it('matches repeated spaceship ticks when a delayed thruster tick spans several minutes', () => {
    const capturedAt = new Date('2026-01-01T00:00:00.000Z').getTime();
    const planetRadiusMeters = 6_371_000;
    const elapsedMilliseconds = 5 * 60 * 1_000;

    const createSandbox = () => {
      const sandbox = new WorldSandbox();
      sandbox.loadBody(
        {
          name: 'Earth',
          position: { x: 0, y: 0 },
          orbitalCenter: null,
          clockwise: true,
          speed: 0,
          mass: 5.972e24,
          radius: planetRadiusMeters,
          updatedAt: new Date(capturedAt),
        },
        'planet',
      );
      sandbox.loadSpaceship({
        securityCode: 'ship-1',
        position: {
          x: planetRadiusMeters + DEFAULT_SPACESHIP_RADIUS_METERS,
          y: 0,
          relativeTo: 'Earth',
        },
        direction: 90,
        speed: 0,
        motionState: 'landed',
        simulatedAt: new Date(capturedAt),
        mass: 10_000,
      });
      sandbox.startSpaceshipThrusters(
        'ship-1',
        [
          { active: false, powerPercent: 0 },
          { active: false, powerPercent: 0 },
          { active: false, powerPercent: 0 },
          { active: true, powerPercent: 20 },
        ],
        capturedAt,
      );
      return sandbox;
    };
    const delayedSandbox = createSandbox();
    const repeatedSandbox = createSandbox();

    delayedSandbox.tick(capturedAt + elapsedMilliseconds);
    for (
      let milliseconds = SANDBOX_SPACESHIP_TICK_MS;
      milliseconds <= elapsedMilliseconds;
      milliseconds += SANDBOX_SPACESHIP_TICK_MS
    ) {
      repeatedSandbox.tick(capturedAt + milliseconds);
    }

    const delayedSpaceship = delayedSandbox.getObject(
      WorldSandbox.getSpaceshipObjectId('ship-1'),
    );
    const repeatedSpaceship = repeatedSandbox.getObject(
      WorldSandbox.getSpaceshipObjectId('ship-1'),
    );

    expect(delayedSpaceship?.position.x).toBeCloseTo(
      repeatedSpaceship?.position.x ?? 0,
    );
    expect(delayedSpaceship?.velocity?.x).toBeCloseTo(
      repeatedSpaceship?.velocity?.x ?? 0,
    );
  });
});
