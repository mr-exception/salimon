import {
  useEffect,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from 'react';
import style from './style.module.css';
import {
  INITIAL_SPACESHIP_FUEL_KNS,
  MAX_HULL_DURABILITY,
  MAX_THRUSTER_DURABILITY,
  MAX_ENGINE_THRUST_KN,
  SPACESHIP_MASS_KG,
  getSpaceshipBurnPlan,
  getSpaceshipBurnAcceleration,
  getSpaceshipBurnRemainingSeconds,
  getSpaceshipVelocity,
  startSpaceshipAutoOrbit,
  stopSpaceshipAutoOrbit,
  repairSpaceshipHull,
  repairSpaceshipThruster,
  useSpaceshipAutoOrbit,
  useSpaceshipFuelKns,
  useSpaceshipHullDurability,
  useSpaceshipMotionState,
  useSpaceshipSpeed,
  useSpaceshipTargetDirection,
  useSpaceshipThrusterDurability,
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
  isEngineRunning?: boolean;
  isSelectingTargetDirection?: boolean;
  showMovementHint?: boolean;
  onStartEngines?: (targetSpeed: number, maximumThrustPercent: number) => void;
  onStopEngines?: () => void;
  onManualThrustChange?: (
    direction: { x: number; y: number } | undefined,
    power: number,
  ) => void;
  onOpenCommunications?: () => void;
  unreadMessageCount?: number;
  onToggleTargetDirectionSelection?: () => void;
};

type SpeedControlTab =
  | 'target-speed'
  | 'auto-orbit'
  | 'manual-drive'
  | 'maintenance';

type Position = {
  x: number;
  y: number;
};

const CONTROL_LABELS: Record<SpeedControlTab, string> = {
  'target-speed': 'Target speed',
  'auto-orbit': 'Auto orbit',
  'manual-drive': 'Manual drive',
  maintenance: 'Ship durability',
};

const INITIAL_PANEL_POSITIONS: Record<SpeedControlTab, Position> = {
  'target-speed': { x: 16, y: 140 },
  'auto-orbit': { x: 32, y: 156 },
  'manual-drive': { x: 48, y: 172 },
  maintenance: { x: 64, y: 188 },
};

function FeatureIcon({ feature }: { feature: SpeedControlTab }) {
  if (feature === 'maintenance') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="m14.7 6.3 3-3a4 4 0 0 1-5 5l-7.4 7.4a2.1 2.1 0 0 0 3 3l7.4-7.4a4 4 0 0 1 5-5l-3 3" />
      </svg>
    );
  }
  if (feature === 'target-speed') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M4 15a8 8 0 1 1 16 0" />
        <path d="m12 15 4-5" />
        <path d="M8 18h8" />
      </svg>
    );
  }

  if (feature === 'auto-orbit') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="12" cy="12" r="3" />
        <path d="M4.6 9.1c-1.4 2.4-1.6 4.7-.3 5.9 1.9 1.9 6.5.4 10.3-3.4s5.3-8.4 3.4-10.3" />
        <path d="m17.8 1.2.4 4-4-.4" />
        <path d="M19.4 14.9c1.4-2.4 1.6-4.7.3-5.9" />
      </svg>
    );
  }

  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 3v18M3 12h18" />
      <path d="m12 3-2.5 2.5M12 3l2.5 2.5M21 12l-2.5-2.5M21 12l-2.5 2.5M12 21l-2.5-2.5M12 21l2.5-2.5M3 12l2.5-2.5M3 12l2.5 2.5" />
    </svg>
  );
}

function MoveIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2v20M2 12h20" />
      <path d="m12 2-3 3m3-3 3 3m7 7-3-3m3 3-3 3m-7 7-3-3m3 3 3-3M2 12l3-3m-3 3 3 3" />
    </svg>
  );
}

function CommunicationsIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 5h16v11H9l-5 4V5Z" />
      <path d="M8 9h8M8 12h5" />
    </svg>
  );
}

