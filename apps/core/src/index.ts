import 'dotenv/config';
import { createApp } from './app';

const DEFAULT_PORT = 3000;
const port = Number(process.env.PORT ?? DEFAULT_PORT);

if (!Number.isInteger(port) || port <= 0) {
  throw new Error('PORT must be a positive integer');
}

const app = createApp();
app.listen(port, () => {
  console.log(`Core API listening on http://localhost:${port}`);
});
