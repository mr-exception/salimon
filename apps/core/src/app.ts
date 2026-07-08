import cors from 'cors';
import express, { type ErrorRequestHandler } from 'express';
import { contactsRouter } from './routes/contacts';
import { spaceshipsRouter } from './routes/spaceships';
import { updatesRouter } from './routes/updates';
import { worldRouter } from './routes/world';
import { spaceshipSecurityCode } from './middleware';

const DEFAULT_CORS_ORIGIN = 'http://localhost:5173';

export function createApp() {
  const app = express();
  app.use(
    cors({
      origin: process.env.CORS_ORIGIN ?? DEFAULT_CORS_ORIGIN,
    }),
  );
  app.use(express.json({ limit: '1mb' }));
  app.use(spaceshipSecurityCode);

  app.get('/health', (_request, response) => {
    response.json({ ok: true });
  });
  app.use('/world', worldRouter);
  app.use('/spaceship', spaceshipsRouter);
  app.use('/contacts', contactsRouter);
  app.use('/updates', updatesRouter);

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
