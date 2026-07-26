import axios from 'axios';
import type { SerializedWorldSystems, SpaceshipDto } from '@repo/types';
import {
  advanceWorld,
  getSimulationFrameSnapshot,
  hydrateSpaceship,
  hydrateWorldSystems,
  setActiveWorldBodyNames,
  setSpaceshipHeading,
  startSpaceshipTargetSpeed,
  startSpaceshipThrusters,
  stopSpaceshipActiveFeatureLocally,
} from './world';
import type {
  SimulationWorkerRequest,
  SimulationWorkerResponse,
} from './world-simulation-protocol';

const workerScope = globalThis as unknown as {
  postMessage(message: SimulationWorkerResponse): void;
  onmessage: ((event: MessageEvent<SimulationWorkerRequest>) => void) | null;
};

const DEFAULT_API_BASE_URL = 'http://localhost:3000';
const SECURITY_CODE_HEADER = 'x-spaceship-security-code';

type SpaceshipResponse = { spaceship: SpaceshipDto };

function getApiBaseUrl() {
  return (import.meta.env.VITE_API_BASE_URL || DEFAULT_API_BASE_URL).replace(
    /\/+$/,
    '',
  );
}

async function initializeSpaceship(
  request: Extract<
    SimulationWorkerRequest,
    { type: 'initialize-spaceship' }
  >['request'],
) {
  if (request.type === 'new') {
    const { data } = await axios.post<SpaceshipResponse>(
      `${getApiBaseUrl()}/spaceship/register`,
    );
    return data.spaceship;
  }

  const { data } = await axios.get<SpaceshipResponse>(
    `${getApiBaseUrl()}/spaceship/info`,
    { headers: { [SECURITY_CODE_HEADER]: request.securityCode } },
  );
  return data.spaceship;
}

async function loadWorldViewport(viewport: {
  x1: string;
  y1: string;
  x2: string;
  y2: string;
  zoom?: number;
  requiredBodyNames?: string[];
}) {
  const { data } = await axios.post<SerializedWorldSystems>(
    `${getApiBaseUrl()}/world/systems`,
    viewport,
  );
  return data;
}

function publishFrame(requestId?: number) {
  workerScope.postMessage({
    type: 'frame',
    requestId,
    snapshot: getSimulationFrameSnapshot(),
  });
}

function publishError(error: unknown) {
  workerScope.postMessage({
    type: 'error',
    message: error instanceof Error ? error.message : String(error),
  });
}

workerScope.onmessage = (event) => {
  try {
    const message = event.data;

    switch (message.type) {
      case 'initialize-spaceship':
        void initializeSpaceship(message.request)
          .then((spaceship) => {
            hydrateSpaceship(spaceship);
            workerScope.postMessage({
              type: 'spaceship',
              requestId: message.requestId,
              spaceship,
              snapshot: getSimulationFrameSnapshot(),
            });
          })
          .catch(publishError);
        break;
      case 'refresh-viewport':
        void loadWorldViewport(message.viewport)
          .then((systems) => {
            hydrateWorldSystems(systems);
            workerScope.postMessage({
              type: 'viewport',
              requestId: message.requestId,
              systems,
              snapshot: getSimulationFrameSnapshot(),
            });
          })
          .catch(publishError);
        break;
      case 'hydrate-world':
        hydrateWorldSystems(message.systems);
        publishFrame();
        break;
      case 'hydrate-spaceship':
        hydrateSpaceship(message.spaceship);
        publishFrame();
        break;
      case 'set-active-bodies':
        setActiveWorldBodyNames(message.names);
        break;
      case 'set-heading':
        setSpaceshipHeading(message.heading);
        publishFrame();
        break;
      case 'start-thrusters':
        startSpaceshipThrusters(message.thrusters);
        publishFrame();
        break;
      case 'start-target-speed':
        startSpaceshipTargetSpeed(
          message.targetSpeedMetersPerSecond,
          message.maximumThrustPercent,
          message.targetDirection,
        );
        publishFrame();
        break;
      case 'stop-active-feature':
        stopSpaceshipActiveFeatureLocally();
        publishFrame();
        break;
      case 'advance':
        advanceWorld(message.elapsedSeconds);
        publishFrame(message.requestId);
        break;
    }
  } catch (error) {
    publishError(error);
  }
};
