const path = require('node:path');
const dotenv = require('dotenv');
const mongoose = require('mongoose');

dotenv.config({ path: path.resolve(__dirname, '../.env') });

const MIN_RENDER_SHAPE_SCREEN_WIDTH = 16;
const MIN_RENDER_NAME_TO_SHAPE_ZOOM_RATIO = 0.01;
const BODY_COLLECTIONS = ['planets', 'moons', 'stars'];

function minZoomExpression(radiusExpression) {
  return {
    $cond: [
      { $gt: [{ $toDouble: radiusExpression }, 0] },
      {
        $divide: [
          MIN_RENDER_SHAPE_SCREEN_WIDTH / 2,
          { $toDouble: radiusExpression },
        ],
      },
      0,
    ],
  };
}

function minNameZoomExpression(radiusExpression) {
  return {
    $multiply: [
      minZoomExpression(radiusExpression),
      MIN_RENDER_NAME_TO_SHAPE_ZOOM_RATIO,
    ],
  };
}

async function updateBodyCollection(db, collectionName) {
  const result = await db.collection(collectionName).updateMany({}, [
    {
      $set: {
        minZoomRenderShape: minZoomExpression('$radius'),
        minZoomRenderName: minNameZoomExpression('$radius'),
      },
    },
  ]);

  return {
    collectionName,
    matched: result.matchedCount,
    modified: result.modifiedCount,
  };
}

async function updateSystemsCollection(db) {
  const result = await db.collection('systems').updateMany({}, [
    {
      $set: {
        bodies: {
          $map: {
            input: '$bodies',
            as: 'body',
            in: {
              $mergeObjects: [
                '$$body',
                {
                  minZoomRenderShape: minZoomExpression('$$body.radius'),
                  minZoomRenderName: minNameZoomExpression('$$body.radius'),
                },
              ],
            },
          },
        },
      },
    },
  ]);

  return {
    collectionName: 'systems',
    matched: result.matchedCount,
    modified: result.modifiedCount,
  };
}

async function main() {
  if (!process.env.MONGODB_URI) {
    throw new Error('MONGODB_URI is not configured');
  }

  await mongoose.connect(process.env.MONGODB_URI, {
    serverSelectionTimeoutMS: 5_000,
  });

  const db = mongoose.connection.db;
  const bodyResults = [];
  for (const collectionName of BODY_COLLECTIONS) {
    bodyResults.push(await updateBodyCollection(db, collectionName));
  }
  const systemsResult = await updateSystemsCollection(db);

  console.log(JSON.stringify([...bodyResults, systemsResult], null, 2));
}

main()
  .catch((error) => {
    console.error(error);
    process.exitCode = 1;
  })
  .finally(async () => {
    await mongoose.disconnect();
  });