function DraggablePanel({
  children,
  control,
  onClose,
}: {
  children: ReactNode;
  control: SpeedControlTab;
  onClose: () => void;
}) {
  const panelRef = useRef<HTMLDivElement>(null);
  const dragRef = useRef<{
    pointerId: number;
    offsetX: number;
    offsetY: number;
  } | null>(null);
  const [position, setPosition] = useState(INITIAL_PANEL_POSITIONS[control]);

  const clampPosition = (nextPosition: Position) => {
    const panel = panelRef.current;
    const width = panel?.offsetWidth ?? 320;
    const height = panel?.offsetHeight ?? 180;

    return {
      x: Math.min(
        Math.max(8, nextPosition.x),
        Math.max(8, window.innerWidth - width - 8),
      ),
      y: Math.min(
        Math.max(8, nextPosition.y),
        Math.max(8, window.innerHeight - height - 8),
      ),
    };
  };

  const handlePointerDown = (event: ReactPointerEvent<HTMLButtonElement>) => {
    const panelBounds = panelRef.current?.getBoundingClientRect();
    if (!panelBounds) return;

    event.currentTarget.setPointerCapture(event.pointerId);
    dragRef.current = {
      pointerId: event.pointerId,
      offsetX: event.clientX - panelBounds.left,
      offsetY: event.clientY - panelBounds.top,
    };
  };

  const handlePointerMove = (event: ReactPointerEvent<HTMLButtonElement>) => {
    const drag = dragRef.current;
    if (!drag || drag.pointerId !== event.pointerId) return;

    setPosition(
      clampPosition({
        x: event.clientX - drag.offsetX,
        y: event.clientY - drag.offsetY,
      }),
    );
  };

  const stopDragging = (event: ReactPointerEvent<HTMLButtonElement>) => {
    if (dragRef.current?.pointerId !== event.pointerId) return;
    dragRef.current = null;
    event.currentTarget.releasePointerCapture(event.pointerId);
  };

  const handleMoveKeyDown = (event: ReactKeyboardEvent<HTMLButtonElement>) => {
    const movement = event.shiftKey ? 40 : 10;
    const offsets: Partial<Record<string, Position>> = {
      ArrowUp: { x: 0, y: -movement },
      ArrowRight: { x: movement, y: 0 },
      ArrowDown: { x: 0, y: movement },
      ArrowLeft: { x: -movement, y: 0 },
    };
    const offset = offsets[event.key];
    if (!offset) return;

    event.preventDefault();
    setPosition((current) =>
      clampPosition({
        x: current.x + offset.x,
        y: current.y + offset.y,
      }),
    );
  };

  return (
    <div
      ref={panelRef}
      id={`footer-${control}-panel`}
      className={style.controlDialog}
      role="dialog"
      aria-labelledby={`footer-${control}-title`}
      style={{ left: position.x, top: position.y }}
    >
      <header className={style.dialogHeader}>
        <button
          className={style.dragHandle}
          type="button"
          aria-label={`Move ${CONTROL_LABELS[control]} dialog`}
          title="Drag to move"
          onKeyDown={handleMoveKeyDown}
          onPointerDown={handlePointerDown}
          onPointerMove={handlePointerMove}
          onPointerUp={stopDragging}
          onPointerCancel={stopDragging}
        >
          <MoveIcon />
        </button>
        <h2 id={`footer-${control}-title`}>{CONTROL_LABELS[control]}</h2>
        <button
          className={style.closeDialog}
          type="button"
          aria-label={`Close ${CONTROL_LABELS[control]}`}
          onClick={onClose}
        >
          ×
        </button>
      </header>
      <div className={style.dialogContent}>{children}</div>
    </div>
  );
}

