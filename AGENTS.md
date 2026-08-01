# Salimon Agent Guide

This file is the source of truth for AI agents working in this repository. Read it before changing code. Keep it aligned with `README.md` whenever project structure, commands, or conventions change.

## Project Overview

Salimon is an npm-workspaces monorepo orchestrated by Turborepo.

The product is **Salimon: Echoes of Absenat**, a deep-space survival game where humanity travels toward Absenat while relying on the Core for oxygen and electricity. See `docs/game-vision.md` for story, canon, and gameplay direction, and `docs/communications.md` for communication-related behavior.

## Workspace Structure

```text
.
├── apps
│   ├── client       # React 19, Vite, TypeScript, Phaser, Jotai, Socket.IO client
│   └── core         # Express, Socket.IO, Mongoose, OpenAI-powered backend
├── docs             # Product and feature documentation
├── packages
│   ├── sandbox      # Simulation sandbox utilities and Vitest tests
│   ├── types        # Shared TypeScript DTOs and domain contracts
│   └── world        # Shared world-domain calculations/services
├── package.json     # Root workspace scripts
└── turbo.json       # Turborepo task pipeline
```

Do not document or build against removed workspaces. `apps/lambda` is not part of the current workspace layout; the backend app is `apps/core`.

## Current Source Layout

### `apps/client`

Current client layout:

```text
apps/client/src
├── app          # Top-level React app composition and app styles
├── assets       # Client-bundled static assets
├── components   # Current UI feature/component folders
├── store        # Jotai atoms, derived state, bootstrap, and worker state
├── utils        # General client utilities
├── index.css
└── main.tsx
```

Important current conventions:

- Use TypeScript, React function components, ES modules, and CSS modules where local component styles already use them.
- Keep public exports in directory-level `index.ts` files when a module is intended for use outside its folder.
- Use existing path aliases from `apps/client/tsconfig.app.json` until the FSD migration updates them.
- Every Jotai atom in `apps/client/src/store` must expose both a getter hook and setter hook. Name them `use<Name>` and `useSet<Name>`, implemented with `useAtomValue` and `useSetAtom`.

Example:

```ts
export function usePlanets() {
  return useAtomValue(planetsAtom);
}

export function useSetPlanets() {
  return useSetAtom(planetsAtom);
}
```

### `apps/core`

Current backend layout:

```text
apps/core/src
├── app.ts        # Express app composition
├── http.ts       # HTTP server setup
├── index.ts      # Backend entry point
├── middleware.ts # Shared Express middleware
├── models        # Typegoose/Mongoose models
├── routes        # REST route groups and handlers
├── services      # Backend domain/application services
└── socket.ts     # Socket.IO setup
```

Backend conventions:

- REST APIs live under nested route folders in `apps/core/src/routes`.
- Use a `routes.ts` file for each route group. It defines the Express router and registers paths.
- Put each route handler in its own sibling file.
- Mirror URL nesting in the filesystem. For example, `/a/b/c` should live under `routes/a/b/routes.ts`, with that `routes.ts` registering `/c`.
- Export route groups from `apps/core/src/routes/index.ts` via their `routes.ts` files, for example `export * from './world/routes';`.
- Services may be single-file services such as `metrics.service.ts` or folder services such as `spaceship.service/index.ts`. Preserve the local pattern around the service being changed.
- For single-file services, use `name.service.ts` and export a static class named `NameService`.
- For folder services, export the service surface from `index.ts` and keep focused helper files inside the service folder.

### `packages`

- `@repo/types`: shared DTOs and contracts. Put cross-workspace types here instead of duplicating them in apps.
- `@repo/world`: reusable world-domain logic used by apps.
- `@repo/sandbox`: sandbox/simulation utilities. This package currently owns the configured Vitest tests.

Prefer shared packages when code is genuinely reused by more than one workspace. Keep app-specific behavior inside the app.

## Planned Client FSD Structure

The client should migrate toward Feature-Sliced Design (FSD) over time. Do not perform a large mechanical migration unless the task explicitly asks for it. For normal feature work, place new or substantially touched client code in the closest FSD layer and migrate nearby code only when it reduces churn.

Target layout:

