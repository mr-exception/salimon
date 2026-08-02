import {
  useEffect,
  useLayoutEffect,
  useMemo,
  useState,
  type CSSProperties,
} from 'react';
import style from './tutorial-tour.module.css';
import {
  completeModuleTutorial,
  completeStartupTutorial,
} from './tutorial-storage';

export type TutorialFooterPanel =
  | 'thrusters'
  | 'modules'
  | 'research'
  | 'fabricator'
  | 'mining';

export type TutorialFooterControl = {
  panel?: TutorialFooterPanel;
  moduleType?: 'mining' | 'fabricator';
};

type TutorialStep = {
  id: string;
  title: string;
  body: string;
  target?: string;
  targetRect?: () => DOMRect;
  footerControl?: TutorialFooterControl;
};

type TutorialTourProps = {
  active: boolean;
  onComplete: () => void;
  onOpenCommunicationThread?: (contactId: string) => void;
  onFooterControlChange?: (control?: TutorialFooterControl) => void;
  onRecenterSpaceship?: () => void;
};

type ModuleTutorialCalloutProps = {
  moduleType?: string;
  moduleName?: string;
  onComplete: () => void;
};

function getSpaceshipTargetRect() {
  const size = 92;
  return new DOMRect(
    window.innerWidth / 2 - size / 2,
    window.innerHeight / 2 - size / 2,
    size,
    size,
  );
}

function getFallbackTargetRect() {
  const width = Math.min(420, window.innerWidth - 32);
  const height = 160;
  return new DOMRect(
    window.innerWidth / 2 - width / 2,
    window.innerHeight / 2 - height / 2,
    width,
    height,
  );
}

function getTargetRect(step: TutorialStep) {
  if (step.targetRect) return step.targetRect();
  if (!step.target) return getFallbackTargetRect();

  const target = document.querySelector<HTMLElement>(
    `[data-tutorial-target="${step.target}"]`,
  );
  return target?.getBoundingClientRect() ?? getFallbackTargetRect();
}

function getCalloutPosition(targetRect: DOMRect) {
  const gap = 14;
  const width = Math.min(360, window.innerWidth - 32);
  const height = 208;
  const spaceRight = window.innerWidth - targetRect.right;
  const spaceLeft = targetRect.left;
  const spaceBottom = window.innerHeight - targetRect.bottom;
  const x =
    spaceRight >= width + gap || spaceRight >= spaceLeft
      ? targetRect.right + gap
      : targetRect.left - width - gap;
  const y =
    spaceBottom >= height + gap
      ? targetRect.top
      : Math.min(targetRect.bottom - height, window.innerHeight - height - 16);

  return {
    left: Math.max(16, Math.min(x, window.innerWidth - width - 16)),
    top: Math.max(16, y),
    width,
  };
}

