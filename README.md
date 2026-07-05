# Salimon

Salimon is an npm-workspaces monorepo orchestrated by Turborepo.

## Game

Salimon: Echoes of Absenat is a deep-space survival journey in which humanity
uses a mysterious energy cube, known as the Core, to produce oxygen and
electricity while travelling toward Absenat.

See [Game Vision](docs/game-vision.md) for the story, canon, and high-level
gameplay requirements.

## Applications

- `apps/client`: React, Vite, and Phaser client.
- `apps/lambda`: AWS Lambda HTTP API.

## Development

Install dependencies:

```sh
npm install
```

Start the Lambda API with the AWS SAM CLI and Docker:

```sh
npm run dev --workspace lambda
```

## Checks

```sh
npm run build
npm run check-types
npm run lint
```

See `apps/lambda/README.md` for backend deployment configuration.
