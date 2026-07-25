#!/usr/bin/env node

const { createHash, randomUUID } = require('node:crypto');
const fs = require('node:fs/promises');
const path = require('node:path');
const dotenv = require('dotenv');
const mongoose = require('mongoose');

dotenv.config({ path: path.resolve(__dirname, '../.env') });

const DEFAULT_PAGE_SIZE = 500;
const SYSTEMS_COLLECTION_NAME = 'systems';
const DEFAULT_STATS_FILE = path.resolve(
  __dirname,
  'fetch-world-data.stats.json',
);

const METERS_PER_PARSEC = 30_856_775_814_913_672n;
const SUN_GALACTOCENTRIC_RADIUS_METERS = 245_978_992_287_100_800_000n;
const SOLAR_MASS_KG = 1.9885e30;
const SOLAR_RADIUS_METERS = 695_700_000;
const EARTH_MASS_KG = 5.97237e24;
const EARTH_RADIUS_METERS = 6_371_000;
const DEFAULT_STAR_ORBITAL_SPEED = '220000';
const DEFAULT_PLANET_ORBITAL_SPEED = '30000';
const DEFAULT_ROTATION_PERIOD_SECONDS = 86_164.1;
const GALACTIC_CENTER = {
  type: 'blackhole',
  name: 'Sagittarius A*',
  isReal: true,
  position: { x: '0', y: '0' },
  orbitalCenter: null,
  clockwise: false,
  speed: '0',
  mass: '8544584500000000000000000000000000000',
  radius: '0',
  rotationPeriodSeconds: 0,
  positionCapturedAt: 1784419200000,
};
const GALACTIC_CENTER_SYSTEM = [GALACTIC_CENTER];
const SOLAR_SYSTEM = require('../data/solar-system.json').bodies;

const EQUATORIAL_TO_GALACTIC = [
  [-0.0548755604162154, -0.873437090234885, -0.4838350155487132],
  [0.4941094278755837, -0.4448296299600112, 0.7469822444972189],
  [-0.8676661490190047, -0.1980763734312015, 0.4559837761750669],
];

