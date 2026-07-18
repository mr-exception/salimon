import { describe, expect, it } from 'vitest';
import {
  DEFAULT_SPACESHIP_RADIUS_METERS,
  SANDBOX_MAX_ENGINE_THRUST_N,
  SANDBOX_SPACESHIP_LAUNCH_CLEARANCE_METERS,
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
    const gravityAcceleration =
      (SandboxObject.gravitationalConstant * planetMassKg) / launchRadius ** 2;
    const expectedAcceleration = thrustAcceleration - gravityAcceleration;
    const expectedVelocity = expectedAcceleration * elapsedSeconds;
    const expectedX = launchRadius + expectedVelocity * elapsedSeconds;

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

    expect(spaceship?.metadata).toMatchObject({ motionState: 'flying' });
    expect(spaceship?.metadata?.relativeTo).toBeUndefined();
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
      velocity: { x: expectedVelocity, y: 0 },
      speed: Math.round(Math.abs(expectedVelocity)).toString(),
      direction: 90,
      simulatedAt: new Date(capturedAt + elapsedSeconds * 1_000),
      updatedAt: new Date(capturedAt + elapsedSeconds * 1_000),
    });
  });
});
