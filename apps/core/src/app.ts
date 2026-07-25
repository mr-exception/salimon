import cors from 'cors';
import express, { type ErrorRequestHandler } from 'express';
import {
  contactsRouter,
  metricsRouter,
  searchRouter,
  spaceshipsRouter,
  worldRouter,
} from '@routes';
import { WorldAssetService } from '@services';
import { apiRequestTiming, spaceshipSecurityCode } from './middleware';
export function createApp() {
  const app = express();
  app.use(apiRequestTiming);
  app.use(cors());
  app.use(express.json({ limit: '1mb' }));
  app.use(spaceshipSecurityCode);

  app.get('/health', (_request, response) => {
    response.json({ ok: true });
  });
  app.use('/spaceship', spaceshipsRouter);
  app.use('/contacts', contactsRouter);
  app.use('/search', searchRouter);
  app.use('/metrics', metricsRouter);
  app.use(
    '/world/assets',
    express.static(WorldAssetService.getAssetsDirectory(), {
      immutable: true,
      maxAge: '1h',
    }),
  );
  app.use('/world', worldRouter);

  const errorHandler: ErrorRequestHandler = (
    error,
    _request,
    response,
    next,
  ) => {
    void next;
    console.error('Unhandled core error', error);
    response.status(500).json({ error: 'Internal server error' });
  };
  app.use(errorHandler);

  return app;
}