const sources = [
  {
    name: 'NASA Exoplanet Archive',
    getCursorLabel(cursor) {
      return cursor
        ? `after hostname=${JSON.stringify(cursor.hostname)}, planet=${JSON.stringify(cursor.planetName)}`
        : 'from beginning';
    },
    getNextCursor(rows) {
      for (let index = rows.length - 1; index >= 0; index -= 1) {
        const row = rows[index];
        const hostname = cleanName(row.hostname);
        const planetName = cleanName(row.pl_name);

        if (hostname && planetName) {
          return { hostname, planetName };
        }
      }

      return null;
    },
    getOpenSystemKey(rows) {
      for (let index = rows.length - 1; index >= 0; index -= 1) {
        const hostName = cleanName(rows[index].hostname);
        const key = normalizeKey(hostName);

        if (key) {
          return key;
        }
      }

      return null;
    },
    buildUrl(pageSize, cursor) {
      const cursorFilter = cursor
        ? `
          and (
            hostname > '${escapeSqlString(cursor.hostname)}'
            or (
              hostname = '${escapeSqlString(cursor.hostname)}'
              and pl_name > '${escapeSqlString(cursor.planetName)}'
            )
          )
        `
        : '';
      const query = `
        select top ${pageSize}
          sy_name, hostname, pl_name, ra, dec, sy_dist, st_mass, st_rad,
          pl_bmasse, pl_rade, pl_orbsmax, pl_orbper
        from pscomppars
        where sy_dist is not null
          and ra is not null
          and dec is not null
          and hostname is not null
          and pl_name is not null
          ${cursorFilter}
        order by hostname, pl_name
      `;

      return `https://exoplanetarchive.ipac.caltech.edu/TAP/sync?query=${encodeURIComponent(
        query,
      )}&format=json`;
    },
    async readRows(response) {
      return response.json();
    },
    parseRows(rows) {
      const systemsByHost = new Map();

      for (const row of rows) {
        const hostName =
          cleanName(row.hostname ?? row.sy_name) ?? createRandomName('System');
        const distanceParsecs = numberOrNull(row.sy_dist);
        const ra = numberOrNull(row.ra);
        const dec = numberOrNull(row.dec);

        if (distanceParsecs === null || ra === null || dec === null) {
          continue;
        }

        const system = getOrCreateSystem(systemsByHost, hostName, () => [
          createStar({
            name: hostName,
            ra,
            dec,
            distanceParsecs,
            massSolar: numberOrNull(row.st_mass),
            radiusSolar: numberOrNull(row.st_rad),
          }),
        ]);

        const planetName =
          cleanName(row.pl_name) ?? createRandomName(`${hostName} Planet`);
        if (system.some((body) => body.name === planetName)) {
          continue;
        }

        system.push(
          createPlanet({
            name: planetName,
            orbitalCenter: hostName,
            massEarth: numberOrNull(row.pl_bmasse),
            radiusEarth: numberOrNull(row.pl_rade),
            semiMajorAxisAu: numberOrNull(row.pl_orbsmax),
            orbitalPeriodDays: numberOrNull(row.pl_orbper),
          }),
        );
      }

      return [...systemsByHost.values()];
    },
  },
  {
    name: 'ESA Gaia Archive',
    getCursorLabel(cursor) {
      return `offset ${cursor?.offset ?? 0}`;
    },
    getNextCursor(rows, cursor, pageSize) {
      return { offset: (cursor?.offset ?? 0) + pageSize };
    },
    getOpenSystemKey() {
      return null;
    },
    buildUrl(pageSize, cursor) {
      const offset = cursor?.offset ?? 0;
      const query = `
        SELECT TOP ${pageSize}
          source_id, ra, dec, parallax, phot_g_mean_mag
        FROM gaiadr3.gaia_source
        WHERE parallax > 0
          AND ra IS NOT NULL
          AND dec IS NOT NULL
        ORDER BY parallax DESC
        OFFSET ${offset}
      `;

      return `https://gea.esac.esa.int/tap-server/tap/sync?REQUEST=doQuery&LANG=ADQL&FORMAT=json&QUERY=${encodeURIComponent(
        query,
      )}`;
    },
    async readRows(response) {
      const payload = await response.json();
      const fields = payload.metadata.map((field) => field.name);

      return payload.data.map((values) =>
        Object.fromEntries(
          values.map((value, index) => [fields[index], value]),
        ),
      );
    },
    parseRows(rows) {
      return rows
        .map((row) => {
          const sourceId =
            cleanName(row.source_id) ?? createRandomName('Gaia Source');
          const parallaxMas = numberOrNull(row.parallax);
          const ra = numberOrNull(row.ra);
          const dec = numberOrNull(row.dec);

          if (
            parallaxMas === null ||
            parallaxMas <= 0 ||
            ra === null ||
            dec === null
          ) {
            return null;
          }

          return [
            createStar({
              name: `Gaia DR3 ${sourceId}`,
              ra,
              dec,
              distanceParsecs: 1000 / parallaxMas,
              massSolar: null,
              radiusSolar: null,
            }),
          ];
        })
        .filter(Boolean);
    },
  },
];

