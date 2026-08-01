import RAPIER from '@dimforge/rapier2d-compat';

export const SPACESHIP_PHYSICS_LABEL = 'spaceship';
export const PLANET_PHYSICS_LABEL_PREFIX = 'planet:';

let rapierInitialized = false;
let rapierWorld: RAPIER.World | undefined;

type Vector = { x: number; y: number };

export function getPlanetPhysicsLabel(name: string) {
  return `${PLANET_PHYSICS_LABEL_PREFIX}${name}`;
}

export function getPlanetNameFromPhysicsLabel(label: string) {
  return label.startsWith(PLANET_PHYSICS_LABEL_PREFIX)
    ? label.slice(PLANET_PHYSICS_LABEL_PREFIX.length)
    : undefined;
}

export async function initializeNavigatorPhysics() {
  if (!rapierInitialized) {
    await RAPIER.init();
    rapierInitialized = true;
  }

  disposeNavigatorPhysics();
  rapierWorld = new RAPIER.World({ x: 0, y: 0 });
}

export function disposeNavigatorPhysics() {
  rapierWorld?.free();
  rapierWorld = undefined;
}

export function createStaticCirclePhysicsBody(
  position: Vector,
  radius: number,
  label: string,
) {
  const world = getNavigatorPhysicsWorld();
  const body = world.createRigidBody(
    RAPIER.RigidBodyDesc.fixed()
      .setTranslation(position.x, position.y)
      .setUserData({ label }),
  );
  world.createCollider(
    RAPIER.ColliderDesc.ball(Math.max(1, radius))
      .setFriction(1)
      .setRestitution(0),
    body,
  );
  return body;
}

export function createDynamicCirclePhysicsBody(
  position: Vector,
  radius: number,
  mass: number,
  label: string,
) {
  const world = getNavigatorPhysicsWorld();
  const body = world.createRigidBody(
    RAPIER.RigidBodyDesc.dynamic()
      .setTranslation(position.x, position.y)
      .setGravityScale(0)
      .setLinearDamping(1)
      .setAngularDamping(1)
      .setAdditionalMass(Math.max(0, mass))
      .setCanSleep(false)
      .setUserData({ label }),
  );
  world.createCollider(
    RAPIER.ColliderDesc.ball(Math.max(1, radius))
      .setFriction(1)
      .setRestitution(0),
    body,
  );
  return body;
}

export function setPhysicsBodyPosition(
  body: RAPIER.RigidBody,
  position: Vector,
) {
  body.setTranslation(position, true);
}

export function resetPhysicsBodyVelocity(body: RAPIER.RigidBody) {
  body.setLinvel({ x: 0, y: 0 }, true);
  body.setAngvel(0, true);
}

export function stepNavigatorPhysics(deltaSeconds: number) {
  if (!rapierWorld) return;

  rapierWorld.integrationParameters.dt = deltaSeconds;
  rapierWorld.step();
}

export function removePhysicsBody(body: RAPIER.RigidBody) {
  rapierWorld?.removeRigidBody(body);
}

function getNavigatorPhysicsWorld() {
  if (!rapierWorld) {
    throw new Error('Navigator physics must be initialized before use.');
  }

  return rapierWorld;
}
