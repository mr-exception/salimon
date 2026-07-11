import type { SpaceshipDto } from '@repo/types';
import {
  MAX_HULL_DURABILITY,
  MAX_THRUSTER_DURABILITY,
  SECURITY_CODE_HEADER,
  SPACESHIP_THRUSTER_COUNT,
} from './constants';
import { createSpaceship } from './create-spaceship';
import { getSecurityCode } from './get-security-code';
import { getSpaceshipVelocity } from './get-spaceship-velocity';
import { loadSpaceship } from './load-spaceship';
import { normalizeSpaceshipInventory } from './normalize-spaceship-inventory';
import { normalizeSpaceshipStats } from './normalize-spaceship-stats';
import { parseSpaceshipInventory } from './parse-spaceship-inventory';
import { parseSpaceshipUpdate } from './parse-spaceship-update';
import { toSpaceshipDto } from './to-spaceship-dto';
import { updateSpaceship } from './update-spaceship';

export {
  MAX_HULL_DURABILITY,
  MAX_THRUSTER_DURABILITY,
  SECURITY_CODE_HEADER,
  SPACESHIP_THRUSTER_COUNT,
};

export type {
  SpaceshipActiveFeature,
  SpaceshipDocument,
  SpaceshipInventory,
  SpaceshipMotionState,
  SpaceshipStats,
  SpaceshipVelocity,
} from '@models';

export type { SpaceshipDto };

export class SpaceshipService {
  static createSpaceship = createSpaceship;
  static toSpaceshipDto = toSpaceshipDto;
  static loadSpaceship = loadSpaceship;
  static updateSpaceship = updateSpaceship;
  static getSpaceshipVelocity = getSpaceshipVelocity;
  static getSecurityCode = getSecurityCode;
  static parseSpaceshipUpdate = parseSpaceshipUpdate;
  static parseSpaceshipInventory = parseSpaceshipInventory;
  static normalizeSpaceshipStats = normalizeSpaceshipStats;
  static normalizeSpaceshipInventory = normalizeSpaceshipInventory;
}