export function Footer({
  isEngineRunning = false,
  isSelectingTargetDirection = false,
  showMovementHint = false,
  onStartEngines,
  onStopEngines,
  onManualThrustChange,
  onOpenCommunications,
  unreadMessageCount = 0,
  onToggleTargetDirectionSelection,
}: FooterProps) {
  const speed = useSpaceshipSpeed();
  const fuelKns = useSpaceshipFuelKns();
  const hullDurability = useSpaceshipHullDurability();
  const thrusterDurability = useSpaceshipThrusterDurability();
  const motionState = useSpaceshipMotionState();
  const targetDirection = useSpaceshipTargetDirection();
  const autoOrbit = useSpaceshipAutoOrbit();
  const [targetSpeed, setTargetSpeed] = useState('10');
  const [maximumThrustPercent, setMaximumThrustPercent] = useState('100');
  const [orbitSpeed, setOrbitSpeed] = useState('7.8');
  const [orbitDistance, setOrbitDistance] = useState('400');
  const [orbitError, setOrbitError] = useState('');
  const [manualPower, setManualPower] = useState(25);
  const [isMovementHintVisible, setIsMovementHintVisible] =
    useState(showMovementHint);
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
  const canStartBurn =
    motionState !== 'crashed' &&
    hasValidBurn &&
    fuelKns > 0 &&
    thrusterDurability.some((durability) => durability > 0);
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
      setIsMovementHintVisible(false);
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
      {isMovementHintVisible && (
        <aside className={style.movementHint} aria-label="Movement hint">
          <span>Hold</span>
          <span className={style.movementKeys} aria-label="W A S D">
            {['W', 'A', 'S', 'D'].map((key) => (
              <kbd key={key}>{key}</kbd>
            ))}
          </span>
          <span>to fire thrusters and move</span>
        </aside>
      )}

      <section className={style.speedControls} aria-label="Ship features">
        <div className={style.controlTabs}>
          {(
            [
              ['target-speed', 'Target speed'],
              ['auto-orbit', 'Auto orbit'],
              ['manual-drive', 'Manual drive'],
              ['maintenance', 'Ship durability'],
            ] as const
          ).map(([tab, label]) => (
            <button
              id={`footer-${tab}-tab`}
              className={style.controlTab}
              type="button"
              aria-controls={`footer-${tab}-panel`}
              aria-expanded={expandedSpeedControls.has(tab)}
              aria-label={label}
              data-active={expandedSpeedControls.has(tab)}
              key={tab}
              onClick={() => toggleSpeedControl(tab)}
            >
              <FeatureIcon feature={tab} />
              <span className={style.tooltip} role="tooltip">
                {label}
              </span>
            </button>
          ))}
          <button
            className={style.controlTab}
            type="button"
            aria-label={
              unreadMessageCount > 0
                ? `Communications, ${unreadMessageCount} unread messages`
                : 'Communications'
            }
            onClick={onOpenCommunications}
          >
            <CommunicationsIcon />
            {unreadMessageCount > 0 && (
              <strong className={style.unreadBadge} aria-hidden="true">
                {unreadMessageCount > 99 ? '99+' : unreadMessageCount}
              </strong>
            )}
            <span className={style.tooltip} role="tooltip">
              Communications
            </span>
          </button>
        </div>

        <div className={style.controlPanels}>
          {expandedSpeedControls.has('maintenance') && (
            <DraggablePanel
              control="maintenance"
              onClose={() => toggleSpeedControl('maintenance')}
            >
              <div className={style.durabilityList}>
                <div className={style.durabilityItem}>
                  <span>Hull</span>
                  <meter
                    min="0"
                    max={MAX_HULL_DURABILITY}
                    low={MAX_HULL_DURABILITY * 0.25}
                    value={hullDurability}
                  />
                  <output>
                    {hullDurability.toFixed(2)} / {MAX_HULL_DURABILITY}
                  </output>
                  <button
                    type="button"
                    disabled={hullDurability >= MAX_HULL_DURABILITY}
                    onClick={repairSpaceshipHull}
                  >
                    Repair
                  </button>
                </div>
                {thrusterDurability.map((durability, index) => (
                  <div className={style.durabilityItem} key={index}>
                    <span>Thruster {index + 1}</span>
                    <meter
                      min="0"
                      max={MAX_THRUSTER_DURABILITY}
                      low={MAX_THRUSTER_DURABILITY * 0.25}
                      value={durability}
                    />
                    <output>
                      {durability.toFixed(2)} / {MAX_THRUSTER_DURABILITY}
                    </output>
                    <button
                      type="button"
                      disabled={durability >= MAX_THRUSTER_DURABILITY}
                      onClick={() => repairSpaceshipThruster(index)}
                    >
                      Repair
                    </button>
                  </div>
                ))}
              </div>
            </DraggablePanel>
          )}
          {expandedSpeedControls.has('manual-drive') && (
            <DraggablePanel
              control="manual-drive"
              onClose={() => toggleSpeedControl('manual-drive')}
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
            </DraggablePanel>
          )}
          {expandedSpeedControls.has('auto-orbit') && (
            <DraggablePanel
              control="auto-orbit"
              onClose={() => toggleSpeedControl('auto-orbit')}
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
            </DraggablePanel>
          )}
          {expandedSpeedControls.has('target-speed') && (
            <DraggablePanel
              control="target-speed"
              onClose={() => toggleSpeedControl('target-speed')}
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
            </DraggablePanel>
          )}
        </div>
      </section>

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
