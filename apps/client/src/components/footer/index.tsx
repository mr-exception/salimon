import {
  useEffect,
  useLayoutEffect,
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
  MINING_BASE_EFFICIENCY_KNS,
  MINING_BASE_DURABILITY_KN,
  MINING_BASE_RANGE_METERS,
  MINING_DURABILITY_PER_LEVEL_KN,
  MINING_RANGE_LEVEL_MULTIPLIER,
  INVENTORY_MATERIALS,
  MODULE_GRID_SIZE,
  MODULE_RESEARCH,
  THRUSTER_BASE_DURABILITY,
  THRUSTER_BASE_POWER_PERCENT,
  THRUSTER_LEVEL_MULTIPLIER,
  placeModule,
  setModuleActive,
  spendInventory,
  unlockModule,
  upgradeModuleAttribute,
  useInventory,
  useModules,
  useSpaceshipActiveFeature,
  useSpaceshipFuelKns,
  useSpaceshipHullDurability,
  useSpaceshipMotionState,
  useSpaceshipSpeed,
  useSpaceshipTargetDirection,
  useSpaceshipThrusterDurability,
  type Inventory,
  type ModuleAttribute,
  type ModuleType,
  type ShipModule,
} from '@store';
import type { InventoryMaterial } from '@repo/types';
import {
  formatAcceleration,
  formatDuration,
  formatImpulse,
  formatPercentage,
  formatSpeed,
} from '../../utils';

type FooterProps = {
  isEngineRunning?: boolean;
  isMeasuring?: boolean;
  isSelectingTargetDirection?: boolean;
  onStartEngines?: (targetSpeed: number, maximumThrustPercent: number) => void;
  onStopEngines?: () => void;
  onToggleMeasuring?: () => void;
  onOpenCommunications?: () => void;
  unreadMessageCount?: number;
  onToggleTargetDirectionSelection?: () => void;
  onPredictionChange?: (active: boolean, seconds: number) => void;
};

type SpeedControlTab =
  | 'target-speed'
  | 'maintenance'
  | 'prediction'
  | 'modules'
  | 'research';

type Position = {
  x: number;
  y: number;
};

const CONTROL_LABELS: Record<SpeedControlTab, string> = {
  'target-speed': 'Target speed',
  maintenance: 'Ship durability',
  prediction: 'Prediction',
  modules: 'Modules',
  research: 'Research',
};

const PANEL_MARGIN = 16;
const PANEL_TOP = 140;
const PANEL_BOTTOM = 88;

const PANEL_PLACEMENTS: Record<
  SpeedControlTab,
  { horizontal: 'left' | 'right'; vertical: 'top' | 'bottom' }
> = {
  'target-speed': { horizontal: 'left', vertical: 'top' },
  maintenance: { horizontal: 'right', vertical: 'bottom' },
  prediction: { horizontal: 'left', vertical: 'bottom' },
  modules: { horizontal: 'left', vertical: 'top' },
  research: { horizontal: 'right', vertical: 'top' },
};

function FeatureIcon({ feature }: { feature: SpeedControlTab }) {
  if (feature === 'modules') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M4 4h6v6H4zM14 4h6v6h-6zM4 14h6v6H4zM14 14h6v6h-6z" />
      </svg>
    );
  }
  if (feature === 'research') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M10 4h4M12 4v6l5 8a2 2 0 0 1-1.7 3H8.7A2 2 0 0 1 7 18l5-8" />
        <path d="M9 16h6" />
      </svg>
    );
  }
  if (feature === 'prediction') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <circle cx="6" cy="17" r="2" />
        <circle cx="18" cy="7" r="2" />
        <path d="M8 16c4-.8 5-6.2 8-8M13 7h3v3" />
      </svg>
    );
  }
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

  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 3v18M3 12h18" />
      <path d="m12 3-2.5 2.5M12 3l2.5 2.5M21 12l-2.5-2.5M21 12l-2.5 2.5M12 21l-2.5-2.5M12 21l2.5-2.5M3 12l2.5-2.5M3 12l2.5 2.5" />
    </svg>
  );
}

