# Salimon

Salimon is an npm-workspaces monorepo orchestrated by Turborepo.

## Game

**Salimon: Echoes of Absenat** is a deep-space survival journey in which humanity uses a mysterious energy cube, known as the Core, to produce oxygen and electricity while travelling toward Absenat.

See [Game Vision](docs/game-vision.md) for story, canon, and high-level gameplay requirements. See [Communications](docs/communications.md) for communication-related behavior.

## Project Structure

```text
.
├── apps
│   ├── client       # React, Vite, Phaser, Jotai, Socket.IO client
│   └── core         # Express, Socket.IO, Mongoose backend
├── docs             # Product and feature documentation
├── packages
│   ├── sandbox      # Simulation sandbox utilities and tests
│   ├── types        # Shared TypeScript contracts
│   └── world        # Shared world-domain services
├── AGENTS.md        # AI agent project guide and architecture rules
├── package.json
└── turbo.json
```

## Applications

- `apps/client`: React 19, TypeScript, Vite, Phaser, Jotai, Axios, and Socket.IO client.
- `apps/core`: Express 5, Socket.IO, Mongoose/Typegoose, and OpenAI-powered backend.

## Packages

- `@repo/types`: shared DTOs and domain contracts.
- `@repo/world`: shared world-domain calculations and services.
- `@repo/sandbox`: simulation sandbox utilities with Vitest coverage.

## Development

Install dependencies:

```sh
npm install
```

Start all development tasks:

```sh
npm run dev
```

Start a single workspace:

```sh
npm run dev --workspace client
npm run dev --workspace core
```

Preview built apps:

```sh
npm run start:client
npm run start:core
```

Run checks:

```sh
npm run build
npm run check-types
npm run lint
npm run test
```

Only `@repo/sandbox` currently defines package-level tests.

## World Data

The backend owns world-data scripts:

```sh
npm run fetch-world-data
npm run seed-solar-system
```

## Environment

Development and start tasks may use:

- `MONGODB_URI`
- `OPENAI_API_KEY`
- `OPENAI_MODEL`
- `PORT`

Keep local secrets in untracked `.env` files.

## Architecture Direction

The client is planned to migrate toward Feature-Sliced Design (FSD). Future client work should prefer this target structure when adding new code:

```text
apps/client/src
├── app
├── pages
├── widgets
├── features
├── entities
├── shared
└── main.tsx
```

Follow the dependency direction:

```text
app -> pages -> widgets -> features -> entities -> shared
```

See [AGENTS.md](AGENTS.md) for the detailed AI-agent rules, current source layout, FSD migration plan, backend route/service conventions, and documentation standards.
