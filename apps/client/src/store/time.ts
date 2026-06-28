import { atom, useAtomValue, useSetAtom } from 'jotai';

export const timeSpeedAtom = atom(1);

export function useTimeSpeed() {
  return useAtomValue(timeSpeedAtom);
}

export function useSetTimeSpeed() {
  return useSetAtom(timeSpeedAtom);
}