async function main() {
  const pageSize = getPageSize();
  const generatedAt = new Date();
  const endpointStats = await readEndpointStats();
  const seenSystemNames = new Set(
    SOLAR_SYSTEM.map((body) => normalizeKey(body.name)).filter(Boolean),
  );
  const systemWriter = await createWorldSystemWriter({ generatedAt });
  let fetchedSystemCount = 0;

  try {
    if (hasEndpointStats(endpointStats)) {
      console.log(
        `Using endpoint stats from ${endpointStats.path}; cached endpoints will be skipped.`,
      );
    } else {
      await systemWriter.clear();
    }

    await systemWriter.addSystem(GALACTIC_CENTER_SYSTEM);
    await systemWriter.addSystem(SOLAR_SYSTEM);
    await systemWriter.flush();

    for (const source of sources) {
      console.log(`Starting ${source.name} with page size ${pageSize}.`);
      const pendingSystemsByKey = new Map();
      let cursor = null;
      let pageNumber = 1;
      let totalRows = 0;
      let sourceAddedCount = 0;

      const storeCompletedSystems = async (openSystemKey) => {
        let addedCount = 0;

        for (const [key, system] of pendingSystemsByKey) {
          if (key === openSystemKey) {
            continue;
          }

          if (seenSystemNames.has(key)) {
            pendingSystemsByKey.delete(key);
            continue;
          }

          seenSystemNames.add(key);
          const storedSystem = fillEmptySystem(system);
          await systemWriter.addSystem(storedSystem);
          pendingSystemsByKey.delete(key);
          addedCount += 1;
          fetchedSystemCount += 1;
        }

        return addedCount;
      };

      while (true) {
        const cursorLabel = source.getCursorLabel(cursor);
        console.log(
          `${source.name}: fetching page ${pageNumber} (${cursorLabel}).`,
        );

        const endpoint = source.buildUrl(pageSize, cursor);

        const page = await readEndpointPage({
          endpointStats,
          endpoint,
          cursor,
          source,
          pageNumber,
          pageSize,
        });
        totalRows += page.rowCount;

        if (page.skipped) {
          console.log(
            `${source.name}: page ${pageNumber} skipped cached endpoint with ${page.rowCount} rows (${totalRows} total rows).`,
          );

          if (page.rowCount < pageSize) {
            console.log(`${source.name}: finished after ${pageNumber} pages.`);
            break;
          }

          if (!page.nextCursor) {
            console.log(
              `${source.name}: stopped because cached endpoint has no next page cursor.`,
            );
            break;
          }

          cursor = page.nextCursor;
          pageNumber += 1;
          await writeEndpointStats(endpointStats);
          continue;
        }

        const rows = page.rows;
        console.log(
          `${source.name}: page ${pageNumber} returned ${page.rowCount} rows (${totalRows} total rows).`,
        );

        const pageSystems = source.parseRows(rows);
        mergeSourceSystems(pendingSystemsByKey, pageSystems);
        const openSystemKey = source.getOpenSystemKey?.(rows) ?? null;
        const addedCount = await storeCompletedSystems(openSystemKey);
        sourceAddedCount += addedCount;
        const storedCount = await systemWriter.flush();
        console.log(
          `${source.name}: page ${pageNumber} parsed ${pageSystems.length} systems and wrote ${storedCount} completed systems (${pendingSystemsByKey.size} pending).`,
        );
        await writeEndpointStats(endpointStats);

        if (rows.length < pageSize) {
          console.log(`${source.name}: finished after ${pageNumber} pages.`);
          break;
        }

        const nextCursor = page.nextCursor;
        if (!nextCursor) {
          console.log(
            `${source.name}: stopped because no next page cursor could be derived.`,
          );
          break;
        }

        cursor = nextCursor;
        pageNumber += 1;
      }

      const addedCount = await storeCompletedSystems(null);
      sourceAddedCount += addedCount;
      const storedCount = await systemWriter.flush();

      console.log(
        `Added ${sourceAddedCount} systems from ${source.name}; ${storedCount} final systems written for source; ${fetchedSystemCount} fetched systems total.`,
      );
    }

    console.log(
      `Finished writing MongoDB collection ${SYSTEMS_COLLECTION_NAME}.`,
    );
    await writeEndpointStats(endpointStats);
  } finally {
    await mongoose.disconnect();
  }
}

async function readEndpointPage({
  endpointStats,
  endpoint,
  cursor,
  source,
  pageNumber,
  pageSize,
}) {
  const endpointKey = getEndpointKey(endpoint);
  const cachedEndpoint = endpointStats.endpoints[endpointKey];

  if (cachedEndpoint) {
    cachedEndpoint.lastUsedAt = new Date().toISOString();
    return {
      skipped: true,
      rowCount: cachedEndpoint.rowCount,
      nextCursor: cachedEndpoint.nextCursor ?? null,
    };
  }

  const response = await fetch(endpoint, {
    headers: { accept: 'application/json' },
  });

  if (!response.ok) {
    const message = await response.text();
    throw new Error(
      `${source.name} page ${pageNumber} returned ${response.status}: ${message.slice(
        0,
        500,
      )}`,
    );
  }

  const rows = await source.readRows(response);
  const nextCursor =
    rows.length < pageSize
      ? null
      : source.getNextCursor(rows, cursor, pageSize);

  endpointStats.endpoints[endpointKey] = {
    source: source.name,
    pageNumber,
    cursor,
    nextCursor,
    rowCount: rows.length,
    fetchedAt: new Date().toISOString(),
    lastUsedAt: new Date().toISOString(),
  };

  return {
    skipped: false,
    rows,
    rowCount: rows.length,
    nextCursor,
  };
}

