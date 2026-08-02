import {
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
  type KeyboardEvent as ReactKeyboardEvent,
  type PointerEvent as ReactPointerEvent,
  type CSSProperties,
  type ReactNode,
} from 'react';
import style from './style.module.css';
import {
  FABRICATOR_BLUEPRINTS,
  FUEL_CELL_TIERS,
  REPAIR_KIT_TIERS,
  type FuelCellTier,
  type RepairKitTier,
} from './fabricator-blueprints';
import type { MiningSelection, MiningTelemetry } from '../navigator/game/scene';
import {
  INITIAL_SPACESHIP_FUEL_KNS,
  HULL_DURABILITY_DRAIN_PER_CRASH,
  HULL_DURABILITY_PER_LEVEL,
  MAX_THRUSTER_DURABILITY,
  FABRICATOR_DURABILITY_DRAIN_PER_CRAFT,
  ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL,
  MINING_BASE_RATE_KG_PER_SECOND,
  MINING_BASE_DURABILITY_KG,
  MINING_BASE_RANGE_METERS,
  MINING_DURABILITY_PER_LEVEL_KG,
  MINING_RANGE_LEVEL_MULTIPLIER,
  INVENTORY_MATERIALS,
  MODULE_RESEARCH,
  SPACESHIP_THRUSTER_COUNT,
  SPACESHIP_INVENTORY_CAPACITY_KG,
  THRUSTER_BASE_DURABILITY,
  THRUSTER_BASE_POWER_PERCENT,
  THRUSTER_LEVEL_MULTIPLIER,
  addSpaceshipFuelKns,
  consumeEnergyCoreDurability,
  consumeFabricatorDurability,
  getModuleMaxDurability,
  getSpaceshipMaxHullDurability,
  getSpaceshipSaveBlockReason,
  repairModuleByAmount,
  repairSpaceshipHullByAmount,
  repairSpaceshipThrusterByAmount,
  setModuleActive,
  saveSpaceship,
  spendInventory,
  unlockModule,
  upgradeModuleAttribute,
  upgradeSpaceshipHull,
  useInventory,
  useModules,
  useSpaceshipActiveFeature,
  useSpaceshipActiveThrusters,
  useSpaceshipAbsoluteSpeed,
  useSpaceshipFuelKns,
  useSpaceshipHullDurability,
  useSpaceshipHullLevel,
  useSpaceshipMotionState,
  useSpaceshipThrusterDurability,
  type Inventory,
  type ModuleAttribute,
  type ModuleType,
  type ShipModule,
} from '@store';
import type { InventoryMaterial } from '@repo/types';
import { MAX_ENGINE_THRUST_KN } from '@repo/world';
import {
  formatDistance,
  formatForce,
  formatImpulse,
  formatPercentage,
  formatSiValue,
  formatSpeed,
} from '../../utils';

type FooterProps = {
  isEngineRunning?: boolean;
  isMeasuring?: boolean;
  isMeasurementRelativeToSpaceship?: boolean;
  isMeasurementVelocityAxesSeparated?: boolean;
  isRulerActive?: boolean;
  onStartThrusters?: (
    thrusters: { powerPercent: number; active: boolean }[],
  ) => void;
  onStopEngines?: () => void;
  onToggleMeasuring?: () => void;
  onMeasurementRelativeToSpaceshipChange?: (active: boolean) => void;
  onMeasurementVelocityAxesSeparatedChange?: (active: boolean) => void;
  onToggleRuler?: () => void;
  onOpenCommunications?: () => void;
  onOpenCommunicationThread?: (contactId: string) => void;
  onOpenSearch?: () => void;
  unreadMessageCount?: number;
  unreadMessages?: CommunicationNotification[];
  onPredictionChange?: (active: boolean, seconds: number) => void;
  miningTelemetry?: MiningTelemetry;
  onMiningSelectionChange?: (selection?: MiningSelection) => void;
};

type CommunicationNotification = {
  id: string;
  contactId: string;
  senderName: string;
  text: string;
};

type ManualThrusterInput = {
  powerPercent: string;
  active: boolean;
};

type SpeedControlTab =
  | 'thrusters'
  | 'measuring'
  | 'prediction'
  | 'modules'
  | 'research'
  | 'fabricator';
type DraggableControl = SpeedControlTab | 'mining-status' | 'inventory';

type ModulePanelSelection =
  | { type: 'module'; id: string }
  | { type: 'hull' }
  | { type: 'thruster'; index: number };
type RepairKitInventory = Record<RepairKitTier, number>;
type FuelCellInventory = Record<FuelCellTier, number>;
type RepairDialogTarget =
  | { type: 'module'; id: string }
  | { type: 'hull' }
  | { type: 'thruster'; index: number };

type Position = {
  x: number;
  y: number;
};

const CONTROL_LABELS: Record<DraggableControl, string> = {
  thrusters: 'Thrusters',
  measuring: 'Measuring',
  prediction: 'Prediction',
  modules: 'Modules',
  research: 'Research',
  fabricator: 'Fabricator',
  'mining-status': 'Mining module',
  inventory: 'Inventory',
};

const PANEL_MARGIN = 16;
const PANEL_TOP = 140;
const PANEL_BOTTOM = 88;

const PANEL_PLACEMENTS: Record<
  DraggableControl,
  { horizontal: 'left' | 'right'; vertical: 'top' | 'bottom' }
> = {
  thrusters: { horizontal: 'left', vertical: 'top' },
  measuring: { horizontal: 'left', vertical: 'bottom' },
  prediction: { horizontal: 'left', vertical: 'bottom' },
  modules: { horizontal: 'left', vertical: 'top' },
  research: { horizontal: 'right', vertical: 'top' },
  fabricator: { horizontal: 'right', vertical: 'top' },
  'mining-status': { horizontal: 'right', vertical: 'top' },
  inventory: { horizontal: 'right', vertical: 'bottom' },
};

const THRUSTER_LABELS = ['Top', 'Right', 'Bottom', 'Left'] as const;

function createRepairKitInventory(): RepairKitInventory {
  return { t1: 0 };
}

function createFuelCellInventory(): FuelCellInventory {
  return { t1: 0 };
}

function createManualThrusterInputs() {
  return Array.from({ length: SPACESHIP_THRUSTER_COUNT }, () => ({
    powerPercent: '100',
    active: false,
  }));
}

function createManualThrusterInputsFromSignals(
  thrusters: { powerPercent: number; active: boolean }[],
) {
  return Array.from({ length: SPACESHIP_THRUSTER_COUNT }, (_, index) => {
    const thruster = thrusters[index];
    const powerPercent =
      thruster && Number.isFinite(thruster.powerPercent)
        ? Math.max(0, Math.min(100, Math.round(thruster.powerPercent)))
        : 0;

    return {
      powerPercent: String(powerPercent),
      active: Boolean(thruster?.active) && powerPercent > 0,
    };
  });
}

const MODULE_LABELS: Record<ModuleType, string> = {
  mining: 'Mining',
  thruster: 'Thruster',
  fabricator: 'Fabricator',
  'energy-core': 'Energy Core',
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
    return `${MINING_BASE_RATE_KG_PER_SECOND * level} kg/s`;
  }
  if (module.type === 'mining' && attribute === 'durability') {
    return `${
      MINING_BASE_DURABILITY_KG +
      Math.max(0, level - 1) * MINING_DURABILITY_PER_LEVEL_KG
    } kg`;
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
  if (module.type === 'fabricator' && attribute === 'durability') {
    return `${Math.round(getModuleMaxDurability(module))}`;
  }
  if (module.type === 'energy-core' && attribute === 'durability') {
    return `${Math.round(getModuleMaxDurability(module))}`;
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
  if (module.type === 'fabricator' && attribute === 'durability') {
    return {
      iron: 28 * nextLevel,
      silicates: 18 * nextLevel,
      carbon: 10 * nextLevel,
    };
  }
  if (module.type === 'energy-core' && attribute === 'durability') {
    return {
      iron: 38 * nextLevel,
      silicates: 28 * nextLevel,
      carbon: 14 * nextLevel,
    };
  }

  return { iron: 0, silicates: 0, ice: 0 };
}

function canAfford(inventory: Inventory, cost: Partial<Inventory>) {
  return INVENTORY_MATERIALS.every(
    (material) => inventory[material] >= (cost[material] ?? 0),
  );
}

function getMessagePreview(text: string) {
  const normalized = text.replace(/\s+/g, ' ').trim();
  if (normalized.length <= 46) return normalized;
  return `${normalized.slice(0, 46).trim()}...`;
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
  if (type === 'mining') return ['efficiency', 'durability', 'range'];
  if (type === 'thruster') return ['power', 'durability'];
  return ['durability'];
}

function MoveIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2v20M2 12h20" />
      <path d="m12 2-3 3m3-3 3 3m7 7-3-3m3 3-3 3m-7 7-3-3m3 3 3-3M2 12l3-3m-3 3 3 3" />
    </svg>
  );
}

