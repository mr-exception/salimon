import { createSpaceship } from './create-spaceship';
import { flushToDatabase } from './flush-to-database';
import { getWorldData } from './get-world-data';
import { getWorldSystemsBodies } from './get-world-systems-bodies';
import { startSpaceshipTargetSpeedFeature } from './start-spaceship-target-speed-feature';
import { start } from './start';
import { stopSpaceshipActiveFeature } from './stop-spaceship-active-feature';
import { stop } from './stop';
import { updateSpaceship } from './update-spaceship';
import { updateSpaceships } from './update-spaceships';
import { updateWorldBodies } from './update-world-bodies';
import { updateWorld } from './update-world';

export class TickingService {
  static start = start;
  static stop = stop;
  static getWorldData = getWorldData;
  static getWorldSystemsBodies = getWorldSystemsBodies;
  static createSpaceship = createSpaceship;
  static updateWorld = updateWorld;
  static updateWorldBodies = updateWorldBodies;
  static updateSpaceships = updateSpaceships;
  static updateSpaceship = updateSpaceship;
  static startSpaceshipTargetSpeedFeature = startSpaceshipTargetSpeedFeature;
  static stopSpaceshipActiveFeature = stopSpaceshipActiveFeature;
  static flushToDatabase = flushToDatabase;
}