async function readEndpointStats() {
  const statsPath = getStatsFilePath();

  try {
    const rawStats = await fs.readFile(statsPath, 'utf8');
    const stats = JSON.parse(rawStats);

    if (!stats || typeof stats !== 'object' || !stats.endpoints) {
      throw new Error('stats file has an invalid shape');
    }

    return {
      path: statsPath,
      endpoints: sanitizeEndpointStats(stats.endpoints),
    };
  } catch (error) {
    if (error.code !== 'ENOENT') throw error;

    return {
      path: statsPath,
      endpoints: {},
    };
  }
}

function sanitizeEndpointStats(endpoints) {
  return Object.fromEntries(
    Object.entries(endpoints).map(([key, endpoint]) => {
      const safeKey = /^[a-f0-9]{64}$/.test(key) ? key : getEndpointKey(key);

      return [
        safeKey,
        {
          source: endpoint.source,
          pageNumber: endpoint.pageNumber,
          cursor: endpoint.cursor ?? null,
          nextCursor: endpoint.nextCursor ?? null,
          rowCount: endpoint.rowCount ?? endpoint.rows?.length ?? 0,
          fetchedAt: endpoint.fetchedAt,
          lastUsedAt: endpoint.lastUsedAt,
        },
      ];
    }),
  );
}

function hasEndpointStats(endpointStats) {
  return Object.keys(endpointStats.endpoints).length > 0;
}

function getEndpointKey(endpoint) {
  return createHash('sha256').update(endpoint).digest('hex');
}

async function writeEndpointStats(endpointStats) {
  await fs.mkdir(path.dirname(endpointStats.path), { recursive: true });
  await fs.writeFile(
    endpointStats.path,
    `${JSON.stringify(
      {
        updatedAt: new Date().toISOString(),
        endpoints: endpointStats.endpoints,
      },
      null,
      2,
    )}\n`,
  );
}

function getStatsFilePath() {
  return process.env.WORLD_FETCH_STATS_FILE
    ? path.resolve(process.env.WORLD_FETCH_STATS_FILE)
    : DEFAULT_STATS_FILE;
}

function getPageSize() {
  const rawPageSize = process.env.WORLD_FETCH_PAGE_SIZE;
  if (!rawPageSize) return DEFAULT_PAGE_SIZE;

  const pageSize = Number(rawPageSize);
  if (!Number.isInteger(pageSize) || pageSize <= 0) {
    throw new Error('WORLD_FETCH_PAGE_SIZE must be a positive integer.');
  }

  return pageSize;
}

async function createWorldSystemWriter({ generatedAt }) {
  const uri = process.env.MONGODB_URI;
  if (!uri) throw new Error('MONGODB_URI is not configured');

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

  const pendingSystemsByName = new Map();

  const writeSystems = async (systems) => {
    if (systems.length === 0) return 0;

    await systemsCollection.bulkWrite(
      systems.map((system) => {
        const name = getSystemName(system);
        return {
          replaceOne: {
            filter: { name },
            replacement: {
              name,
              primaryPosition: getPrimaryPosition(system),
              bodies: system,
              generatedAt,
              updatedAt: new Date(),
            },
            upsert: true,
          },
        };
      }),
      { ordered: false },
    );

    return systems.length;
  };

  return {
    async clear() {
      await systemsCollection.deleteMany({});
      console.log(
        `Cleared MongoDB collection ${SYSTEMS_COLLECTION_NAME} before fetching.`,
      );
    },
    async addSystem(system) {
      const name = getSystemName(system);
      const existingSystem = pendingSystemsByName.get(name);

      if (existingSystem) {
        mergeSystemBodies(existingSystem, system);
      } else {
        pendingSystemsByName.set(name, system);
      }
    },
    async flush() {
      const systemCount = await writeSystems([
        ...pendingSystemsByName.values(),
      ]);
      pendingSystemsByName.clear();

      if (systemCount > 0) {
        console.log(
          `Stored ${systemCount} unique systems in MongoDB collection ${SYSTEMS_COLLECTION_NAME}.`,
        );
      }

      return systemCount;
    },
  };
}