function clampThrusterAxisValue(value: number) {
  if (!Number.isFinite(value)) return 0;
  return Math.max(-100, Math.min(100, Math.round(value)));
}

function formatSignedThrusterAxisValue(value: number) {
  const clampedValue = clampThrusterAxisValue(value);
  return `${clampedValue > 0 ? '+' : ''}${clampedValue}%`;
}

function getThrusterAxisValue(
  thrusters: { powerPercent: number; active: boolean }[],
  positiveIndex: number,
  negativeIndex: number,
) {
  const positiveThruster = thrusters[positiveIndex];
  const negativeThruster = thrusters[negativeIndex];
  const positivePower =
    positiveThruster?.active && positiveThruster.powerPercent > 0
      ? positiveThruster.powerPercent
      : 0;
  const negativePower =
    negativeThruster?.active && negativeThruster.powerPercent > 0
      ? negativeThruster.powerPercent
      : 0;

  return clampThrusterAxisValue(positivePower - negativePower);
}

function setManualThrusterAxisValue(
  thrusters: ManualThrusterInput[],
  axis: 'horizontal' | 'vertical',
  value: number,
) {
  const positiveIndex = axis === 'horizontal' ? 3 : 2;
  const negativeIndex = axis === 'horizontal' ? 1 : 0;
  const clampedValue = clampThrusterAxisValue(value);
  const powerPercent = String(Math.abs(clampedValue));

  return thrusters.map((thruster, index) => {
    if (index === positiveIndex) {
      return {
        powerPercent,
        active: clampedValue > 0,
      };
    }

    if (index === negativeIndex) {
      return {
        powerPercent,
        active: clampedValue < 0,
      };
    }

    return thruster;
  });
}

function DraggablePanel({
  children,
  control,
  onClose,
}: {
  children: ReactNode;
  control: DraggableControl;
  onClose?: () => void;
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
        {onClose ? (
          <button
            className={style.closeDialog}
            type="button"
            aria-label={`Close ${CONTROL_LABELS[control]}`}
            onClick={onClose}
          >
            ×
          </button>
        ) : (
          <span aria-hidden="true" />
        )}
      </header>
      <div className={style.dialogContent}>{children}</div>
    </div>
  );
}

function MiningStatusPanel({
  telemetry,
  active,
  selectedMiningMaterial,
  inventoryMassKg,
  inventoryCapacityKg,
  onSelectMiningMaterial,
  onToggleMining,
}: {
  telemetry: MiningTelemetry;
  active: boolean;
  selectedMiningMaterial?: MiningSelection;
  inventoryMassKg: number;
  inventoryCapacityKg: number;
  onSelectMiningMaterial: (selection: MiningSelection) => void;
  onToggleMining: () => void;
}) {
  const durabilityPercent =
    telemetry.maxDurability > 0
      ? (telemetry.durability / telemetry.maxDurability) * 100
      : 0;
  const inventoryFull = inventoryMassKg >= inventoryCapacityKg;
  const selectedMaterialAvailable = telemetry.targets.some(
    (target) =>
      target.id === selectedMiningMaterial?.asteroidId &&
      target.materials.some(
        (material) =>
          material.name === selectedMiningMaterial.material &&
          material.massKg > 0,
      ),
  );
  const canStart =
    active ||
    (selectedMiningMaterial !== undefined &&
      selectedMaterialAvailable &&
      telemetry.durability > 0 &&
      !inventoryFull);
  const minedEntries = INVENTORY_MATERIALS.flatMap((material) => {
    const massKg = telemetry.minedMaterials[material] ?? 0;
    return massKg > 0 ? [{ material, massKg }] : [];
  });

  return (
    <div className={style.miningStatusPanel}>
      <section className={style.miningStatusSummary}>
        <div className={style.miningDurability}>
          <span>Durability</span>
          <meter
            min={0}
            max={telemetry.maxDurability}
            value={telemetry.durability}
          />
          <output>
            {formatPercentage(durabilityPercent)} ·{' '}
            {formatSiValue(telemetry.durability, 'kg')}
          </output>
        </div>
        <dl>
          <div>
            <dt>Rate</dt>
            <dd>{formatSiValue(telemetry.rateKgPerSecond, 'kg/s')}</dd>
          </div>
          <div>
            <dt>Range</dt>
            <dd>{formatDistance(telemetry.rangeMeters)}</dd>
          </div>
          <div>
            <dt>Inventory</dt>
            <dd>
              {formatSiValue(inventoryMassKg, 'kg')} /{' '}
              {formatSiValue(inventoryCapacityKg, 'kg')}
            </dd>
          </div>
        </dl>
      </section>

      <section className={style.miningTargetList}>
        <h3>Mining asteroids</h3>
        {telemetry.targets.length > 0 ? (
          <ul>
            {telemetry.targets.map((target) => (
              <li key={target.id} data-active={target.active}>
                <span>{target.name}</span>
                <small>
                  {formatDistance(target.distanceMeters)} ·{' '}
                  {formatSiValue(target.remainingMassKg, 'kg')}
                </small>
                <div className={style.miningMaterialList}>
                  {target.materials.map((material) => {
                    const selected =
                      target.id === selectedMiningMaterial?.asteroidId &&
                      material.name === selectedMiningMaterial.material;
                    return (
                      <button
                        key={material.name}
                        type="button"
                        data-selected={selected}
                        disabled={active}
                        onClick={() =>
                          onSelectMiningMaterial({
                            asteroidId: target.id,
                            material: material.name,
                          })
                        }
                      >
                        <span>{material.name}</span>
                        <small>{formatSiValue(material.massKg, 'kg')}</small>
                      </button>
                    );
                  })}
                </div>
              </li>
            ))}
          </ul>
        ) : (
          <p>No asteroids in range</p>
        )}
      </section>

      <div className={style.miningActions}>
        <button type="button" disabled={!canStart} onClick={onToggleMining}>
          {active ? 'Stop mining' : 'Start mining'}
        </button>
        {inventoryFull && <span>Inventory full</span>}
      </div>

      <section className={style.minedMaterials}>
        <h3>Mined this run</h3>
        {minedEntries.length > 0 ? (
          <dl>
            {minedEntries.map(({ material, massKg }) => (
              <div key={material}>
                <dt>{material}</dt>
                <dd>{formatSiValue(massKg, 'kg')}</dd>
              </div>
            ))}
          </dl>
        ) : (
          <p>No materials extracted yet</p>
        )}
      </section>
    </div>
  );
}