const MODULE_LABELS: Record<ModuleType, string> = {
  mining: 'Mining',
  thruster: 'Thruster',
};

const ATTRIBUTE_LABELS: Record<ModuleAttribute, string> = {
  efficiency: 'Efficiency',
  durability: 'Durability',
  range: 'Range',
  power: 'Power',
};

function getModuleAttributeValue(
  module: ShipModule,
  attribute: ModuleAttribute,
) {
  const level = module.levels[attribute] ?? 1;
  if (module.type === 'mining' && attribute === 'efficiency') {
    return `${MINING_BASE_EFFICIENCY_KNS * level} KN/s`;
  }
  if (module.type === 'mining' && attribute === 'durability') {
    return `${
      MINING_BASE_DURABILITY_KN +
      Math.max(0, level - 1) * MINING_DURABILITY_PER_LEVEL_KN
    } KN`;
  }
  if (module.type === 'mining' && attribute === 'range') {
    return `${Math.round(
      MINING_BASE_RANGE_METERS *
        (1 + Math.max(0, level - 1) * MINING_RANGE_LEVEL_MULTIPLIER),
    ).toLocaleString()} m`;
  }
  if (module.type === 'thruster' && attribute === 'power') {
    return `${Math.round(
      THRUSTER_BASE_POWER_PERCENT *
        (1 + Math.max(0, level - 1) * THRUSTER_LEVEL_MULTIPLIER),
    )}%`;
  }
  if (module.type === 'thruster' && attribute === 'durability') {
    return `${Math.round(
      THRUSTER_BASE_DURABILITY *
        (1 + Math.max(0, level - 1) * THRUSTER_LEVEL_MULTIPLIER),
    )}`;
  }

  return String(level);
}

function getUpgradeCost(
  module: ShipModule,
  attribute: ModuleAttribute,
): Partial<Inventory> {
  const nextLevel = (module.levels[attribute] ?? 1) + 1;
  if (module.type === 'mining' && attribute === 'efficiency') {
    return { iron: 20 * nextLevel, silicates: 8 * nextLevel, ice: 0 };
  }
  if (module.type === 'mining' && attribute === 'durability') {
    return {
      iron: 14 * nextLevel,
      silicates: 12 * nextLevel,
      ice: 4 * nextLevel,
    };
  }
  if (module.type === 'mining' && attribute === 'range') {
    return {
      iron: 18 * nextLevel,
      silicates: 10 * nextLevel,
      ice: 6 * nextLevel,
    };
  }
  if (module.type === 'thruster' && attribute === 'power') {
    return {
      iron: 45 * nextLevel,
      silicates: 30 * nextLevel,
      ice: 12 * nextLevel,
    };
  }
  if (module.type === 'thruster' && attribute === 'durability') {
    return {
      iron: 32 * nextLevel,
      silicates: 34 * nextLevel,
      ice: 8 * nextLevel,
    };
  }

  return { iron: 0, silicates: 0, ice: 0 };
}

function canAfford(inventory: Inventory, cost: Partial<Inventory>) {
  return INVENTORY_MATERIALS.every(
    (material) => inventory[material] >= (cost[material] ?? 0),
  );
}

const COST_LABELS: Record<InventoryMaterial, string> = {
  iron: 'Fe',
  silicates: 'Si',
  ice: 'Ice',
  silver: 'Ag',
  carbon: 'C',
  gold: 'Au',
  hydrogen: 'H',
  nitrogen: 'N',
};

function formatCost(cost: Partial<Inventory>) {
  const entries = INVENTORY_MATERIALS.filter(
    (material) => (cost[material] ?? 0) > 0,
  );
  if (entries.length === 0) return 'Free';

  return entries
    .map((material) => `${COST_LABELS[material]} ${cost[material]}`)
    .join(' / ');
}

