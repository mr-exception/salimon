export const TUTORIAL_STORAGE_KEY = 'salimon:tutorial';

type TutorialStorage = {
  startupTourCompleted?: boolean;
  explainedModules?: Record<string, boolean>;
};

function readTutorialStorage(): TutorialStorage {
  try {
    const value = window.localStorage.getItem(TUTORIAL_STORAGE_KEY);
    if (!value) return {};

    const parsed = JSON.parse(value) as TutorialStorage;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

function writeTutorialStorage(nextStorage: TutorialStorage) {
  try {
    window.localStorage.setItem(
      TUTORIAL_STORAGE_KEY,
      JSON.stringify({
        ...readTutorialStorage(),
        ...nextStorage,
      }),
    );
  } catch {
    // Tutorial progress is non-critical and can be retried next session.
  }
}

export function startupTutorialIsComplete() {
  if (typeof window === 'undefined') return true;
  return readTutorialStorage().startupTourCompleted === true;
}

export function completeStartupTutorial() {
  writeTutorialStorage({ startupTourCompleted: true });
}

export function moduleTutorialIsComplete(moduleType: string) {
  if (typeof window === 'undefined') return true;
  return readTutorialStorage().explainedModules?.[moduleType] === true;
}

export function completeModuleTutorial(moduleType: string) {
  writeTutorialStorage({
    explainedModules: {
      ...readTutorialStorage().explainedModules,
      [moduleType]: true,
    },
  });
}