function mergeSystemBodies(targetSystem, sourceSystem) {
  const existingBodyNames = new Set(targetSystem.map((body) => body.name));

  for (const body of sourceSystem) {
    if (!existingBodyNames.has(body.name)) {
      targetSystem.push(body);
      existingBodyNames.add(body.name);
    }
  }
}

function getSystemName(system) {
  const primary = system.find((body) => body.type === 'star') ?? system[0];
  const name = cleanName(primary?.name);

  if (!name) {
    throw new Error('Cannot store a system without a primary body name.');
  }

  return name;
}

function getPrimaryPosition(system) {
  const primary = system.find((body) => body.type === 'star') ?? system[0];

  if (!primary?.position?.x || !primary.position.y) {
    throw new Error(
      `Cannot store ${getSystemName(system)} without a position.`,
    );
  }

  return {
    x: mongoose.Types.Decimal128.fromString(primary.position.x),
    y: mongoose.Types.Decimal128.fromString(primary.position.y),
  };
}

function fillEmptySystem(system) {
  const star = system.find((body) => body.type === 'star');
  const hasOrbitingBodies = system.some(
    (body) => body.type === 'planet' || body.type === 'moon',
  );

  if (!star || hasOrbitingBodies) {
    return system;
  }

  return [...system, ...createFakePlanets(star)];
}

function createFakePlanets(star) {
  const planetCount = 3 + Math.floor(seededUnit(`${star.name}:count`) * 3);
  const planets = [];

  for (let index = 0; index < planetCount; index += 1) {
    const name = `${star.name} Generated Planet ${index + 1}`;
    const massEarth = 0.2 + seededUnit(`${name}:mass`) * 9.8;
    const radiusEarth = 0.4 + seededUnit(`${name}:radius`) * 1.8;
    const semiMajorAxisAu =
      0.25 + index * 0.45 + seededUnit(`${name}:orbit`) * 0.35;
    const orbitalPeriodDays =
      35 + index * 80 + seededUnit(`${name}:period`) * 420;

    planets.push(
      createPlanet({
        name,
        orbitalCenter: star.name,
        isReal: false,
        massEarth,
        radiusEarth,
        semiMajorAxisAu,
        orbitalPeriodDays,
      }),
    );
  }

  return planets;
}

function mergeSourceSystems(systemsByKey, systems) {
  for (const system of systems) {
    const primary = system.find((body) => body.type === 'star') ?? system[0];
    const key = normalizeKey(primary?.name);

    if (!key) {
      continue;
    }

    const existingSystem = systemsByKey.get(key);
    if (!existingSystem) {
      systemsByKey.set(key, system);
      continue;
    }

    const existingBodyNames = new Set(existingSystem.map((body) => body.name));
    for (const body of system) {
      if (!existingBodyNames.has(body.name)) {
        existingSystem.push(body);
        existingBodyNames.add(body.name);
      }
    }
  }
}

function escapeSqlString(value) {
  return String(value).replaceAll("'", "''");
}

function getOrCreateSystem(systemsByHost, hostName, createSystem) {
  const key = normalizeKey(hostName);
  if (!systemsByHost.has(key)) {
    systemsByHost.set(key, createSystem());
  }

  return systemsByHost.get(key);
}

