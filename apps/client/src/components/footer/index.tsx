import { useEffect, useRef, useState } from 'react';
import style from './style.module.css';
import {
  type FooterView,
  INITIAL_SPACESHIP_FUEL_KNS,
  MAX_ENGINE_THRUST_KN,
  SPACESHIP_MASS_KG,
  getSpaceshipBurnPlan,
  getSpaceshipBurnAcceleration,
  getSpaceshipBurnRemainingSeconds,
  getSpaceshipVelocity,
  startSpaceshipAutoOrbit,
  stopSpaceshipAutoOrbit,
  useSpaceshipAutoOrbit,
  useSpaceshipFuelKns,
  useSpaceshipMotionState,
  useSpaceshipSpeed,
  useSpaceshipTargetDirection,
} from '@store';
import {
  formatAcceleration,
  formatDuration,
  formatForce,
  formatImpulse,
  formatPercentage,
  formatSpeed,
} from '../../utils';

type FooterProps = {
  activeView: FooterView;
  isEngineRunning?: boolean;
  isSelectingTargetDirection?: boolean;
  onStartEngines?: (targetSpeed: number, maximumThrustPercent: number) => void;
  onStopEngines?: () => void;
  onManualThrustChange?: (
    direction: { x: number; y: number } | undefined,
    power: number,
  ) => void;
  onToggleTargetDirectionSelection?: () => void;
  onViewChange: (view: FooterView) => void;
};

type SpeedControlTab = 'target-speed' | 'auto-orbit' | 'manual-drive';

