import { cloneWorldData } from './clone-world-data';
import { requireWorldData } from './state';
import { start } from './start';

export async function getWorldData() {
  await start();
  return cloneWorldData(requireWorldData());
}

