import { findOldestSpaceshipsForSimulation } from './find-oldest-spaceships-for-simulation';
import { findSpaceshipBySecurityCode } from './find-spaceship-by-security-code';
import { flushToDatabase } from './flush-to-database';
import { getWorldData } from './get-world-data';
import { getWorldSystemsBodies } from './get-world-systems-bodies';
import { insertSpaceship } from './insert-spaceship';
import { start } from './start';
import { stop } from './stop';
import { updatePropagatedSpaceship } from './update-propagated-spaceship';
import { updateSpaceshipInventoryBySecurityCode } from './update-spaceship-inventory-by-security-code';
import { updateSpaceshipBySecurityCode } from './update-spaceship-by-security-code';
import { updateSpaceships } from './update-spaceships';
import { updateWorldBodies } from './update-world-bodies';

export class RepositoryService {
  static start = start;
  static stop = stop;
  static getWorldData = getWorldData;
  static getWorldSystemsBodies = getWorldSystemsBodies;
  static updateWorldBodies = updateWorldBodies;
  static updateSpaceships = updateSpaceships;
  static insertSpaceship = insertSpaceship;
  static findSpaceshipBySecurityCode = findSpaceshipBySecurityCode;
  static updateSpaceshipBySecurityCode = updateSpaceshipBySecurityCode;
  static updateSpaceshipInventoryBySecurityCode =
    updateSpaceshipInventoryBySecurityCode;
  static findOldestSpaceshipsForSimulation = findOldestSpaceshipsForSimulation;
  static updatePropagatedSpaceship = updatePropagatedSpaceship;
  static flushToDatabase = flushToDatabase;
}