export function Footer({
  activeView,
  isEngineRunning = false,
  isSelectingTargetDirection = false,
  onStartEngines,
  onStopEngines,
  onManualThrustChange,
  onToggleTargetDirectionSelection,
  onViewChange,
}: FooterProps) {
  const speed = useSpaceshipSpeed();
  const fuelKns = useSpaceshipFuelKns();
  const motionState = useSpaceshipMotionState();
  const targetDirection = useSpaceshipTargetDirection();
  const autoOrbit = useSpaceshipAutoOrbit();
  const [targetSpeed, setTargetSpeed] = useState('10');
  const [maximumThrustPercent, setMaximumThrustPercent] = useState('100');
  const [orbitSpeed, setOrbitSpeed] = useState('7.8');
  const [orbitDistance, setOrbitDistance] = useState('400');
  const [orbitError, setOrbitError] = useState('');
  const [manualPower, setManualPower] = useState(25);
  const pressedDriveKeys = useRef(new Set<string>());
  const [expandedSpeedControls, setExpandedSpeedControls] = useState(
    () => new Set<SpeedControlTab>(),
  );
  const [burnStartVelocity, setBurnStartVelocity] = useState(() =>
    getSpaceshipVelocity(),
  );
  const targetSpeedMetersPerSecond = Number(targetSpeed) * 1_000;
  const maximumThrustPercentValue = Number(maximumThrustPercent);
  const currentVelocity = getSpaceshipVelocity();
  const calculationStartVelocity = isEngineRunning
    ? burnStartVelocity
    : currentVelocity;
  const burnPlan =
    targetDirection !== undefined
      ? getSpaceshipBurnPlan(
          targetSpeedMetersPerSecond,
          maximumThrustPercentValue,
          targetDirection,
          calculationStartVelocity,
        )
      : undefined;
  const burnAcceleration = isEngineRunning
    ? getSpaceshipBurnAcceleration()
    : burnPlan?.acceleration;
  const burnRemainingSeconds = getSpaceshipBurnRemainingSeconds();
  const burnTimeSeconds =
    burnRemainingSeconds ??
    (isEngineRunning ? undefined : burnPlan?.durationSeconds);
  const acceleration = burnAcceleration
    ? Math.hypot(burnAcceleration.x, burnAcceleration.y)
    : Number.NaN;
  const requiredThrustNewtons = Math.abs(acceleration) * SPACESHIP_MASS_KG;
  const maxEngineThrustNewtons = MAX_ENGINE_THRUST_KN * 1_000;
  const hasValidBurn = Number.isFinite(acceleration) && burnPlan !== undefined;
  const canStartBurn = motionState !== 'crashed' && hasValidBurn && fuelKns > 0;
  const currentEnginePowerPercent = isEngineRunning
    ? (requiredThrustNewtons / maxEngineThrustNewtons) * 100
    : 0;
  const orbitSpeedMetersPerSecond = Number(orbitSpeed) * 1_000;
  const orbitDistanceMeters = Number(orbitDistance) * 1_000;
  const hasValidOrbit =
    Number.isFinite(orbitSpeedMetersPerSecond) &&
    orbitSpeedMetersPerSecond > 0 &&
    Number.isFinite(orbitDistanceMeters) &&
    orbitDistanceMeters >= 0;

  useEffect(() => {
    const stopThrust = () => {
      pressedDriveKeys.current.clear();
      onManualThrustChange?.(undefined, manualPower);
    };
    const updateThrust = () => {
      const keys = pressedDriveKeys.current;
      const direction = {
        x: Number(keys.has('a')) - Number(keys.has('d')),
        y: Number(keys.has('w')) - Number(keys.has('s')),
      };
      onManualThrustChange?.(
        direction.x === 0 && direction.y === 0 ? undefined : direction,
        manualPower,
      );
    };
    const hasEditableFocus = () => {
      const activeElement = document.activeElement;
      return (
        activeElement instanceof HTMLElement &&
        (activeElement.matches('input, textarea, select') ||
          activeElement.isContentEditable)
      );
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      const key = event.code.replace('Key', '').toLowerCase();
      if (
        event.repeat ||
        !['w', 'a', 's', 'd'].includes(key) ||
        hasEditableFocus()
      ) {
        return;
      }
      event.preventDefault();
      pressedDriveKeys.current.add(key);
      updateThrust();
    };
    const handleKeyUp = (event: KeyboardEvent) => {
      const key = event.code.replace('Key', '').toLowerCase();
      if (!['w', 'a', 's', 'd'].includes(key)) return;
      event.preventDefault();
      pressedDriveKeys.current.delete(key);
      updateThrust();
    };
    const handleBlur = () => stopThrust();
    const handleFocusIn = () => {
      if (hasEditableFocus()) stopThrust();
    };

    window.addEventListener('keydown', handleKeyDown, true);
    window.addEventListener('keyup', handleKeyUp, true);
    window.addEventListener('blur', handleBlur);
    document.addEventListener('focusin', handleFocusIn);
    return () => {
      window.removeEventListener('keydown', handleKeyDown, true);
      window.removeEventListener('keyup', handleKeyUp, true);
      window.removeEventListener('blur', handleBlur);
      document.removeEventListener('focusin', handleFocusIn);
      stopThrust();
    };
  }, [manualPower, onManualThrustChange]);

  const toggleSpeedControl = (control: SpeedControlTab) => {
    setExpandedSpeedControls((expandedControls) => {
      const nextControls = new Set(expandedControls);
      if (nextControls.has(control)) {
        nextControls.delete(control);
      } else {
        nextControls.add(control);
      }
      return nextControls;
    });
  };

  const startEngines = () => {
    if (!canStartBurn || isEngineRunning || !onStartEngines) return;

    setOrbitError('');
    setBurnStartVelocity(currentVelocity);
    onStartEngines(targetSpeedMetersPerSecond, maximumThrustPercentValue);
  };

  const toggleEngines = () => {
    if (isEngineRunning) {
      onStopEngines?.();
      return;
    }

    startEngines();
  };

  const toggleAutoOrbit = () => {
    if (autoOrbit.active) {
      stopSpaceshipAutoOrbit();
      setOrbitError('');
      return;
    }

    if (isEngineRunning || motionState === 'crashed' || !hasValidOrbit) return;

    const started = startSpaceshipAutoOrbit(
      orbitSpeedMetersPerSecond,
      orbitDistanceMeters,
    );
    setOrbitError(
      started ? '' : 'Move within 1,000 km of a planet surface to orbit',
    );
  };

  return (
    <footer className={style.container} aria-label="Ship controls">
      <nav className={style.views} aria-label="Display view">
        {(['navigation', 'ship'] as const).map((view) => (
          <button
            className={style.viewButton}
            data-active={activeView === view}
            key={view}
            onClick={() => onViewChange(view)}
            type="button"
          >
            {view}
          </button>
        ))}
      </nav>

      {activeView === 'navigation' && (
        <section className={style.speedControls} aria-label="Ship features">
          <div className={style.controlTabs} aria-label="Ship features">
            {(
              [
                ['target-speed', 'Target speed'],
                ['auto-orbit', 'Auto orbit'],
                ['manual-drive', 'Manual drive'],
              ] as const
            ).map(([tab, label]) => (
              <button
                id={`footer-${tab}-tab`}
                className={style.controlTab}
                type="button"
                aria-controls={`footer-${tab}-panel`}
                aria-expanded={expandedSpeedControls.has(tab)}
                data-active={expandedSpeedControls.has(tab)}
                key={tab}
                onClick={() => toggleSpeedControl(tab)}
              >
                <span>{label}</span>
                <span aria-hidden="true">
                  {expandedSpeedControls.has(tab) ? '−' : '+'}
                </span>
              </button>
            ))}
          </div>

          <div className={style.controlPanels}>
            {expandedSpeedControls.has('manual-drive') && (
              <div
                id="footer-manual-drive-panel"
                className={style.controlPanel}
                aria-labelledby="footer-manual-drive-tab"
              >
                <fieldset className={style.powerControls}>
                  <legend>Thruster power</legend>
                  <div className={style.powerPresets}>
                    {[10, 25, 50, 100].map((power) => (
                      <button
                        type="button"
                        data-active={manualPower === power}
                        key={power}
                        onClick={() => setManualPower(power)}
                      >
                        {power}%
                      </button>
                    ))}
                  </div>
                  <label htmlFor="footer-manual-power">
                    <span>Custom</span>
                    <span className={style.speedField}>
                      <input
                        id="footer-manual-power"
                        type="number"
                        min="1"
                        max="100"
                        step="1"
                        value={manualPower}
                        onChange={(event) => {
                          const power = event.currentTarget.valueAsNumber;
                          if (Number.isFinite(power)) {
                            setManualPower(Math.min(100, Math.max(1, power)));
                          }
                        }}
                      />
                      <span aria-hidden="true">%</span>
                    </span>
                  </label>
                </fieldset>
              </div>
            )}
            {expandedSpeedControls.has('auto-orbit') && (
              <div
                id="footer-auto-orbit-panel"
                className={style.controlPanel}
                aria-labelledby="footer-auto-orbit-tab"
              >
                <div className={style.speedInputs}>
                  <label htmlFor="footer-orbit-speed">
                    <span>Orbit speed</span>
                    <span className={style.speedField}>
                      <input
                        id="footer-orbit-speed"
                        type="number"
                        min="0.1"
                        step="0.1"
                        value={orbitSpeed}
                        disabled={autoOrbit.active || isEngineRunning}
                        onChange={(event) => {
                          setOrbitSpeed(event.currentTarget.value);
                          setOrbitError('');
                        }}
                      />
                      <span aria-hidden="true">km/s</span>
                    </span>
                  </label>
                  <label htmlFor="footer-orbit-distance">
                    <span>Orbit distance</span>
                    <span className={style.speedField}>
                      <input
                        id="footer-orbit-distance"
                        type="number"
                        min="0"
                        step="10"
                        value={orbitDistance}
                        disabled={autoOrbit.active || isEngineRunning}
                        onChange={(event) => {
                          setOrbitDistance(event.currentTarget.value);
                          setOrbitError('');
                        }}
                      />
                      <span aria-hidden="true">km</span>
                    </span>
                  </label>
                </div>
                <div className={style.orbitActions}>
                  <button
                    type="button"
                    data-running={autoOrbit.active}
                    disabled={
                      autoOrbit.active
                        ? false
                        : isEngineRunning ||
                          motionState === 'crashed' ||
                          !hasValidOrbit
                    }
                    onClick={toggleAutoOrbit}
                  >
                    {autoOrbit.active ? 'Stop orbit' : 'Auto orbit'}
                  </button>
                  <output className={style.orbitStatus}>
                    {autoOrbit.active
                      ? `Autopilot burning near ${autoOrbit.planetName ?? 'planet'}`
                      : orbitError || 'Requires surface range < 1,000 km'}
                  </output>
                </div>
              </div>
            )}
            {expandedSpeedControls.has('target-speed') && (
              <div
                id="footer-target-speed-panel"
                className={style.controlPanel}
                aria-labelledby="footer-target-speed-tab"
              >
                <div className={style.speedInputs}>
                  <label htmlFor="footer-target-speed">
                    <span>Target speed</span>
                    <span className={style.speedField}>
                      <input
                        id="footer-target-speed"
                        type="number"
                        min="0"
                        step="0.1"
                        value={targetSpeed}
                        disabled={isEngineRunning}
                        onChange={(event) =>
                          setTargetSpeed(event.currentTarget.value)
                        }
                      />
                      <span aria-hidden="true">km/s</span>
                    </span>
                  </label>
                  <label htmlFor="footer-maximum-thrust">
                    <span>Max thrust</span>
                    <span className={style.speedField}>
                      <input
                        id="footer-maximum-thrust"
                        type="number"
                        min="1"
                        max="100"
                        step="1"
                        value={maximumThrustPercent}
                        disabled={isEngineRunning}
                        onChange={(event) =>
                          setMaximumThrustPercent(event.currentTarget.value)
                        }
                      />
                      <span aria-hidden="true">%</span>
                    </span>
                  </label>
                </div>
                <div className={style.speedActions}>
                  <button
                    type="button"
                    data-active={isSelectingTargetDirection}
                    onClick={onToggleTargetDirectionSelection}
                  >
                    {isSelectingTargetDirection
                      ? 'Cancel target direction'
                      : 'Set target direction'}
                  </button>
                  <div className={style.speedMetrics}>
                    <span className={style.speedMetric}>
                      <small>Acceleration</small>
                      {hasValidBurn ? formatAcceleration(acceleration) : '—'}
                    </span>
                    <span className={style.speedMetric}>
                      <small>
                        {burnRemainingSeconds !== undefined
                          ? 'Time remaining'
                          : 'Time'}
                      </small>
                      {burnTimeSeconds !== undefined
                        ? formatDuration(burnTimeSeconds)
                        : '—'}
                    </span>
                  </div>
                  <button
                    type="button"
                    data-running={isEngineRunning}
                    disabled={
                      isEngineRunning
                        ? !onStopEngines
                        : !canStartBurn || !onStartEngines
                    }
                    onClick={toggleEngines}
                  >
                    {isEngineRunning ? 'Stop engines' : 'Start engines'}
                  </button>
                </div>
              </div>
            )}
          </div>
        </section>
      )}

      <dl className={style.telemetry} aria-label="Ship telemetry">
        <div className={style.readout}>
          <dt>State</dt>
          <dd>{motionState}</dd>
        </div>
        <div className={style.readout}>
          <dt>Engine power</dt>
          <dd>
            {formatPercentage(currentEnginePowerPercent)} /{' '}
            {formatForce(maxEngineThrustNewtons)}
          </dd>
        </div>
        <div className={style.readout}>
          <dt>Fuel</dt>
          <dd>
            {formatImpulse(fuelKns * 1_000)} /{' '}
            {formatImpulse(INITIAL_SPACESHIP_FUEL_KNS * 1_000)}
          </dd>
        </div>
        <div className={style.readout}>
          <dt>Speed</dt>
          <dd>{formatSpeed(speed)}</dd>
        </div>
      </dl>
    </footer>
  );
}