function getModuleAttributes(type: ModuleType): ModuleAttribute[] {
  return type === 'mining'
    ? ['efficiency', 'durability', 'range']
    : ['power', 'durability'];
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

function MeasuringIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 20 20 4M12 4h8v8" />
      <path d="M4 15v5h5" />
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
  const [position, setPosition] = useState<Position>({
    x: PANEL_MARGIN,
    y: PANEL_TOP,
  });

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

  useLayoutEffect(() => {
    const setInitialPosition = () => {
      const panel = panelRef.current;
      if (!panel) return;

      const placement = PANEL_PLACEMENTS[control];
      setPosition(
        clampPosition({
          x:
            placement.horizontal === 'left'
              ? PANEL_MARGIN
              : window.innerWidth - panel.offsetWidth - PANEL_MARGIN,
          y:
            placement.vertical === 'top'
              ? PANEL_TOP
              : window.innerHeight - panel.offsetHeight - PANEL_BOTTOM,
        }),
      );
    };

    setInitialPosition();
    window.addEventListener('resize', setInitialPosition);
    return () => window.removeEventListener('resize', setInitialPosition);
  }, [control]);

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
  isMeasuring = false,
  isSelectingTargetDirection = false,
  onStartEngines,
  onStopEngines,
  onToggleMeasuring,
  onOpenCommunications,
  unreadMessageCount = 0,
  onToggleTargetDirectionSelection,
  onPredictionChange,
}: FooterProps) {
  const speed = useSpaceshipSpeed();
  const fuelKns = useSpaceshipFuelKns();
  const hullDurability = useSpaceshipHullDurability();
  const thrusterDurability = useSpaceshipThrusterDurability();
  const motionState = useSpaceshipMotionState();
  const targetDirection = useSpaceshipTargetDirection();
  const activeFeature = useSpaceshipActiveFeature();
  const inventory = useInventory();
  const modules = useModules();
  const [targetSpeed, setTargetSpeed] = useState('10');
  const [maximumThrustPercent, setMaximumThrustPercent] = useState('100');
  const [predictionAmount, setPredictionAmount] = useState('2');
  const [predictionUnit, setPredictionUnit] = useState<'s' | 'm' | 'h'>('m');
  const [isPredictionActive, setIsPredictionActive] = useState(false);
  const [selectedModuleId, setSelectedModuleId] = useState<string | undefined>(
    'mining-module-1',
  );
  const [expandedSpeedControls, setExpandedSpeedControls] = useState(
    () => new Set<SpeedControlTab>(),
  );
  const selectedModule = modules.find(
    (module) => module.id === selectedModuleId,
  );
  const targetSpeedMetersPerSecond = Number(targetSpeed) * 1_000;
  const maximumThrustPercentValue = Number(maximumThrustPercent);
  const activeTargetSpeed =
    activeFeature?.type === 'target-speed' ? activeFeature : undefined;
  const burnTimeSeconds = activeTargetSpeed
    ? Math.max(
        0,
        activeTargetSpeed.durationSeconds - activeTargetSpeed.elapsedSeconds,
      )
    : undefined;
  const acceleration = activeTargetSpeed?.maximumAcceleration ?? Number.NaN;
  const hasValidBurn =
    targetDirection !== undefined &&
    Number.isFinite(targetSpeedMetersPerSecond) &&
    targetSpeedMetersPerSecond >= 0 &&
    Number.isFinite(maximumThrustPercentValue) &&
    maximumThrustPercentValue > 0 &&
    maximumThrustPercentValue <= 100;
  const canStartBurn =
    motionState !== 'crashed' &&
    hasValidBurn &&
    fuelKns > 0 &&
    thrusterDurability.some((durability) => durability > 0);
  const currentEnginePowerPercent = activeTargetSpeed
    ? activeTargetSpeed.maximumThrustPercent
    : 0;
  const predictionSeconds =
    Number(predictionAmount) *
    ({ s: 1, m: 60, h: 3_600 } as const)[predictionUnit];
  const hasValidPrediction =
    Number.isFinite(predictionSeconds) && predictionSeconds > 0;

  useEffect(() => {
    if (isPredictionActive && hasValidPrediction) {
      onPredictionChange?.(true, predictionSeconds);
    } else {
      onPredictionChange?.(false, 0);
    }
  }, [
    hasValidPrediction,
    isPredictionActive,
    onPredictionChange,
    predictionSeconds,
  ]);

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

    onStartEngines(targetSpeedMetersPerSecond, maximumThrustPercentValue);
  };

  const toggleEngines = () => {
    if (isEngineRunning) {
      onStopEngines?.();
      return;
    }

    startEngines();
  };

  const selectModuleGridCell = (x: number, y: number) => {
    const cellModule = modules.find(
      (module) => module.position.x === x && module.position.y === y,
    );
    if (cellModule) {
      setSelectedModuleId(cellModule.id);
      return;
    }

    if (selectedModuleId) {
      placeModule(selectedModuleId, { x, y });
    }
  };

  const unlockResearchModule = (type: ModuleType, cost: Partial<Inventory>) => {
    if (!spendInventory(cost)) return;
    unlockModule(type);
  };

  const upgradeResearchAttribute = (
    module: ShipModule,
    attribute: ModuleAttribute,
  ) => {
    const cost = getUpgradeCost(module, attribute);
    if (!spendInventory(cost)) return;
    upgradeModuleAttribute(module.id, attribute);
  };

  return (
    <footer className={style.container} aria-label="Ship controls">
      <section className={style.speedControls} aria-label="Ship features">
        <div className={style.controlTabs}>
          {(
            [
              ['target-speed', 'Target speed'],
              ['modules', 'Modules'],
              ['research', 'Research'],
              ['maintenance', 'Ship durability'],
              ['prediction', 'Prediction'],
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
            aria-label="Measuring"
            aria-pressed={isMeasuring}
            data-active={isMeasuring}
            onClick={onToggleMeasuring}
          >
            <MeasuringIcon />
            <span className={style.tooltip} role="tooltip">
              Measuring
            </span>
          </button>
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
          {expandedSpeedControls.has('modules') && (
            <DraggablePanel
              control="modules"
              onClose={() => toggleSpeedControl('modules')}
            >
              <div className={style.moduleGridPanel}>
                <div
                  className={style.moduleGrid}
                  style={{
                    gridTemplateColumns: `repeat(${MODULE_GRID_SIZE}, 1fr)`,
                  }}
                >
                  {Array.from(
                    { length: MODULE_GRID_SIZE * MODULE_GRID_SIZE },
                    (_, index) => {
                      const x = index % MODULE_GRID_SIZE;
                      const y = Math.floor(index / MODULE_GRID_SIZE);
                      const module = modules.find(
                        (candidate) =>
                          candidate.position.x === x &&
                          candidate.position.y === y,
                      );

                      return (
                        <button
                          key={`${x}:${y}`}
                          type="button"
                          aria-label={
                            module
                              ? `${module.name} at ${x + 1}, ${y + 1}`
                              : `Empty module cell ${x + 1}, ${y + 1}`
                          }
                          data-occupied={module ? 'true' : 'false'}
                          data-selected={module?.id === selectedModuleId}
                          onClick={() => selectModuleGridCell(x, y)}
                        >
                          {module ? MODULE_LABELS[module.type].slice(0, 1) : ''}
                        </button>
                      );
                    },
                  )}
                </div>
                <div className={style.moduleInspector}>
                  {selectedModule ? (
                    <>
                      <header>
                        <span>{selectedModule.name}</span>
                        <small>
                          {selectedModule.position.x + 1},
                          {selectedModule.position.y + 1}
                        </small>
                      </header>
                      <dl>
                        {getModuleAttributes(selectedModule.type).map(
                          (attribute) => (
                            <div key={attribute}>
                              <dt>{ATTRIBUTE_LABELS[attribute]}</dt>
                              <dd>
                                {getModuleAttributeValue(
                                  selectedModule,
                                  attribute,
                                )}
                              </dd>
                            </div>
                          ),
                        )}
                        <div>
                          <dt>Durability left</dt>
                          <dd>{selectedModule.durability.toFixed(0)}</dd>
                        </div>
                      </dl>
                      {selectedModule.type === 'mining' && (
                        <button
                          type="button"
                          data-active={selectedModule.active}
                          disabled={selectedModule.durability <= 0}
                          onClick={() =>
                            setModuleActive(
                              selectedModule.id,
                              !selectedModule.active,
                            )
                          }
                        >
                          {selectedModule.active ? 'Deactivate' : 'Activate'}
                        </button>
                      )}
                    </>
                  ) : (
                    <span className={style.emptyModuleSelection}>
                      Select a module
                    </span>
                  )}
                </div>
              </div>
            </DraggablePanel>
          )}
          {expandedSpeedControls.has('research') && (
            <DraggablePanel
              control="research"
              onClose={() => toggleSpeedControl('research')}
            >
              <div className={style.researchList}>
                {MODULE_RESEARCH.map((research) => {
                  const module = modules.find(
                    (candidate) => candidate.type === research.module,
                  );
                  const unlocked = module !== undefined;

                  return (
                    <section
                      className={style.researchItem}
                      key={research.module}
                    >
                      <header>
                        <span>{research.name}</span>
                        <small>
                          {unlocked ? 'Unlocked' : formatCost(research.cost)}
                        </small>
                      </header>
                      {!unlocked ? (
                        <button
                          type="button"
                          disabled={!canAfford(inventory, research.cost)}
                          onClick={() =>
                            unlockResearchModule(research.module, research.cost)
                          }
                        >
                          Unlock
                        </button>
                      ) : (
                        <div className={style.researchUpgrades}>
                          {getModuleAttributes(module.type).map((attribute) => {
                            const cost = getUpgradeCost(module, attribute);
                            return (
                              <div key={attribute}>
                                <span>
                                  {ATTRIBUTE_LABELS[attribute]} L
                                  {module.levels[attribute] ?? 1}
                                </span>
                                <small>{formatCost(cost)}</small>
                                <button
                                  type="button"
                                  disabled={!canAfford(inventory, cost)}
                                  onClick={() =>
                                    upgradeResearchAttribute(module, attribute)
                                  }
                                >
                                  Improve
                                </button>
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </section>
                  );
                })}
              </div>
            </DraggablePanel>
          )}
          {expandedSpeedControls.has('prediction') && (
            <DraggablePanel
              control="prediction"
              onClose={() => toggleSpeedControl('prediction')}
            >
              <div className={style.predictionControls}>
                <label htmlFor="footer-prediction-amount">
                  <span>Time ahead</span>
                  <span className={style.predictionField}>
                    <input
                      id="footer-prediction-amount"
                      type="number"
                      min="1"
                      step="1"
                      value={predictionAmount}
                      onChange={(event) =>
                        setPredictionAmount(event.currentTarget.value)
                      }
                    />
                    <select
                      aria-label="Prediction time unit"
                      value={predictionUnit}
                      onChange={(event) =>
                        setPredictionUnit(
                          event.currentTarget.value as 's' | 'm' | 'h',
                        )
                      }
                    >
                      <option value="s">s</option>
                      <option value="m">m</option>
                      <option value="h">h</option>
                    </select>
                  </span>
                </label>
                <button
                  type="button"
                  data-active={isPredictionActive}
                  disabled={!hasValidPrediction}
                  onClick={() => setIsPredictionActive((active) => !active)}
                >
                  {isPredictionActive ? 'Deactivate' : 'Activate'}
                </button>
              </div>
            </DraggablePanel>
          )}
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
                  </div>
                ))}
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
                    {activeTargetSpeed ? formatAcceleration(acceleration) : '—'}
                  </span>
                  <span className={style.speedMetric}>
                    <small>Time remaining</small>
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
          <dd>{formatPercentage(currentEnginePowerPercent)}</dd>
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
