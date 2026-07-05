import { useCallback, useRef, useState } from 'react';
import { useAtom } from 'jotai';
import { Footer, Navigator, SpaceshipHull, StartMenu } from '@components';
import { activeViewAtom, useBootstrap, type BootstrapRequest } from '@store';
import style from './style.module.css';

export default function App() {
  const [bootstrapRequest, setBootstrapRequest] =
    useState<BootstrapRequest | null>(null);
  const bootstrapState = useBootstrap(bootstrapRequest);
  const [activeView, setActiveView] = useAtom(activeViewAtom);
  const [isEngineRunning, setIsEngineRunning] = useState(false);
  const [isSelectingTargetDirection, setIsSelectingTargetDirection] =
    useState(false);
  const sceneRef = useRef<{
    startEngines: (targetSpeed: number, maximumThrustPercent: number) => void;
    stopEngines: () => void;
    setManualThrust: (
      direction: { x: number; y: number } | undefined,
      power: number,
    ) => void;
    setTargetDirectionSelectionActive: (active: boolean) => void;
  } | null>(null);
  const handleSceneChange = useCallback(
    (
      scene: {
        startEngines: (
          targetSpeed: number,
          maximumThrustPercent: number,
        ) => void;
        stopEngines: () => void;
        setManualThrust: (
          direction: { x: number; y: number } | undefined,
          power: number,
        ) => void;
        setTargetDirectionSelectionActive: (active: boolean) => void;
      } | null,
    ) => {
      sceneRef.current = scene;
    },
    [],
  );
  const startEngines = useCallback(
    (targetSpeed: number, maximumThrustPercent: number) => {
      sceneRef.current?.startEngines(targetSpeed, maximumThrustPercent);
    },
    [],
  );
  const stopEngines = useCallback(() => {
    sceneRef.current?.stopEngines();
  }, []);
  const setManualThrust = useCallback(
    (direction: { x: number; y: number } | undefined, power: number) => {
      sceneRef.current?.setManualThrust(direction, power);
    },
    [],
  );
  const toggleTargetDirectionSelection = useCallback(() => {
    setIsSelectingTargetDirection((isSelecting) => {
      const active = !isSelecting;
      sceneRef.current?.setTargetDirectionSelectionActive(active);
      return active;
    });
  }, []);
  const handleTargetDirectionSelected = useCallback(() => {
    setIsSelectingTargetDirection(false);
  }, []);

  if (bootstrapState !== 'ready') {
    return (
      <StartMenu
        bootstrapState={bootstrapState}
        onStart={setBootstrapRequest}
      />
    );
  }

  return (
    <div className={style.app}>
      <div className={style.view} hidden={activeView !== 'navigation'}>
        <Navigator
          onSceneChange={handleSceneChange}
          onSpaceshipEngineChange={setIsEngineRunning}
          isSelectingTargetDirection={isSelectingTargetDirection}
          onTargetDirectionSelected={handleTargetDirectionSelected}
        />
      </div>
      <div className={style.view} hidden={activeView !== 'ship'}>
        <SpaceshipHull />
      </div>
      <Footer
        activeView={activeView}
        isEngineRunning={isEngineRunning}
        onStartEngines={startEngines}
        onStopEngines={stopEngines}
        onManualThrustChange={setManualThrust}
        isSelectingTargetDirection={isSelectingTargetDirection}
        onToggleTargetDirectionSelection={toggleTargetDirectionSelection}
        onViewChange={setActiveView}
      />
    </div>
  );
}