function createStar({
  name,
  isReal = true,
  ra,
  dec,
  distanceParsecs,
  massSolar,
  radiusSolar,
}) {
  return {
    type: 'star',
    name,
    isReal,
    position: getGalactocentricPosition(ra, dec, distanceParsecs),
    orbitalCenter: null,
    clockwise: false,
    speed: DEFAULT_STAR_ORBITAL_SPEED,
    mass: toPositiveIntegerString((massSolar ?? 1) * SOLAR_MASS_KG),
    radius: toPositiveIntegerString((radiusSolar ?? 1) * SOLAR_RADIUS_METERS),
    rotationPeriodSeconds: DEFAULT_ROTATION_PERIOD_SECONDS,
    positionCapturedAt: Date.now(),
  };
}

function createPlanet({
  name,
  orbitalCenter,
  isReal = true,
  massEarth,
  radiusEarth,
  semiMajorAxisAu,
  orbitalPeriodDays,
}) {
  const orbitRadiusMeters = Math.max(
    (semiMajorAxisAu ?? 1) * 149_597_870_700,
    1,
  );
  const angle = seededAngle(name);

  return {
    type: 'planet',
    name,
    isReal,
    position: {
      x: toPositiveOrNegativeIntegerString(Math.cos(angle) * orbitRadiusMeters),
      y: toPositiveOrNegativeIntegerString(Math.sin(angle) * orbitRadiusMeters),
      relativeTo: orbitalCenter,
    },
    orbitalCenter,
    clockwise: false,
    speed: DEFAULT_PLANET_ORBITAL_SPEED,
    mass: toPositiveIntegerString((massEarth ?? 1) * EARTH_MASS_KG),
    radius: toPositiveIntegerString((radiusEarth ?? 1) * EARTH_RADIUS_METERS),
    rotationPeriodSeconds:
      orbitalPeriodDays === null
        ? DEFAULT_ROTATION_PERIOD_SECONDS
        : orbitalPeriodDays * 86_400,
    positionCapturedAt: Date.now(),
  };
}

function getGalactocentricPosition(raDegrees, decDegrees, distanceParsecs) {
  const ra = degreesToRadians(raDegrees);
  const dec = degreesToRadians(decDegrees);
  const equatorial = [
    Math.cos(dec) * Math.cos(ra),
    Math.cos(dec) * Math.sin(ra),
    Math.sin(dec),
  ];
  const galactic = EQUATORIAL_TO_GALACTIC.map((row) =>
    row.reduce((sum, value, index) => sum + value * equatorial[index], 0),
  );
  const distanceMeters =
    BigInt(Math.round(distanceParsecs)) * METERS_PER_PARSEC;
  const planarDistance =
    Number(distanceMeters) * Math.hypot(galactic[0], galactic[1]);
  const l = Math.atan2(galactic[1], galactic[0]);
  const xOffset = BigInt(Math.round(Math.cos(l) * planarDistance));
  const yOffset = BigInt(Math.round(Math.sin(l) * planarDistance));

  return {
    x: (SUN_GALACTOCENTRIC_RADIUS_METERS - xOffset).toString(),
    y: yOffset.toString(),
    relativeTo: 'Sagittarius A*',
  };
}

function seededAngle(value) {
  return seededUnit(value) * Math.PI * 2;
}

function seededUnit(value) {
  let hash = 0;
  for (let index = 0; index < value.length; index += 1) {
    hash = (hash * 31 + value.charCodeAt(index)) >>> 0;
  }

  return hash / 0xffffffff;
}

function cleanName(value) {
  if (value === null || value === undefined) return null;
  const name = String(value).trim();
  return name.length > 0 ? name : null;
}

function createRandomName(prefix) {
  return `${prefix} ${randomUUID().slice(0, 8)}`;
}

function normalizeKey(value) {
  return cleanName(value)?.toLowerCase().replace(/\s+/g, ' ') ?? null;
}

function numberOrNull(value) {
  if (value === null || value === undefined || value === '') return null;
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}

function degreesToRadians(degrees) {
  return (degrees * Math.PI) / 180;
}

function toPositiveIntegerString(value) {
  return Math.max(value, 1).toLocaleString('en-US', {
    maximumFractionDigits: 0,
    useGrouping: false,
  });
}

function toPositiveOrNegativeIntegerString(value) {
  return Math.round(value).toString();
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