```text
apps/client/src
├── app
│   ├── providers     # React providers, global app wiring
│   ├── routes        # Route/view composition if routing is introduced
│   ├── styles        # Global app styles
│   └── index.tsx
├── pages             # Full screen/page composition
├── widgets           # Large UI blocks composed from features/entities
├── features          # User actions and workflows
├── entities          # Business entities such as spaceship, world, contact
├── shared
│   ├── api           # HTTP/socket clients and transport helpers
│   ├── assets        # Shared assets
│   ├── config        # Client config/constants
│   ├── lib           # Generic utilities
│   ├── model         # Shared client state primitives
│   └── ui            # Reusable generic UI components
└── main.tsx
```

FSD dependency rule:

```text
app -> pages -> widgets -> features -> entities -> shared
```

A layer may import only from layers below it. `shared` imports from no upper layer. Avoid lateral imports between slices unless using a public API from an allowed lower layer.

Slice structure:

```text
features/send-message
├── model      # State, atoms, selectors, schema, business state transitions
├── api        # Feature-specific API calls when not generic enough for shared/api
├── ui         # Components owned by the feature
├── lib        # Feature-local helpers
└── index.ts   # Public exports for other layers
```

FSD migration rules:

- Use kebab-case folder names for slices: `send-message`, `world-map`, `spaceship-status`.
- Import a slice through its public `index.ts` unless working inside that slice.
- Keep generic UI in `shared/ui`; keep domain-specific UI in `entities`, `features`, or `widgets`.
- Keep reusable transport clients in `shared/api`; keep endpoint calls near the feature/entity that owns the behavior.
- Keep Jotai atoms in the owning slice's `model` folder after migration. Continue exporting getter and setter hooks for every atom.
- Avoid dumping new code into legacy `components`, `store`, or `utils` if an FSD home is clear.
- While legacy folders exist, do not mix both directions in one change. New FSD code may consume legacy modules, but legacy modules should not start importing from new FSD slices unless part of a deliberate migration.

Suggested first migration steps:

1. Create `shared/lib` and move generic utilities from `utils` there.
2. Create `shared/ui` for reusable visual primitives.
3. Move world-related state and worker code from `store` into `entities/world/model`.
4. Move communications UI/state into `features` or `widgets` based on ownership.
5. Move navigator UI into `widgets/navigator` and entity-specific pieces into `entities/world`.
6. Update TypeScript path aliases after the first real FSD folders exist.

## Commands

Install dependencies:

```sh
npm install
```

Run all workspace dev tasks:

```sh
npm run dev
```

Run one app:

```sh
npm run dev --workspace client
npm run dev --workspace core
```

Preview built apps:

```sh
npm run start:client
npm run start:core
```

Build, type-check, lint, and format:

```sh
npm run build
npm run check-types
npm run lint
npm run format
```

Tests:

```sh
npm run test
npm run test --workspace @repo/sandbox
```

Only `@repo/sandbox` currently defines a package-level `test` script. Do not claim client or core tests exist unless they are added to those package manifests.

World data scripts:

```sh
npm run fetch-world-data
npm run seed-solar-system
```

## Environment

`turbo.json` passes these variables to `dev` and `start` tasks:

- `MONGODB_URI`
- `OPENAI_API_KEY`
- `OPENAI_MODEL`
- `PORT`

Do not hard-code secrets. Prefer `.env` files for local configuration and keep them out of version control.

## Coding Standards

- Preserve the existing TypeScript and ES module style in each workspace.
- Respect each package's module system. `apps/client` is ESM; `apps/core` and shared packages currently use CommonJS package settings.
- Do not introduce unused locals, unused parameters, or switch fallthrough. The TypeScript configs reject them.
- Use structured APIs and parsers over ad hoc string manipulation when reasonable.
- Keep changes scoped to the requested behavior. Avoid unrelated refactors.
- Add abstractions only when they remove real duplication or match an established pattern.
- Run Prettier instead of manually aligning code.
- For frontend work, preserve existing CSS module patterns unless migrating a slice intentionally.
- For shared contracts, update `@repo/types` first and then update consumers.

## Documentation Standards

- Keep `README.md` useful for humans starting the project.
- Keep `AGENTS.md` precise for AI agents changing the project.
- When adding or moving a workspace, update both docs in the same change.
- When adding scripts, document only commands that actually exist in `package.json`.
- When changing architecture direction, document migration rules and import boundaries.

## Commit Guidelines

Use Conventional Commit style matching recent history:

```text
feat(client): add navigator panel
feat(core): add world metrics endpoint
refactor(client): move communications into FSD feature
docs: update project structure guide
```

Keep summaries concise and imperative. Use a scope when it clarifies the affected area.