export function TutorialTour({
  active,
  onComplete,
  onOpenCommunicationThread,
  onFooterControlChange,
  onRecenterSpaceship,
}: TutorialTourProps) {
  const [stepIndex, setStepIndex] = useState(0);
  const [targetRect, setTargetRect] = useState<DOMRect>(() =>
    getFallbackTargetRect(),
  );
  const steps = useMemo<TutorialStep[]>(
    () => [
      {
        id: 'ship',
        title: 'This ship is you',
        body: 'The ship marker is you. Keep it intact, fueled, and under control while the Core keeps everyone alive.',
        targetRect: getSpaceshipTargetRect,
      },
      {
        id: 'mission',
        title: 'Mission',
        body: 'You are leaving Earth and crossing deep space toward Absenat. The map is your flight deck for navigation, distance, and nearby hazards.',
        targetRect: getSpaceshipTargetRect,
      },
      {
        id: 'research',
        title: 'Research',
        body: 'Research is where ship capabilities are unlocked and upgraded. Start here when you need new systems or stronger modules.',
        target: 'footer-panel-research',
        footerControl: { panel: 'research' },
      },
      {
        id: 'unlock-mining',
        title: 'Unlock Mining',
        body: 'Click Unlock on the Mining module. Mining lets the ship extract asteroid materials for research, repairs, refueling, and fabrication.',
        target: 'research-unlock-mining',
        footerControl: { panel: 'research' },
      },
      {
        id: 'unlock-fabricator',
        title: 'Fabricator',
        body: 'Click Unlock on the Fabricator module. The fabricator turns raw materials into repair kits and fuel cells for longer missions.',
        target: 'research-unlock-fabricator',
        footerControl: { panel: 'research' },
      },
      {
        id: 'status',
        title: 'Status labels',
        body: 'The bottom-right ship telemetry shows whether you are flying, landed, or crashed, plus engine power, fuel, and current speed.',
        target: 'ship-telemetry',
      },
      {
        id: 'thrusters',
        title: 'Thruster modules',
        body: 'Thrusters move the ship. Use the vector pad or axis fields to burn in a direction, and watch durability and fuel before committing.',
        target: 'footer-panel-thrusters',
        footerControl: { panel: 'thrusters' },
      },
      {
        id: 'communications',
        title: 'Communications',
        body: 'This is ship communications. The first thread is from the Chief of EASA, with your opening mission briefing and contact channel.',
        target: 'communications-dialog',
      },
    ],
    [],
  );
  const currentStep = steps[stepIndex];
  const calloutPosition = getCalloutPosition(targetRect);
  const scrimStyle = {
    '--target-left': `${targetRect.left}px`,
    '--target-top': `${targetRect.top}px`,
    '--target-width': `${targetRect.width}px`,
    '--target-height': `${targetRect.height}px`,
  } as CSSProperties;

  useEffect(() => {
    if (!active) return;

    onRecenterSpaceship?.();
  }, [active, onRecenterSpaceship]);

  useEffect(() => {
    if (!active) return;

    onFooterControlChange?.(currentStep.footerControl);
  }, [active, currentStep, onFooterControlChange]);

  useEffect(() => {
    if (!active || currentStep.id !== 'communications') return;

    onFooterControlChange?.(undefined);
    onOpenCommunicationThread?.('easa-chief');
  }, [active, currentStep, onFooterControlChange, onOpenCommunicationThread]);

  useLayoutEffect(() => {
    if (!active) return;

    let frame = 0;
    const updateTarget = () => {
      setTargetRect(getTargetRect(currentStep));
    };
    const scheduleUpdate = () => {
      window.cancelAnimationFrame(frame);
      frame = window.requestAnimationFrame(updateTarget);
    };

    updateTarget();
    frame = window.requestAnimationFrame(updateTarget);
    window.addEventListener('resize', scheduleUpdate);
    window.addEventListener('scroll', scheduleUpdate, true);
    return () => {
      window.cancelAnimationFrame(frame);
      window.removeEventListener('resize', scheduleUpdate);
      window.removeEventListener('scroll', scheduleUpdate, true);
    };
  }, [active, currentStep]);

  if (!active) return null;

  const finish = () => {
    completeStartupTutorial();
    onFooterControlChange?.(undefined);
    onComplete();
  };
  const goNext = () => {
    if (stepIndex >= steps.length - 1) {
      finish();
      return;
    }

    setStepIndex((index) => index + 1);
  };

  return (
    <div className={style.overlay} role="dialog" aria-modal="true">
      <div className={style.scrim} style={scrimStyle} />
      <section
        className={style.callout}
        style={{
          left: calloutPosition.left,
          top: calloutPosition.top,
          width: calloutPosition.width,
        }}
        aria-labelledby="tutorial-title"
      >
        <small>
          {stepIndex + 1} / {steps.length}
        </small>
        <h2 id="tutorial-title">{currentStep.title}</h2>
        <p>{currentStep.body}</p>
        <div className={style.actions}>
          <button type="button" onClick={finish}>
            Skip tutorial
          </button>
          <button type="button" onClick={goNext}>
            {stepIndex >= steps.length - 1 ? 'Finish' : 'Next'}
          </button>
        </div>
      </section>
    </div>
  );
}

export function ModuleTutorialCallout({
  moduleType,
  moduleName,
  onComplete,
}: ModuleTutorialCalloutProps) {
  const [targetRect, setTargetRect] = useState<DOMRect>(() =>
    getFallbackTargetRect(),
  );

  useLayoutEffect(() => {
    if (!moduleType) return;

    const updateTarget = () => {
      const target = document.querySelector<HTMLElement>(
        '[data-tutorial-target="footer-module-inspector"]',
      );
      setTargetRect(target?.getBoundingClientRect() ?? getFallbackTargetRect());
    };
    const frame = window.requestAnimationFrame(updateTarget);
    updateTarget();
    window.addEventListener('resize', updateTarget);
    return () => {
      window.cancelAnimationFrame(frame);
      window.removeEventListener('resize', updateTarget);
    };
  }, [moduleType]);

  if (!moduleType) return null;

  const calloutPosition = getCalloutPosition(targetRect);
  const scrimStyle = {
    '--target-left': `${targetRect.left}px`,
    '--target-top': `${targetRect.top}px`,
    '--target-width': `${targetRect.width}px`,
    '--target-height': `${targetRect.height}px`,
  } as CSSProperties;
  const finish = () => {
    completeModuleTutorial(moduleType);
    onComplete();
  };

  return (
    <div className={style.overlay} role="dialog" aria-modal="true">
      <div className={style.scrim} style={scrimStyle} />
      <section
        className={style.callout}
        style={{
          left: calloutPosition.left,
          top: calloutPosition.top,
          width: calloutPosition.width,
        }}
        aria-labelledby="module-tutorial-title"
      >
        <small>Module briefing</small>
        <h2 id="module-tutorial-title">{moduleName ?? 'Ship module'}</h2>
        <p>
          This module adds a ship system with its own durability, drain rate,
          and upgrades. Inspect it here before relying on it during flight.
        </p>
        <div className={style.actions}>
          <button type="button" onClick={finish}>
            Skip
          </button>
          <button type="button" onClick={finish}>
            Got it
          </button>
        </div>
      </section>
    </div>
  );
}
