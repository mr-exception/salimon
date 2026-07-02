import { atom } from 'jotai';
export * from './bootstrap';
export * from './world';

export type FooterView = 'navigation' | 'ship';

export const activeViewAtom = atom<FooterView>('navigation');