function InventoryPanel({
  inventory,
  inventoryMassKg,
  inventoryCapacityKg,
  fuelCellInventory,
  repairKitInventory,
}: {
  inventory: Inventory;
  inventoryMassKg: number;
  inventoryCapacityKg: number;
  fuelCellInventory: FuelCellInventory;
  repairKitInventory: RepairKitInventory;
}) {
  const materialEntries = INVENTORY_MATERIALS.map((material) => ({
    material,
    massKg: inventory[material],
  }));
  const itemEntries = [
    ...FUEL_CELL_TIERS.flatMap((cell) => {
      const quantity = fuelCellInventory[cell.tier];
      return quantity > 0
        ? [
            {
              key: `fuel-cell:${cell.tier}`,
              name: cell.label,
              detail: `${quantity} stored / ${formatForce(cell.fuelKns * 1_000)} each`,
            },
          ]
        : [];
    }),
    ...REPAIR_KIT_TIERS.flatMap((kit) => {
      const quantity = repairKitInventory[kit.tier];
      return quantity > 0
        ? [
            {
              key: `repair-kit:${kit.tier}`,
              name: kit.label,
              detail: `${quantity} stored / ${kit.repairAmount} durability each`,
            },
          ]
        : [];
    }),
  ];

  return (
    <div className={style.inventoryPanel}>
      <section className={style.inventorySummary}>
        <span>Capacity</span>
        <meter min={0} max={inventoryCapacityKg} value={inventoryMassKg} />
        <output>
          {formatSiValue(inventoryMassKg, 'kg')} /{' '}
          {formatSiValue(inventoryCapacityKg, 'kg')}
        </output>
      </section>

      <section className={style.inventorySection}>
        <h3>Items</h3>
        {itemEntries.length > 0 ? (
          <dl>
            {itemEntries.map((item) => (
              <div key={item.key}>
                <dt>{item.name}</dt>
                <dd>{item.detail}</dd>
              </div>
            ))}
          </dl>
        ) : (
          <p>No produced items stored</p>
        )}
      </section>

      <section className={style.inventorySection}>
        <h3>Materials</h3>
        <dl>
          {materialEntries.map(({ material, massKg }) => (
            <div key={material}>
              <dt>{material}</dt>
              <dd>{formatSiValue(massKg, 'kg')}</dd>
            </div>
          ))}
        </dl>
      </section>
    </div>
  );
}

