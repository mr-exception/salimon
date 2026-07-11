import cors from 'cors';
import express, { type ErrorRequestHandler } from 'express';
import { contactsRouter, spaceshipsRouter } from '@routes';
import { spaceshipSecurityCode } from './middleware';
export function createApp() {
  const app = express();
  app.use(cors());
  app.use(express.json({ limit: '1mb' }));
  app.use(spaceshipSecurityCode);

  app.get('/health', (_request, response) => {
    response.json({ ok: true });
  });
  app.use('/spaceship', spaceshipsRouter);
  app.use('/contacts', contactsRouter);

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
