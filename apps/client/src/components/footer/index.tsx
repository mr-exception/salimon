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
  MINING_BASE_RATE_KG_PER_SECOND,
  MINING_BASE_DURABILITY_KG,
  MINING_BASE_RANGE_METERS,
  MINING_DURABILITY_PER_LEVEL_KG,
  MINING_RANGE_LEVEL_MULTIPLIER,
  INVENTORY_MATERIALS,
  MODULE_RESEARCH,
  SPACESHIP_THRUSTER_COUNT,
  THRUSTER_BASE_DURABILITY,
  THRUSTER_BASE_POWER_PERCENT,
  THRUSTER_LEVEL_MULTIPLIER,
  getModuleMaxDurability,
  getSpaceshipSaveBlockReason,
  repairModule,
  repairSpaceshipHull,
  repairSpaceshipThruster,
  setModuleActive,
  saveSpaceship,
  spendInventory,
  unlockModule,
  upgradeModuleAttribute,
  useInventory,
  useModules,
  useSpaceshipActiveFeature,
  useSpaceshipActiveThrusters,
  useSpaceshipAbsoluteSpeed,
  useSpaceshipFuelKns,
  useSpaceshipHullDurability,
  useSpaceshipMotionState,
  useSpaceshipThrusterDurability,
  type Inventory,
  type ModuleAttribute,
  type ModuleType,
  type ShipModule,
} from '@store';
import type { InventoryMaterial } from '@repo/types';
import {
  formatDuration,
  formatImpulse,
  formatPercentage,
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
  | 'research';

type ModulePanelSelection =
  | { type: 'module'; id: string }
  | { type: 'hull' }
  | { type: 'thruster'; index: number };

type Position = {
  x: number;
  y: number;
};

const CONTROL_LABELS: Record<SpeedControlTab, string> = {
  thrusters: 'Thrusters',
  measuring: 'Measuring',
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
  thrusters: { horizontal: 'left', vertical: 'top' },
  measuring: { horizontal: 'left', vertical: 'bottom' },
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
  if (feature === 'thrusters') {
    return (
      <svg viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 3v18M3 12h18" />
        <path d="m12 3-2.5 2.5M12 3l2.5 2.5M21 12l-2.5-2.5M21 12l-2.5 2.5M12 21l-2.5-2.5M12 21l2.5-2.5M3 12l2.5-2.5M3 12l2.5 2.5" />
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

function SaveIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M5 4h12l2 2v14H5z" />
      <path d="M8 4v6h8V4M8 20v-6h8v6" />
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

function getRepairCost(
  currentDurability: number,
  maximumDurability: number,
  scale = 1,
): Partial<Inventory> {
  if (maximumDurability <= 0 || currentDurability >= maximumDurability) {
    return { iron: 0, silicates: 0, ice: 0 };
  }

  const missingRatio =
    (maximumDurability - currentDurability) / maximumDurability;
  return {
    iron: Math.ceil(24 * missingRatio * scale),
    silicates: Math.ceil(12 * missingRatio * scale),
    ice: Math.ceil(4 * missingRatio * scale),
  };
}

function canAfford(inventory: Inventory, cost: Partial<Inventory>) {
  return INVENTORY_MATERIALS.every(
    (material) => inventory[material] >= (cost[material] ?? 0),
  );
}

function hasCost(cost: Partial<Inventory>) {
  return INVENTORY_MATERIALS.some((material) => (cost[material] ?? 0) > 0);
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

function SearchIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="10.5" cy="10.5" r="5.5" />
      <path d="m15 15 5 5" />
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

function RulerIcon() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true">
      <path d="M4 16 16 4l4 4L8 20 4 16Z" />
      <path d="m9 15-2-2m5-1-2-2m5-1-2-2" />
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
}: FooterProps) {
  const speed = useSpaceshipAbsoluteSpeed();
  const fuelKns = useSpaceshipFuelKns();
  const hullDurability = useSpaceshipHullDurability();
  const thrusterDurability = useSpaceshipThrusterDurability();
  const motionState = useSpaceshipMotionState();
  const activeFeature = useSpaceshipActiveFeature();
  const activeThrusterSignals = useSpaceshipActiveThrusters();
  const inventory = useInventory();
  const modules = useModules();
  const [manualThrusters, setManualThrusters] = useState(() =>
    Array.from({ length: SPACESHIP_THRUSTER_COUNT }, () => ({
      powerPercent: '100',
      active: false,
    })),
  );
  const [predictionAmount, setPredictionAmount] = useState('2');
  const [predictionUnit, setPredictionUnit] = useState<'s' | 'm' | 'h'>('m');
  const [isPredictionActive, setIsPredictionActive] = useState(false);
  const [modulePanelSelection, setModulePanelSelection] =
    useState<ModulePanelSelection>({ type: 'module', id: 'mining-module-1' });
  const [expandedSpeedControls, setExpandedSpeedControls] = useState(
    () => new Set<SpeedControlTab>(),
  );
  const [saveStatus, setSaveStatus] = useState<
    { type: 'success' | 'error'; message: string } | undefined
  >();
  const [isSaving, setIsSaving] = useState(false);
  const saveStatusTimerRef = useRef<number | undefined>(undefined);
  const selectedModule =
    modulePanelSelection.type === 'module'
      ? modules.find((module) => module.id === modulePanelSelection.id)
      : undefined;
  const miningModule = modules.find(
    (module) => module.type === 'mining' && module.unlocked,
  );
  const miningModuleActive = Boolean(
    miningModule?.active && miningModule.durability > 0,
  );
  const activeTargetSpeed =
    activeFeature?.type === 'target-speed' ? activeFeature : undefined;
  const activeThrusters =
    activeFeature?.type === 'thrusters' ||
    activeFeature?.type === 'manual-force'
      ? activeFeature
      : undefined;
  const thrustersSchedule = manualThrusters.map((thruster) => ({
    powerPercent: Number(thruster.powerPercent),
    active: thruster.active,
  }));
  const thrustersTimeSeconds = activeThrusters
    ? Math.max(0, activeThrusters.elapsedSeconds)
    : activeTargetSpeed
      ? Math.max(
          0,
          activeTargetSpeed.durationSeconds - activeTargetSpeed.elapsedSeconds,
        )
      : undefined;
  const thrustersFieldsAreValid = thrustersSchedule.every(
    (thruster) =>
      Number.isFinite(thruster.powerPercent) &&
      thruster.powerPercent >= 0 &&
      thruster.powerPercent <= 100,
  );
  const hasValidThrusters =
    thrustersFieldsAreValid &&
    thrustersSchedule.some(
      (thruster, index) =>
        thruster.active &&
        thruster.powerPercent > 0 &&
        (thrusterDurability[index] ?? 0) > 0,
    );
  const canControlManualThrusters =
    motionState !== 'crashed' && !activeTargetSpeed;
  const canStartThrusters =
    canControlManualThrusters && hasValidThrusters && fuelKns > 0;
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

  useEffect(
    () => () => {
      window.clearTimeout(saveStatusTimerRef.current);
    },
    [],
  );

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

  const updateManualThrusterPower = (index: number, value: string) => {
    setManualThrusters((thrusters) =>
      thrusters.map((thruster, thrusterIndex) =>
        thrusterIndex === index
          ? { ...thruster, powerPercent: value }
          : thruster,
      ),
    );

    if (activeThrusters) {
      applyManualThrusters(
        manualThrusters.map((thruster, thrusterIndex) =>
          thrusterIndex === index
            ? { ...thruster, powerPercent: value }
            : thruster,
        ),
      );
    }
  };

  const toggleManualThruster = (index: number) => {
    const nextThrusters = manualThrusters.map((thruster, thrusterIndex) =>
      thrusterIndex === index
        ? { ...thruster, active: !thruster.active }
        : thruster,
    );
    setManualThrusters(nextThrusters);
    applyManualThrusters(nextThrusters);
  };

  const startThrusters = () => {
    if (!canStartThrusters || activeThrusters || !onStartThrusters) return;

    onStartThrusters(thrustersSchedule);
  };

  const stopEngines = () => {
    onStopEngines?.();
    setManualThrusters((thrusters) =>
      thrusters.map((thruster) => ({ ...thruster, active: false })),
    );
  };

  const renderManualThrusterControl = (
    index: number,
    label: string,
    className: string,
  ) => {
    const thruster = manualThrusters[index];
    const displayedThruster = displayedThrusters[index];
    if (!thruster || !displayedThruster) return null;
    const displayedPower = activeFeature
      ? Number.isInteger(displayedThruster.powerPercent)
        ? String(displayedThruster.powerPercent)
        : displayedThruster.powerPercent.toFixed(2)
      : thruster.powerPercent;

    return (
      <div className={`${style.thrustersRow} ${className}`}>
        <span>{label}</span>
        <button
          className={style.thrusterToggle}
          type="button"
          aria-pressed={displayedThruster.active}
          disabled={!canControlManualThrusters}
          onClick={() => toggleManualThruster(index)}
        >
          {displayedThruster.active ? 'On' : 'Off'}
        </button>
        <label htmlFor={`footer-manual-thruster-${index}-power`}>
          <span>Power</span>
          <span className={style.speedField}>
            <input
              id={`footer-manual-thruster-${index}-power`}
              type="number"
              min="0"
              max="100"
              step="1"
              value={displayedPower}
              disabled={!canControlManualThrusters}
              onChange={(event) =>
                updateManualThrusterPower(index, event.currentTarget.value)
              }
            />
            <span aria-hidden="true">%</span>
          </span>
        </label>
      </div>
    );
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
    const cost = getRepairCost(
      module.durability,
      getModuleMaxDurability(module),
      module.type === 'mining' ? 1.2 : 1,
    );
    if (!hasCost(cost) || !spendInventory(cost)) return;

    repairModule(module.id);
  };

  const repairHull = () => {
    const cost = getRepairCost(hullDurability, MAX_HULL_DURABILITY, 2);
    if (!hasCost(cost) || !spendInventory(cost)) return;

    repairSpaceshipHull();
  };

  const repairThruster = (index: number, durability: number) => {
    const cost = getRepairCost(durability, MAX_THRUSTER_DURABILITY, 0.8);
    if (!hasCost(cost) || !spendInventory(cost)) return;

    repairSpaceshipThruster(index);
  };

  const toggleMiningModule = () => {
    if (!miningModule || miningModule.durability <= 0) return;

    setModuleActive(miningModule.id, !miningModule.active);
  };

  return (
    <footer className={style.container} aria-label="Ship controls">
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
        <button
          className={style.communicationButton}
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
      <section className={style.speedControls} aria-label="Ship features">
        <div className={style.controlTabs}>
          <button
            className={style.controlTab}
            type="button"
            aria-label="Search"
            onClick={onOpenSearch}
          >
            <SearchIcon />
            <span className={style.tooltip} role="tooltip">
              Search
            </span>
          </button>
          {(
            [
              ['thrusters', 'Thrusters'],
              ['modules', 'Modules'],
              ['research', 'Research'],
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
            aria-controls="footer-measuring-panel"
            aria-expanded={expandedSpeedControls.has('measuring')}
            data-active={isMeasuring}
            onClick={toggleMeasuring}
          >
            <MeasuringIcon />
            <span className={style.tooltip} role="tooltip">
              Measuring
            </span>
          </button>
          <button
            className={style.controlTab}
            type="button"
            aria-label="Ruler"
            aria-pressed={isRulerActive}
            data-active={isRulerActive}
            onClick={onToggleRuler}
          >
            <RulerIcon />
            <span className={style.tooltip} role="tooltip">
              Ruler
            </span>
          </button>
          <button
            className={style.controlTab}
            type="button"
            aria-label={isSaving ? 'Saving spaceship' : 'Save spaceship'}
            data-loading={isSaving}
            disabled={isSaving}
            onClick={handleSaveSpaceship}
          >
            {isSaving ? (
              <span className={style.loadingSpinner} aria-hidden="true" />
            ) : (
              <SaveIcon />
            )}
            <span className={style.tooltip} role="tooltip">
              {isSaving ? 'Saving spaceship' : 'Save spaceship'}
            </span>
          </button>
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
                        {Math.round(
                          (hullDurability / MAX_HULL_DURABILITY) * 100,
                        )}
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
                            getModuleMaxDurability(selectedModule) ||
                          !canAfford(
                            inventory,
                            getRepairCost(
                              selectedModule.durability,
                              getModuleMaxDurability(selectedModule),
                              selectedModule.type === 'mining' ? 1.2 : 1,
                            ),
                          )
                        }
                        onClick={() => repairSelectedModule(selectedModule)}
                      >
                        Repair{' '}
                        {formatCost(
                          getRepairCost(
                            selectedModule.durability,
                            getModuleMaxDurability(selectedModule),
                            selectedModule.type === 'mining' ? 1.2 : 1,
                          ),
                        )}
                      </button>
                    </>
                  ) : modulePanelSelection.type === 'hull' ? (
                    <>
                      <header>
                        <span>Hull</span>
                        <small>Ship system</small>
                      </header>
                      <div className={style.durabilityItem}>
                        <span>Integrity</span>
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
                      <button
                        type="button"
                        disabled={
                          hullDurability >= MAX_HULL_DURABILITY ||
                          !canAfford(
                            inventory,
                            getRepairCost(
                              hullDurability,
                              MAX_HULL_DURABILITY,
                              2,
                            ),
                          )
                        }
                        onClick={repairHull}
                      >
                        Repair{' '}
                        {formatCost(
                          getRepairCost(hullDurability, MAX_HULL_DURABILITY, 2),
                        )}
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
                            0) >= MAX_THRUSTER_DURABILITY ||
                          !canAfford(
                            inventory,
                            getRepairCost(
                              thrusterDurability[modulePanelSelection.index] ??
                                0,
                              MAX_THRUSTER_DURABILITY,
                              0.8,
                            ),
                          )
                        }
                        onClick={() =>
                          repairThruster(
                            modulePanelSelection.index,
                            thrusterDurability[modulePanelSelection.index] ?? 0,
                          )
                        }
                      >
                        Repair{' '}
                        {formatCost(
                          getRepairCost(
                            thrusterDurability[modulePanelSelection.index] ?? 0,
                            MAX_THRUSTER_DURABILITY,
                            0.8,
                          ),
                        )}
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
                {renderManualThrusterControl(
                  0,
                  'Top thruster',
                  style.thrustersTop,
                )}
                {renderManualThrusterControl(
                  3,
                  'Left thruster',
                  style.thrustersLeft,
                )}
                <div
                  className={`${style.speedActions} ${style.thrustersCenter}`}
                >
                  <div className={style.speedMetrics}>
                    <span className={style.speedMetric}>
                      <small>Acceleration</small>—
                    </span>
                    <span className={style.speedMetric}>
                      <small>
                        {activeThrusters ? 'Active time' : 'Time remaining'}
                      </small>
                      {thrustersTimeSeconds !== undefined
                        ? formatDuration(thrustersTimeSeconds)
                        : '—'}
                    </span>
                  </div>
                  <button
                    type="button"
                    data-running={isEngineRunning}
                    disabled={
                      isEngineRunning
                        ? !onStopEngines
                        : !canStartThrusters || !onStartThrusters
                    }
                    onClick={isEngineRunning ? stopEngines : startThrusters}
                  >
                    {isEngineRunning ? 'Stop engines' : 'Start'}
                  </button>
                </div>
                {renderManualThrusterControl(
                  1,
                  'Right thruster',
                  style.thrustersRight,
                )}
                {renderManualThrusterControl(
                  2,
                  'Bottom thruster',
                  style.thrustersBottom,
                )}
              </div>
            </DraggablePanel>
          )}
        </div>
      </section>

      <div className={style.telemetryDock}>
        <button
          className={style.miningToggle}
          type="button"
          data-active={miningModuleActive}
          disabled={!miningModule || miningModule.durability <= 0}
          onClick={toggleMiningModule}
        >
          Mining {miningModuleActive ? 'Active' : 'Off'}
        </button>
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