export function Footer({
  isMeasuring = false,
  isMeasurementRelativeToSpaceship = false,
  isMeasurementVelocityAxesSeparated = false,
  isRulerActive = false,
  onStartThrusters,
  onStopEngines,
  onToggleMeasuring,
  onMeasurementRelativeToSpaceshipChange,
  onMeasurementVelocityAxesSeparatedChange,
  onToggleRuler,
  onOpenCommunications,
  onOpenCommunicationThread,
  onOpenSearch,
  unreadMessageCount = 0,
  unreadMessages = [],
  onPredictionChange,
  miningTelemetry,
  onMiningSelectionChange,
}: FooterProps) {
  const speed = useSpaceshipAbsoluteSpeed();
  const fuelKns = useSpaceshipFuelKns();
  const hullDurability = useSpaceshipHullDurability();
  const hullLevel = useSpaceshipHullLevel();
  const thrusterDurability = useSpaceshipThrusterDurability();
  const motionState = useSpaceshipMotionState();
  const activeFeature = useSpaceshipActiveFeature();
  const activeThrusterSignals = useSpaceshipActiveThrusters();
  const inventory = useInventory();
  const modules = useModules();
  const [manualThrusters, setManualThrusters] = useState(
    createManualThrusterInputs,
  );
  const [manualThrusterAxisFields, setManualThrusterAxisFields] = useState({
    horizontal: '0',
    vertical: '0',
  });
  const [predictionAmount, setPredictionAmount] = useState('2');
  const [predictionUnit, setPredictionUnit] = useState<'s' | 'm' | 'h'>('m');
  const [isPredictionActive, setIsPredictionActive] = useState(false);
  const [isMiningPanelOpen, setIsMiningPanelOpen] = useState(false);
  const [isInventoryPanelOpen, setIsInventoryPanelOpen] = useState(false);
  const [selectedMiningMaterial, setSelectedMiningMaterial] =
    useState<MiningSelection>();
  const [repairKitInventory, setRepairKitInventory] = useState(
    createRepairKitInventory,
  );
  const [fuelCellInventory, setFuelCellInventory] = useState(
    createFuelCellInventory,
  );
  const [repairDialogTarget, setRepairDialogTarget] =
    useState<RepairDialogTarget>();
  const [selectedRepairKitTier, setSelectedRepairKitTier] =
    useState<RepairKitTier>('t1');
  const [repairKitCountField, setRepairKitCountField] = useState('1');
  const [isRefuelDialogOpen, setIsRefuelDialogOpen] = useState(false);
  const [selectedFuelCellTier, setSelectedFuelCellTier] =
    useState<FuelCellTier>('t1');
  const [fuelCellCountField, setFuelCellCountField] = useState('1');
  const [modulePanelSelection, setModulePanelSelection] =
    useState<ModulePanelSelection>({ type: 'module', id: 'mining-module-1' });
  const [expandedSpeedControls, setExpandedSpeedControls] = useState(
    () => new Set<SpeedControlTab>(),
  );
  const [saveStatus, setSaveStatus] = useState<
    { type: 'success' | 'error'; message: string } | undefined
  >();
  const [isSaving, setIsSaving] = useState(false);
  const thrusterPadRef = useRef<HTMLDivElement>(null);
  const saveStatusTimerRef = useRef<number | undefined>(undefined);
  const selectedModule =
    modulePanelSelection.type === 'module'
      ? modules.find((module) => module.id === modulePanelSelection.id)
      : undefined;
  const miningModule = modules.find(
    (module) => module.type === 'mining' && module.unlocked,
  );
  const fabricatorModule = modules.find(
    (module) => module.type === 'fabricator' && module.unlocked,
  );
  const energyCoreModule = modules.find(
    (module) => module.type === 'energy-core' && module.unlocked,
  );
  const miningModuleActive = Boolean(
    miningModule?.active && miningModule.durability > 0,
  );
  const maxHullDurability = getSpaceshipMaxHullDurability(hullLevel);
  const inventoryMassKg = INVENTORY_MATERIALS.reduce(
    (total, material) => total + inventory[material],
    0,
  );
  const activeTargetSpeed =
    activeFeature?.type === 'target-speed' ? activeFeature : undefined;
  const activeThrusters =
    activeFeature?.type === 'thrusters' ||
    activeFeature?.type === 'manual-force'
      ? activeFeature
      : undefined;
  const canControlManualThrusters =
    motionState !== 'crashed' && !activeTargetSpeed;
  const currentEnginePowerPercent =
    activeThrusterSignals.reduce(
      (totalPowerPercent, thruster) =>
        totalPowerPercent + (thruster.active ? thruster.powerPercent : 0),
      0,
    ) / SPACESHIP_THRUSTER_COUNT;
  const displayedThrusters = activeFeature
    ? activeThrusterSignals
    : manualThrusters.map((thruster) => ({
        powerPercent: Number(thruster.powerPercent),
        active: thruster.active,
      }));
  const displayedHorizontalThrusterValue = getThrusterAxisValue(
    displayedThrusters,
    3,
    1,
  );
  const displayedVerticalThrusterValue = getThrusterAxisValue(
    displayedThrusters,
    2,
    0,
  );
  const thrusterPadStyle = {
    '--thruster-x': `${50 + displayedHorizontalThrusterValue / 2}%`,
    '--thruster-y': `${50 - displayedVerticalThrusterValue / 2}%`,
  } as CSSProperties;
  const thrusterReadouts = THRUSTER_LABELS.map((label, index) => {
    const thruster = displayedThrusters[index];
    const forceN =
      thruster?.active && thruster.powerPercent > 0
        ? MAX_ENGINE_THRUST_KN * 1_000 * (thruster.powerPercent / 100)
        : 0;

    return {
      label,
      forceN,
      durability: thrusterDurability[index] ?? 0,
    };
  });
  const predictionSeconds =
    Number(predictionAmount) *
    ({ s: 1, m: 60, h: 3_600 } as const)[predictionUnit];
  const hasValidPrediction =
    Number.isFinite(predictionSeconds) && predictionSeconds > 0;

  useEffect(() => {
    const syncTimer = window.setTimeout(() => {
      setManualThrusterAxisFields({
        horizontal: String(displayedHorizontalThrusterValue),
        vertical: String(displayedVerticalThrusterValue),
      });
    }, 0);

    return () => window.clearTimeout(syncTimer);
  }, [displayedHorizontalThrusterValue, displayedVerticalThrusterValue]);

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

  useEffect(
    () => () => {
      window.clearTimeout(saveStatusTimerRef.current);
    },
    [],
  );

  useEffect(() => {
    const blurThrusterPadOnOutsidePointerDown = (event: PointerEvent) => {
      const thrusterPad = thrusterPadRef.current;
      if (
        !thrusterPad ||
        document.activeElement !== thrusterPad ||
        !(event.target instanceof Node) ||
        thrusterPad.contains(event.target)
      ) {
        return;
      }

      thrusterPad.blur();
    };

    document.addEventListener(
      'pointerdown',
      blurThrusterPadOnOutsidePointerDown,
      true,
    );

    return () => {
      document.removeEventListener(
        'pointerdown',
        blurThrusterPadOnOutsidePointerDown,
        true,
      );
    };
  }, []);

  const showSaveStatus = (
    status: { type: 'success' | 'error'; message: string },
    durationMs = status.type === 'success' ? 2_500 : 4_000,
  ) => {
    window.clearTimeout(saveStatusTimerRef.current);
    setSaveStatus(status);
    saveStatusTimerRef.current = window.setTimeout(() => {
      setSaveStatus(undefined);
      saveStatusTimerRef.current = undefined;
    }, durationMs);
  };

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

  const toggleMeasuring = () => {
    const measuringPanelIsOpen = expandedSpeedControls.has('measuring');
    if (!isMeasuring || measuringPanelIsOpen) onToggleMeasuring?.();

    setExpandedSpeedControls((expandedControls) => {
      const nextControls = new Set(expandedControls);
      if (!isMeasuring) nextControls.add('measuring');
      else if (measuringPanelIsOpen) {
        nextControls.delete('measuring');
      } else {
        nextControls.add('measuring');
      }
      return nextControls;
    });
  };

  const handleSaveSpaceship = () => {
    if (isSaving) return;

    const blockReason = getSpaceshipSaveBlockReason();
    if (blockReason) {
      showSaveStatus({
        type: 'error',
        message: blockReason,
      });
      return;
    }

    setIsSaving(true);
    void saveSpaceship()
      .then(() => {
        showSaveStatus({ type: 'success', message: 'Spaceship saved.' });
      })
      .catch((error: unknown) => {
        showSaveStatus({
          type: 'error',
          message:
            error instanceof Error && error.message
              ? error.message
              : 'Failed to save spaceship.',
        });
      })
      .finally(() => setIsSaving(false));
  };

  const getThrustersSchedule = (thrusters: ManualThrusterInput[]) =>
    thrusters.map((thruster) => ({
      powerPercent: Number(thruster.powerPercent),
      active: thruster.active,
    }));

  const applyManualThrusters = (thrusters: ManualThrusterInput[]) => {
    if (!canControlManualThrusters) return;

    const schedule = getThrustersSchedule(thrusters);
    const fieldsAreValid = schedule.every(
      (thruster) =>
        Number.isFinite(thruster.powerPercent) &&
        thruster.powerPercent >= 0 &&
        thruster.powerPercent <= 100,
    );
    const hasActiveThruster =
      fieldsAreValid &&
      fuelKns > 0 &&
      schedule.some(
        (thruster, index) =>
          thruster.active &&
          thruster.powerPercent > 0 &&
          (thrusterDurability[index] ?? 0) > 0,
      );

    if (hasActiveThruster) {
      onStartThrusters?.(schedule);
    } else if (activeThrusters) {
      onStopEngines?.();
    }
  };

  const updateManualThrusterVector = (horizontal: number, vertical: number) => {
    const nextThrusters = setManualThrusterAxisValue(
      setManualThrusterAxisValue(
        createManualThrusterInputsFromSignals(displayedThrusters),
        'horizontal',
        horizontal,
      ),
      'vertical',
      vertical,
    );
    setManualThrusters(nextThrusters);
    applyManualThrusters(nextThrusters);
  };

  const updateManualThrusterAxis = (
    axis: 'horizontal' | 'vertical',
    value: string,
  ) => {
    setManualThrusterAxisFields((fields) => ({ ...fields, [axis]: value }));

    const axisValue = Number(value);
    if (!Number.isFinite(axisValue)) return;

    updateManualThrusterVector(
      axis === 'horizontal' ? axisValue : displayedHorizontalThrusterValue,
      axis === 'vertical' ? axisValue : displayedVerticalThrusterValue,
    );
  };

  const commitManualThrusterAxis = (axis: 'horizontal' | 'vertical') => {
    const axisValue = Number(manualThrusterAxisFields[axis]);
    const clampedValue = clampThrusterAxisValue(axisValue);

    setManualThrusterAxisFields((fields) => ({
      ...fields,
      [axis]: String(clampedValue),
    }));

    updateManualThrusterVector(
      axis === 'horizontal' ? clampedValue : displayedHorizontalThrusterValue,
      axis === 'vertical' ? clampedValue : displayedVerticalThrusterValue,
    );
  };

  const updateManualThrusterVectorFromPoint = (
    event: ReactPointerEvent<HTMLDivElement>,
  ) => {
    if (!canControlManualThrusters) return;

    const rect = event.currentTarget.getBoundingClientRect();
    const horizontal = clampThrusterAxisValue(
      ((event.clientX - rect.left) / rect.width - 0.5) * 200,
    );
    const vertical = clampThrusterAxisValue(
      (0.5 - (event.clientY - rect.top) / rect.height) * 200,
    );

    updateManualThrusterVector(horizontal, vertical);
  };

  const handleThrusterPadPointerDown = (
    event: ReactPointerEvent<HTMLDivElement>,
  ) => {
    event.currentTarget.focus({ preventScroll: true });
    event.currentTarget.setPointerCapture(event.pointerId);
    updateManualThrusterVectorFromPoint(event);
  };

  const handleThrusterPadPointerMove = (
    event: ReactPointerEvent<HTMLDivElement>,
  ) => {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;

    updateManualThrusterVectorFromPoint(event);
  };

  const handleThrusterPadKeyDown = (
    event: ReactKeyboardEvent<HTMLDivElement>,
  ) => {
    if (!canControlManualThrusters) return;

    const offsets: Partial<
      Record<string, { horizontal: number; vertical: number }>
    > = {
      ArrowUp: { horizontal: 0, vertical: 1 },
      ArrowRight: { horizontal: 1, vertical: 0 },
      ArrowDown: { horizontal: 0, vertical: -1 },
      ArrowLeft: { horizontal: -1, vertical: 0 },
    };
    const offset = offsets[event.key];
    if (!offset) return;

    event.preventDefault();
    updateManualThrusterVector(
      displayedHorizontalThrusterValue + offset.horizontal,
      displayedVerticalThrusterValue + offset.vertical,
    );
  };

  const turnOffAllThrusters = () => {
    setManualThrusters(createManualThrusterInputs());
    onStopEngines?.();
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

  const repairSelectedModule = (module: ShipModule) => {
    setRepairDialogTarget({ type: 'module', id: module.id });
    setRepairKitCountField('1');
  };

  const repairHull = () => {
    setRepairDialogTarget({ type: 'hull' });
    setRepairKitCountField('1');
  };

  const improveHull = () => {
    const nextLevel = hullLevel + 1;
    const cost: Partial<Inventory> = {
      iron: 40 * nextLevel,
      silicates: 18 * nextLevel,
      carbon: 8 * nextLevel,
    };
    if (!spendInventory(cost)) return;

    upgradeSpaceshipHull();
  };

  const repairThruster = (index: number, durability: number) => {
    if (durability >= MAX_THRUSTER_DURABILITY) return;

    setRepairDialogTarget({ type: 'thruster', index });
    setRepairKitCountField('1');
  };

  const selectMiningMaterial = (selection: MiningSelection) => {
    setSelectedMiningMaterial(selection);
    onMiningSelectionChange?.(selection);
  };

  const toggleMiningModule = () => {
    if (!miningModule || miningModule.durability <= 0) return;

    if (miningModuleActive) {
      setModuleActive(miningModule.id, false);
      return;
    }

    if (
      !selectedMiningMaterial ||
      inventoryMassKg >= SPACESHIP_INVENTORY_CAPACITY_KG
    ) {
      return;
    }

    onMiningSelectionChange?.(selectedMiningMaterial);
    setModuleActive(miningModule.id, true);
  };

  const canUseFabricator = Boolean(
    fabricatorModule &&
    fabricatorModule.durability >= FABRICATOR_DURABILITY_DRAIN_PER_CRAFT,
  );
  const repairDialogTargetDetails =
    repairDialogTarget?.type === 'module'
      ? (() => {
          const module = modules.find(
            (candidate) => candidate.id === repairDialogTarget.id,
          );
          if (!module) return undefined;
          return {
            label: module.name,
            durability: module.durability,
            maxDurability: getModuleMaxDurability(module),
          };
        })()
      : repairDialogTarget?.type === 'hull'
        ? {
            label: 'Hull',
            durability: hullDurability,
            maxDurability: maxHullDurability,
          }
        : repairDialogTarget?.type === 'thruster'
          ? {
              label: `Thruster ${repairDialogTarget.index + 1}`,
              durability: thrusterDurability[repairDialogTarget.index] ?? 0,
              maxDurability: MAX_THRUSTER_DURABILITY,
            }
          : undefined;
  const selectedRepairKit =
    REPAIR_KIT_TIERS.find((kit) => kit.tier === selectedRepairKitTier) ??
    REPAIR_KIT_TIERS[0];
  const repairUnitsNeeded = repairDialogTargetDetails
    ? Math.max(
        0,
        repairDialogTargetDetails.maxDurability -
          repairDialogTargetDetails.durability,
      )
    : 0;
  const selectedRepairKitCount = Math.max(
    0,
    Math.floor(Number(repairKitCountField)),
  );
  const selectedRepairKitAvailable = repairKitInventory[selectedRepairKit.tier];
  const selectedRepairKitUsefulCount =
    repairUnitsNeeded > 0
      ? Math.ceil(repairUnitsNeeded / selectedRepairKit.repairAmount)
      : 0;
  const selectedRepairKitAppliedCount = Math.min(
    selectedRepairKitCount,
    selectedRepairKitAvailable,
    selectedRepairKitUsefulCount,
  );
  const selectedRepairAmount =
    selectedRepairKitAppliedCount * selectedRepairKit.repairAmount;
  const repairDialogCanApply =
    repairDialogTarget !== undefined &&
    repairUnitsNeeded > 0 &&
    selectedRepairKitAppliedCount > 0;
  const selectedFuelCell =
    FUEL_CELL_TIERS.find((cell) => cell.tier === selectedFuelCellTier) ??
    FUEL_CELL_TIERS[0];
  const fuelUnitsNeeded = Math.max(0, INITIAL_SPACESHIP_FUEL_KNS - fuelKns);
  const selectedFuelCellCount = Math.max(
    0,
    Math.floor(Number(fuelCellCountField)),
  );
  const selectedFuelCellAvailable = fuelCellInventory[selectedFuelCell.tier];
  const selectedFuelCellUsefulCount =
    fuelUnitsNeeded > 0
      ? Math.ceil(fuelUnitsNeeded / selectedFuelCell.fuelKns)
      : 0;
  const selectedFuelCellAppliedCount = Math.min(
    selectedFuelCellCount,
    selectedFuelCellAvailable,
    selectedFuelCellUsefulCount,
  );
  const selectedRefuelAmount =
    selectedFuelCellAppliedCount * selectedFuelCell.fuelKns;
  const appliedRefuelAmount = Math.min(selectedRefuelAmount, fuelUnitsNeeded);
  const refuelDialogCanApply =
    isRefuelDialogOpen &&
    Boolean(energyCoreModule) &&
    fuelUnitsNeeded > 0 &&
    selectedFuelCellAppliedCount > 0 &&
    (energyCoreModule?.durability ?? 0) >=
      ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL * selectedFuelCellAppliedCount;

  const fabricateBlueprint = (
    blueprintId: (typeof FABRICATOR_BLUEPRINTS)[number]['id'],
  ) => {
    const blueprint = FABRICATOR_BLUEPRINTS.find(
      (candidate) => candidate.id === blueprintId,
    );
    if (!blueprint) return;
    if (!fabricatorModule || !canUseFabricator) return;
    if (!spendInventory(blueprint.cost)) return;
    if (!consumeFabricatorDurability(fabricatorModule.id)) return;

    if (blueprint.output.type === 'fuel-cell') {
      const fuelCellOutput = blueprint.output;
      setFuelCellInventory((current) => {
        const tier = fuelCellOutput.tier;
        return {
          ...current,
          [tier]: current[tier] + fuelCellOutput.quantity,
        };
      });
      return;
    }

    const repairKitOutput = blueprint.output;
    setRepairKitInventory((current) => {
      const tier = repairKitOutput.tier;
      return {
        ...current,
        [tier]: current[tier] + repairKitOutput.quantity,
      };
    });
  };

  const applyRepairKits = () => {
    if (!repairDialogTarget || !repairDialogCanApply) return;

    const repaired =
      repairDialogTarget.type === 'module'
        ? repairModuleByAmount(repairDialogTarget.id, selectedRepairAmount)
        : repairDialogTarget.type === 'hull'
          ? repairSpaceshipHullByAmount(selectedRepairAmount)
          : repairSpaceshipThrusterByAmount(
              repairDialogTarget.index,
              selectedRepairAmount,
            );
    if (!repaired) return;

    setRepairKitInventory((current) => ({
      ...current,
      [selectedRepairKit.tier]:
        current[selectedRepairKit.tier] - selectedRepairKitAppliedCount,
    }));
    setRepairDialogTarget(undefined);
  };

  const applyFuelCells = () => {
    if (!energyCoreModule || !refuelDialogCanApply) return;
    if (
      !consumeEnergyCoreDurability(
        energyCoreModule.id,
        selectedFuelCellAppliedCount,
      )
    ) {
      return;
    }

    setFuelCellInventory((current) => ({
      ...current,
      [selectedFuelCell.tier]:
        current[selectedFuelCell.tier] - selectedFuelCellAppliedCount,
    }));
    addSpaceshipFuelKns(appliedRefuelAmount);
    setIsRefuelDialogOpen(false);
  };

  const featureButtons: ReactNode[] = [
    <button
      key="communications"
      className={style.featureButton}
      type="button"
      aria-label={
        unreadMessageCount > 0
          ? `Communications, ${unreadMessageCount} unread messages`
          : 'Communications'
      }
      onClick={onOpenCommunications}
    >
      <span>Communications</span>
      {unreadMessageCount > 0 && (
        <strong className={style.unreadBadge} aria-hidden="true">
          {unreadMessageCount > 99 ? '99+' : unreadMessageCount}
        </strong>
      )}
    </button>,
    <button
      key="search"
      className={style.featureButton}
      type="button"
      onClick={onOpenSearch}
    >
      Search
    </button>,
    ...(
      [
        ['thrusters', 'Thrusters'],
        ['modules', 'Modules'],
        ['research', 'Research'],
        ['fabricator', 'Fabricator'],
        ['prediction', 'Prediction'],
      ] as const
    ).map(([tab, label]) => (
      <button
        id={`footer-${tab}-tab`}
        className={style.featureButton}
        type="button"
        aria-controls={`footer-${tab}-panel`}
        aria-expanded={expandedSpeedControls.has(tab)}
        data-active={expandedSpeedControls.has(tab)}
        key={tab}
        disabled={tab === 'fabricator' && !fabricatorModule}
        onClick={() => toggleSpeedControl(tab)}
      >
        {label}
      </button>
    )),
    <button
      key="measuring"
      className={style.featureButton}
      type="button"
      aria-pressed={isMeasuring}
      aria-controls="footer-measuring-panel"
      aria-expanded={expandedSpeedControls.has('measuring')}
      data-active={isMeasuring}
      onClick={toggleMeasuring}
    >
      Measuring
    </button>,
    <button
      key="ruler"
      className={style.featureButton}
      type="button"
      aria-pressed={isRulerActive}
      data-active={isRulerActive}
      onClick={onToggleRuler}
    >
      Ruler
    </button>,
    <button
      key="save"
      className={style.featureButton}
      type="button"
      data-loading={isSaving}
      disabled={isSaving}
      onClick={handleSaveSpaceship}
    >
      {isSaving ? (
        <span className={style.loadingSpinner} aria-hidden="true" />
      ) : (
        'Save'
      )}
    </button>,
    <button
      key="mining"
      className={style.featureButton}
      type="button"
      data-active={isMiningPanelOpen}
      disabled={!miningModule}
      onClick={() => setIsMiningPanelOpen((open) => !open)}
    >
      Mining
    </button>,
    <button
      key="inventory"
      className={style.featureButton}
      type="button"
      data-active={isInventoryPanelOpen}
      onClick={() => setIsInventoryPanelOpen((open) => !open)}
    >
      Inventory
    </button>,
  ];
  const featureSlots = Array.from({ length: 16 }, (_, index) => ({
    id: `feature-slot-${index + 1}`,
    button: featureButtons[index],
  }));

  return (
    <footer
      className={style.container}
      data-repair-dialog-open={
        repairDialogTargetDetails || isRefuelDialogOpen ? 'true' : undefined
      }
      aria-label="Ship controls"
    >
      <div className={style.communicationsDock}>
        {unreadMessages.length > 0 && (
          <ol className={style.messageNotifications} aria-live="polite">
            {unreadMessages.slice(-3).map((message) => (
              <li key={message.id}>
                <button
                  type="button"
                  onClick={() => onOpenCommunicationThread?.(message.contactId)}
                >
                  <strong>{message.senderName}</strong>
                  <span>{getMessagePreview(message.text)}</span>
                </button>
              </li>
            ))}
          </ol>
        )}
      </div>
      <section className={style.speedControls} aria-label="Feature row">
        <h2 className={style.featureRowTitle}>Feature row</h2>
        <div className={style.featureSlots}>
          {featureSlots.map((slot) => (
            <div className={style.featureSlot} key={slot.id}>
              {slot.button}
            </div>
          ))}
        </div>
        {saveStatus && (
          <output
            className={style.saveToast}
            data-status={saveStatus.type}
            aria-live="polite"
          >
            {saveStatus.message}
          </output>
        )}

        <div className={style.controlPanels}>
          {expandedSpeedControls.has('modules') && (
            <DraggablePanel
              control="modules"
              onClose={() => toggleSpeedControl('modules')}
            >
              <div className={style.modulesPanel}>
                <div className={style.moduleList}>
                  <section>
                    <h3>Current modules</h3>
                    {modules.map((module) => {
                      const maximumDurability = getModuleMaxDurability(module);
                      return (
                        <button
                          key={module.id}
                          type="button"
                          data-selected={
                            modulePanelSelection.type === 'module' &&
                            modulePanelSelection.id === module.id
                          }
                          onClick={() =>
                            setModulePanelSelection({
                              type: 'module',
                              id: module.id,
                            })
                          }
                        >
                          <span>{module.name}</span>
                          <small>
                            {MODULE_LABELS[module.type]} ·{' '}
                            {Math.round(
                              (module.durability / maximumDurability) * 100,
                            )}
                            %
                          </small>
                        </button>
                      );
                    })}
                  </section>
                  <section>
                    <h3>Ship systems</h3>
                    <button
                      type="button"
                      data-selected={modulePanelSelection.type === 'hull'}
                      onClick={() => setModulePanelSelection({ type: 'hull' })}
                    >
                      <span>Hull</span>
                      <small>
                        {Math.round((hullDurability / maxHullDurability) * 100)}
                        %
                      </small>
                    </button>
                    {thrusterDurability.map((durability, index) => (
                      <button
                        key={index}
                        type="button"
                        data-selected={
                          modulePanelSelection.type === 'thruster' &&
                          modulePanelSelection.index === index
                        }
                        onClick={() =>
                          setModulePanelSelection({ type: 'thruster', index })
                        }
                      >
                        <span>Thruster {index + 1}</span>
                        <small>
                          {Math.round(
                            (durability / MAX_THRUSTER_DURABILITY) * 100,
                          )}
                          %
                        </small>
                      </button>
                    ))}
                  </section>
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
                          (attribute) => {
                            const cost = getUpgradeCost(
                              selectedModule,
                              attribute,
                            );
                            return (
                              <div key={attribute}>
                                <dt>
                                  {ATTRIBUTE_LABELS[attribute]} L
                                  {selectedModule.levels[attribute] ?? 1}
                                </dt>
                                <dd>
                                  <span>
                                    {getModuleAttributeValue(
                                      selectedModule,
                                      attribute,
                                    )}
                                  </span>
                                  <button
                                    type="button"
                                    disabled={!canAfford(inventory, cost)}
                                    onClick={() =>
                                      upgradeResearchAttribute(
                                        selectedModule,
                                        attribute,
                                      )
                                    }
                                  >
                                    Improve
                                  </button>
                                  <small>{formatCost(cost)}</small>
                                </dd>
                              </div>
                            );
                          },
                        )}
                        <div>
                          <dt>Durability left</dt>
                          <dd>
                            <span>
                              {selectedModule.durability.toFixed(0)} /{' '}
                              {getModuleMaxDurability(selectedModule).toFixed(
                                0,
                              )}
                            </span>
                          </dd>
                        </div>
                      </dl>
                      <button
                        type="button"
                        disabled={
                          selectedModule.durability >=
                          getModuleMaxDurability(selectedModule)
                        }
                        onClick={() => repairSelectedModule(selectedModule)}
                      >
                        Repair
                      </button>
                      {selectedModule.type === 'energy-core' && (
                        <button
                          type="button"
                          disabled={
                            fuelCellInventory.t1 <= 0 ||
                            selectedModule.durability <
                              ENERGY_CORE_DURABILITY_DRAIN_PER_REFUEL
                          }
                          onClick={() => {
                            setFuelCellCountField('1');
                            setIsRefuelDialogOpen(true);
                          }}
                        >
                          Refuel
                        </button>
                      )}
                    </>
                  ) : modulePanelSelection.type === 'hull' ? (
                    <>
                      <header>
                        <span>Hull</span>
                        <small>Ship system L{hullLevel}</small>
                      </header>
                      <dl>
                        <div>
                          <dt>Drain rate</dt>
                          <dd>
                            <span>
                              {HULL_DURABILITY_DRAIN_PER_CRASH} durability /
                              crash
                            </span>
                          </dd>
                        </div>
                        <div>
                          <dt>Improve gain</dt>
                          <dd>
                            <span>
                              +{HULL_DURABILITY_PER_LEVEL} max durability
                            </span>
                          </dd>
                        </div>
                      </dl>
                      <div className={style.durabilityItem}>
                        <span>Integrity</span>
                        <meter
                          min="0"
                          max={maxHullDurability}
                          low={maxHullDurability * 0.25}
                          value={hullDurability}
                        />
                        <output>
                          {hullDurability.toFixed(2)} / {maxHullDurability}
                        </output>
                      </div>
                      <button
                        type="button"
                        disabled={
                          !canAfford(inventory, {
                            iron: 40 * (hullLevel + 1),
                            silicates: 18 * (hullLevel + 1),
                            carbon: 8 * (hullLevel + 1),
                          })
                        }
                        onClick={improveHull}
                      >
                        Improve{' '}
                        {formatCost({
                          iron: 40 * (hullLevel + 1),
                          silicates: 18 * (hullLevel + 1),
                          carbon: 8 * (hullLevel + 1),
                        })}
                      </button>
                      <button
                        type="button"
                        disabled={hullDurability >= maxHullDurability}
                        onClick={repairHull}
                      >
                        Repair
                      </button>
                    </>
                  ) : modulePanelSelection.type === 'thruster' ? (
                    <>
                      <header>
                        <span>Thruster {modulePanelSelection.index + 1}</span>
                        <small>Ship system</small>
                      </header>
                      <div className={style.durabilityItem}>
                        <span>Integrity</span>
                        <meter
                          min="0"
                          max={MAX_THRUSTER_DURABILITY}
                          low={MAX_THRUSTER_DURABILITY * 0.25}
                          value={
                            thrusterDurability[modulePanelSelection.index] ?? 0
                          }
                        />
                        <output>
                          {(
                            thrusterDurability[modulePanelSelection.index] ?? 0
                          ).toFixed(2)}{' '}
                          / {MAX_THRUSTER_DURABILITY}
                        </output>
                      </div>
                      <button
                        type="button"
                        disabled={
                          (thrusterDurability[modulePanelSelection.index] ??
                            0) >= MAX_THRUSTER_DURABILITY
                        }
                        onClick={() =>
                          repairThruster(
                            modulePanelSelection.index,
                            thrusterDurability[modulePanelSelection.index] ?? 0,
                          )
                        }
                      >
                        Repair
                      </button>
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
                        <button
                          type="button"
                          onClick={() => {
                            setModulePanelSelection({
                              type: 'module',
                              id: module.id,
                            });
                            setExpandedSpeedControls((expandedControls) => {
                              const nextControls = new Set(expandedControls);
                              nextControls.add('modules');
                              return nextControls;
                            });
                          }}
                        >
                          Inspect
                        </button>
                      )}
                    </section>
                  );
                })}
              </div>
            </DraggablePanel>
          )}
          {expandedSpeedControls.has('fabricator') && (
            <DraggablePanel
              control="fabricator"
              onClose={() => toggleSpeedControl('fabricator')}
            >
              <div className={style.fabricatorPanel}>
                <section className={style.fabricatorStatus}>
                  <span>Fabricator durability</span>
                  <meter
                    min={0}
                    max={
                      fabricatorModule
                        ? getModuleMaxDurability(fabricatorModule)
                        : FABRICATOR_DURABILITY_DRAIN_PER_CRAFT
                    }
                    value={fabricatorModule?.durability ?? 0}
                  />
                  <output>
                    {(fabricatorModule?.durability ?? 0).toFixed(0)} /{' '}
                    {fabricatorModule
                      ? getModuleMaxDurability(fabricatorModule).toFixed(0)
                      : 0}
                  </output>
                </section>

                <section className={style.fabricatorBlueprints}>
                  <h3>Blueprints</h3>
                  {FABRICATOR_BLUEPRINTS.map((blueprint) => (
                    <article
                      className={style.fabricatorBlueprint}
                      key={blueprint.id}
                    >
                      <header>
                        <span>{blueprint.name}</span>
                        <small>
                          {blueprint.output.type === 'repair-kit'
                            ? `${repairKitInventory[blueprint.output.tier]} stored`
                            : `${fuelCellInventory[blueprint.output.tier]} stored`}
                        </small>
                      </header>
                      <dl>
                        <div>
                          <dt>Cost</dt>
                          <dd>{formatCost(blueprint.cost)}</dd>
                        </div>
                        <div>
                          <dt>Output</dt>
                          <dd>
                            {blueprint.output.type === 'fuel-cell'
                              ? `${blueprint.output.quantity} cell / ${formatForce(
                                  blueprint.output.fuelKns * 1_000,
                                )}`
                              : `${blueprint.output.quantity} kit / ${blueprint.output.repairAmount} durability`}
                          </dd>
                        </div>
                      </dl>
                      <button
                        type="button"
                        disabled={
                          !canUseFabricator ||
                          !canAfford(inventory, blueprint.cost)
                        }
                        onClick={() => fabricateBlueprint(blueprint.id)}
                      >
                        Fabricate
                      </button>
                    </article>
                  ))}
                </section>
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
          {expandedSpeedControls.has('measuring') && (
            <DraggablePanel
              control="measuring"
              onClose={() => toggleSpeedControl('measuring')}
            >
              <div className={style.measuringControls}>
                <label className={style.switchControl}>
                  <span>Relative to spaceship</span>
                  <input
                    type="checkbox"
                    role="switch"
                    checked={isMeasurementRelativeToSpaceship}
                    onChange={(event) =>
                      onMeasurementRelativeToSpaceshipChange?.(
                        event.currentTarget.checked,
                      )
                    }
                  />
                </label>
                <label className={style.switchControl}>
                  <span>Separate velocity axes</span>
                  <input
                    type="checkbox"
                    role="switch"
                    checked={isMeasurementVelocityAxesSeparated}
                    onChange={(event) =>
                      onMeasurementVelocityAxesSeparatedChange?.(
                        event.currentTarget.checked,
                      )
                    }
                  />
                </label>
              </div>
            </DraggablePanel>
          )}
          {expandedSpeedControls.has('thrusters') && (
            <DraggablePanel
              control="thrusters"
              onClose={() => toggleSpeedControl('thrusters')}
            >
              <div className={style.thrustersPanel}>
                <div
                  ref={thrusterPadRef}
                  className={style.thrustersPad}
                  style={thrusterPadStyle}
                  role="application"
                  tabIndex={canControlManualThrusters ? 0 : -1}
                  aria-label="Thruster vector pad"
                  aria-disabled={!canControlManualThrusters}
                  onKeyDown={handleThrusterPadKeyDown}
                  onPointerDown={handleThrusterPadPointerDown}
                  onPointerMove={handleThrusterPadPointerMove}
                >
                  <span className={style.thrustersPadAxis} />
                  <span className={style.thrustersPadAxis} />
                  <span className={style.thrustersPadRing} />
                  <span className={style.thrustersPadDot} />
                </div>
                <dl className={style.thrustersPadReadout}>
                  <div>
                    <dt>Horizontal</dt>
                    <dd>
                      {formatSignedThrusterAxisValue(
                        displayedHorizontalThrusterValue,
                      )}
                    </dd>
                  </div>
                  <div>
                    <dt>Vertical</dt>
                    <dd>
                      {formatSignedThrusterAxisValue(
                        displayedVerticalThrusterValue,
                      )}
                    </dd>
                  </div>
                </dl>
                <div className={style.thrustersDetails}>
                  <dl className={style.thrustersStatusList}>
                    {thrusterReadouts.map((thruster) => (
                      <div key={thruster.label}>
                        <dt>{thruster.label}</dt>
                        <dd>
                          <span>{formatForce(thruster.forceN)}</span>
                          <small>{formatPercentage(thruster.durability)}</small>
                        </dd>
                      </div>
                    ))}
                  </dl>
                  <div className={style.thrustersActions}>
                    <div className={style.thrustersVectorInputs}>
                      <label htmlFor="footer-thruster-horizontal">
                        <span>Horizontal</span>
                        <input
                          id="footer-thruster-horizontal"
                          type="number"
                          min="-100"
                          max="100"
                          step="1"
                          value={manualThrusterAxisFields.horizontal}
                          disabled={!canControlManualThrusters}
                          onChange={(event) =>
                            updateManualThrusterAxis(
                              'horizontal',
                              event.currentTarget.value,
                            )
                          }
                          onBlur={() => commitManualThrusterAxis('horizontal')}
                        />
                      </label>
                      <label htmlFor="footer-thruster-vertical">
                        <span>Vertical</span>
                        <input
                          id="footer-thruster-vertical"
                          type="number"
                          min="-100"
                          max="100"
                          step="1"
                          value={manualThrusterAxisFields.vertical}
                          disabled={!canControlManualThrusters}
                          onChange={(event) =>
                            updateManualThrusterAxis(
                              'vertical',
                              event.currentTarget.value,
                            )
                          }
                          onBlur={() => commitManualThrusterAxis('vertical')}
                        />
                      </label>
                    </div>
                    <button
                      type="button"
                      data-running={activeFeature ? 'true' : undefined}
                      onClick={turnOffAllThrusters}
                    >
                      Turn off all thrusters
                    </button>
                  </div>
                </div>
              </div>
            </DraggablePanel>
          )}
        </div>
      </section>

      {isMiningPanelOpen && miningTelemetry && (
        <div className={style.controlPanels}>
          <DraggablePanel
            control="mining-status"
            onClose={() => setIsMiningPanelOpen(false)}
          >
            <MiningStatusPanel
              telemetry={miningTelemetry}
              active={miningModuleActive}
              selectedMiningMaterial={selectedMiningMaterial}
              inventoryMassKg={inventoryMassKg}
              inventoryCapacityKg={SPACESHIP_INVENTORY_CAPACITY_KG}
              onSelectMiningMaterial={selectMiningMaterial}
              onToggleMining={toggleMiningModule}
            />
          </DraggablePanel>
        </div>
      )}

      {isInventoryPanelOpen && (
        <div className={style.controlPanels}>
          <DraggablePanel
            control="inventory"
            onClose={() => setIsInventoryPanelOpen(false)}
          >
            <InventoryPanel
              inventory={inventory}
              inventoryMassKg={inventoryMassKg}
              inventoryCapacityKg={SPACESHIP_INVENTORY_CAPACITY_KG}
              fuelCellInventory={fuelCellInventory}
              repairKitInventory={repairKitInventory}
            />
          </DraggablePanel>
        </div>
      )}

      {isRefuelDialogOpen && energyCoreModule && (
        <div className={style.repairDialogBackdrop} role="presentation">
          <section
            className={style.repairDialog}
            role="dialog"
            aria-modal="true"
            aria-labelledby="refuel-dialog-title"
          >
            <header className={style.repairDialogHeader}>
              <div>
                <small>Energy Core refuel</small>
                <h2 id="refuel-dialog-title">Energy Core</h2>
              </div>
              <button
                type="button"
                aria-label="Close refuel dialog"
                onClick={() => setIsRefuelDialogOpen(false)}
              >
                ×
              </button>
            </header>

            <div className={style.repairDialogContent}>
              <section className={style.repairNeed}>
                <header>
                  <span>Fuel needed</span>
                  <strong>{formatImpulse(fuelUnitsNeeded * 1_000)}</strong>
                </header>
                <div
                  className={style.repairProgress}
                  style={
                    {
                      '--repair-current': `${Math.min(
                        100,
                        (fuelKns / INITIAL_SPACESHIP_FUEL_KNS) * 100,
                      )}%`,
                      '--repair-preview': `${Math.min(
                        100,
                        ((fuelKns + appliedRefuelAmount) /
                          INITIAL_SPACESHIP_FUEL_KNS) *
                          100,
                      )}%`,
                    } as CSSProperties
                  }
                  aria-label={`${formatImpulse(
                    fuelUnitsNeeded * 1_000,
                  )} fuel needed`}
                />
                <output>
                  {formatImpulse(fuelKns * 1_000)} /{' '}
                  {formatImpulse(INITIAL_SPACESHIP_FUEL_KNS * 1_000)}
                </output>
              </section>

              <section className={style.repairKitList}>
                <h3>Fuel cells</h3>
                {FUEL_CELL_TIERS.map((cell) => (
                  <button
                    key={cell.tier}
                    type="button"
                    data-selected={selectedFuelCellTier === cell.tier}
                    onClick={() => {
                      setSelectedFuelCellTier(cell.tier);
                      setFuelCellCountField('1');
                    }}
                  >
                    <span>{cell.label}</span>
                    <small>
                      {fuelCellInventory[cell.tier]} stored ·{' '}
                      {formatForce(cell.fuelKns * 1_000)} each
                    </small>
                  </button>
                ))}
              </section>

              <label className={style.repairCountField}>
                <span>Use cells</span>
                <input
                  type="number"
                  min="1"
                  max={Math.max(
                    1,
                    Math.min(
                      selectedFuelCellAvailable,
                      selectedFuelCellUsefulCount,
                    ),
                  )}
                  step="1"
                  value={fuelCellCountField}
                  onChange={(event) =>
                    setFuelCellCountField(event.currentTarget.value)
                  }
                />
              </label>
            </div>

            <footer className={style.repairDialogActions}>
              <span>
                Applying {selectedFuelCellAppliedCount} cell
                {selectedFuelCellAppliedCount === 1 ? '' : 's'} refuels up to{' '}
                {formatImpulse(appliedRefuelAmount * 1_000)}.
              </span>
              <button
                type="button"
                disabled={!refuelDialogCanApply}
                onClick={applyFuelCells}
              >
                Refuel
              </button>
            </footer>
          </section>
        </div>
      )}

      {repairDialogTargetDetails && (
        <div className={style.repairDialogBackdrop} role="presentation">
          <section
            className={style.repairDialog}
            role="dialog"
            aria-modal="true"
            aria-labelledby="repair-dialog-title"
          >
            <header className={style.repairDialogHeader}>
              <div>
                <small>Module repair</small>
                <h2 id="repair-dialog-title">
                  {repairDialogTargetDetails.label}
                </h2>
              </div>
              <button
                type="button"
                aria-label="Close repair dialog"
                onClick={() => setRepairDialogTarget(undefined)}
              >
                ×
              </button>
            </header>

            <div className={style.repairDialogContent}>
              <section className={style.repairNeed}>
                <header>
                  <span>Durability needed</span>
                  <strong>{repairUnitsNeeded.toFixed(0)} units</strong>
                </header>
                <div
                  className={style.repairProgress}
                  style={
                    {
                      '--repair-current': `${Math.min(
                        100,
                        (repairDialogTargetDetails.durability /
                          repairDialogTargetDetails.maxDurability) *
                          100,
                      )}%`,
                      '--repair-preview': `${Math.min(
                        100,
                        ((repairDialogTargetDetails.durability +
                          selectedRepairAmount) /
                          repairDialogTargetDetails.maxDurability) *
                          100,
                      )}%`,
                    } as CSSProperties
                  }
                  aria-label={`${repairUnitsNeeded.toFixed(
                    0,
                  )} durability units needed`}
                />
                <output>
                  {repairDialogTargetDetails.durability.toFixed(0)} /{' '}
                  {repairDialogTargetDetails.maxDurability.toFixed(0)}
                </output>
              </section>

              <section className={style.repairKitList}>
                <h3>Repair kits</h3>
                {REPAIR_KIT_TIERS.map((kit) => (
                  <button
                    key={kit.tier}
                    type="button"
                    data-selected={selectedRepairKitTier === kit.tier}
                    onClick={() => {
                      setSelectedRepairKitTier(kit.tier);
                      setRepairKitCountField('1');
                    }}
                  >
                    <span>{kit.label}</span>
                    <small>
                      {repairKitInventory[kit.tier]} stored · {kit.repairAmount}{' '}
                      units each
                    </small>
                  </button>
                ))}
              </section>

              <label className={style.repairCountField}>
                <span>Use kits</span>
                <input
                  type="number"
                  min="1"
                  max={Math.max(
                    1,
                    Math.min(
                      selectedRepairKitAvailable,
                      selectedRepairKitUsefulCount,
                    ),
                  )}
                  step="1"
                  value={repairKitCountField}
                  onChange={(event) =>
                    setRepairKitCountField(event.currentTarget.value)
                  }
                />
              </label>
            </div>

            <footer className={style.repairDialogActions}>
              <span>
                Applying {selectedRepairKitAppliedCount} kit
                {selectedRepairKitAppliedCount === 1 ? '' : 's'} repairs up to{' '}
                {Math.min(selectedRepairAmount, repairUnitsNeeded).toFixed(0)}{' '}
                units.
              </span>
              <button
                type="button"
                disabled={!repairDialogCanApply}
                onClick={applyRepairKits}
              >
                Repair
              </button>
            </footer>
          </section>
        </div>
      )}

      <div className={style.telemetryDock}>
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
      </div>
    </footer>
  );
}
