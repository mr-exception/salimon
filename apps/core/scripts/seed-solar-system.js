#!/usr/bin/env node

const fs = require('node:fs/promises');
const path = require('node:path');
const dotenv = require('dotenv');
const mongoose = require('mongoose');

dotenv.config({ path: path.resolve(__dirname, '../.env') });

const MIN_RENDER_SHAPE_SCREEN_WIDTH = 16;
const MIN_RENDER_NAME_TO_SHAPE_ZOOM_RATIO = 0.01;
const SYSTEMS_COLLECTION_NAME = 'systems';
const SOLAR_SYSTEM_FILE = path.resolve(__dirname, '../data/solar-system.json');

async function main() {
  const uri = process.env.MONGODB_URI;
  if (!uri) throw new Error('MONGODB_URI is not configured');

  const solarSystem = JSON.parse(await fs.readFile(SOLAR_SYSTEM_FILE, 'utf8'));
  if (!solarSystem?.name || !Array.isArray(solarSystem.bodies)) {
    throw new Error('Solar System seed file has an invalid shape.');
  }

  const bodies = solarSystem.bodies.map(withMinZoomRenderShape);
  const primary = bodies.find((body) => body.type === 'star') ?? bodies[0];
  if (!primary?.position?.x || !primary.position.y) {
    throw new Error('Solar System seed is missing a primary position.');
  }

  await mongoose.connect(uri, { serverSelectionTimeoutMS: 5_000 });
  const systemsCollection = mongoose.connection.collection(
    SYSTEMS_COLLECTION_NAME,
  );
  await systemsCollection.createIndex({ name: 1 }, { unique: true });
  await systemsCollection.createIndex({
    'primaryPosition.x': 1,
    'primaryPosition.y': 1,
  });
  await systemsCollection.createIndex({ 'bodies.name': 1 });

  await systemsCollection.replaceOne(
    { name: solarSystem.name },
    {
      name: solarSystem.name,
      primaryPosition: {
        x: mongoose.Types.Decimal128.fromString(primary.position.x),
        y: mongoose.Types.Decimal128.fromString(primary.position.y),
      },
      bodies,
      updatedAt: new Date(),
    },
    { upsert: true },
  );

  console.log(`Seeded ${bodies.length} Solar System bodies into systems.`);
}

function withMinZoomRenderShape(body) {
  const minZoomRenderShape =
    body.minZoomRenderShape ?? getMinZoomRenderShape(body.radius);

  return {
    ...body,
    minZoomRenderShape,
    minZoomRenderName:
      body.minZoomRenderName ??
      body.renderZoomLevel ??
      minZoomRenderShape * MIN_RENDER_NAME_TO_SHAPE_ZOOM_RATIO,
  };
}

function getMinZoomRenderShape(radius) {
  const radiusNumber = Number(radius);
  if (!Number.isFinite(radiusNumber) || radiusNumber <= 0) return 0;

  return MIN_RENDER_SHAPE_SCREEN_WIDTH / 2 / radiusNumber;
}

main()
  .catch((error) => {
    console.error(error);
    process.exitCode = 1;
  })
  .finally(async () => {
    await mongoose.disconnect();
  });
