# Repository Guidelines

## Project Structure & Module Organization

This npm-workspaces monorepo is orchestrated by Turborepo. The active application is `apps/client`, a React 19, TypeScript, Vite, Phaser, and Jotai client. Keep UI modules in `apps/client/src/components`, application composition in `src/app`, shared domain types in `src/types`, state in `src/store`, and general helpers in `src/utils`. Public entry points such as each directory's `index.ts` should expose modules intended for use elsewhere. TypeScript path aliases are configured in `apps/client/tsconfig.app.json`.

## Build, Type-Check, and Development Commands

- `npm run dev` starts workspace development tasks through Turbo.
- `npm run build` builds all workspaces; the client runs `tsc -b` before Vite.
- `npm run check-types` runs TypeScript checks across workspaces.
- `npm run lint` runs each workspace's ESLint task.
- `npm run format` formats TypeScript, TSX, and Markdown with Prettier.
- `npm run dev --workspace client` starts only the Vite client.

No automated test command or test framework is currently configured. Do not document or invoke a test command unless one is added to `package.json`.

## Coding Style & State Conventions

Use TypeScript and preserve the existing ES module style. ESLint applies the recommended JavaScript, TypeScript, React Hooks, and Vite React Refresh rules. The client compiler rejects unused locals, unused parameters, and switch fallthrough. Run Prettier rather than manually aligning code.

Every Jotai atom in `apps/client/src/store` must have an exported getter hook and setter hook. Name them `use<Name>` and `useSet<Name>` and implement them with `useAtomValue` and `useSetAtom`:

```ts
export function usePlanets() {
  return useAtomValue(planetsAtom);
}

export function useSetPlanets() {
  return useSetAtom(planetsAtom);
}
```

## Commit Guidelines

Recent history generally uses Conventional Commit prefixes such as `feat(client):`, `feat(navigator):`, and `refactor:`. Follow that pattern with a concise, imperative summary and an optional scope describing the affected area.
