import { add } from './add';
import { calculateAcceleration } from './calculate-acceleration';
import { calculateGravityAcceleration } from './calculate-gravity-acceleration';
import { calculateMaximumEngineAcceleration } from './calculate-maximum-engine-acceleration';
import { calculateRequiredBurnAcceleration } from './calculate-required-burn-acceleration';
import { calculateTargetSpeedBurnDuration } from './calculate-target-speed-burn-duration';
import { getActiveThrusters } from './get-active-thrusters';
import { getBodyPositions } from './get-body-positions';
import { getImpactMotionState } from './get-impact-motion-state';
import { getSurfaceVelocity } from './get-surface-velocity';
import { integrateStep } from './integrate-step';
import { rotateAttachedPosition } from './rotate-attached-position';
import type { ActiveThrusters } from './types';
import { wearThrusters } from './wear-thrusters';

export type { ActiveThrusters };

export class PhysicsService {
  static add = add;
  static getBodyPositions = getBodyPositions;
  static calculateGravityAcceleration = calculateGravityAcceleration;
  static calculateAcceleration = calculateAcceleration;
  static integrateStep = integrateStep;
  static getActiveThrusters = getActiveThrusters;
  static wearThrusters = wearThrusters;
  static calculateRequiredBurnAcceleration = calculateRequiredBurnAcceleration;
  static calculateTargetSpeedBurnDuration = calculateTargetSpeedBurnDuration;
  static calculateMaximumEngineAcceleration = calculateMaximumEngineAcceleration;
  static rotateAttachedPosition = rotateAttachedPosition;
  static getSurfaceVelocity = getSurfaceVelocity;
  static getImpactMotionState = getImpactMotionState;
}
