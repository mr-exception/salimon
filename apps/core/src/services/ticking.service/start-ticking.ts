import { RepositoryService } from '../repository.service';
import { TICK_INTERVAL_MS } from './constants';
import { tickingState } from './state';
import { tick } from './tick';

export async function startTicking() {
  await RepositoryService.start();
  await tick(new Date());

  tickingState.timer ??= setInterval(() => {
    void tick(new Date()).catch((error: unknown) => {
      console.error('Failed to tick world data', error);
    });
  }, TICK_INTERVAL_MS);
}

