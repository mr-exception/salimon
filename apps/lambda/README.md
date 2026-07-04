# Lambda

AWS Lambda backend for Salimon. Each HTTP endpoint is implemented by a
dedicated handler under `src/functions`.

`npm run build --workspace lambda` produces deployable archives for the
systems API and the planet, moon, and star position updaters. Each archive has
its deployable handler as `index.js` at its root. esbuild bundles the handlers
and their runtime npm dependencies into `index.js`; development-only and
type-only packages are not shipped.

`GET /world/systems` accepts a search center and radius in meters:

```text
/world/systems?x=0&y=0&radius=1000000
/world/systems?coordinate=0,0&radius=1000000
```

It returns systems whose star's resolved center coordinates are inside the
search circle. Each system contains its star, the planets that directly orbit
the star, and the moons that directly orbit each planet. Once a star is in
range, all of those orbiting bodies are returned regardless of their own
positions.

Coordinates and radii must be integers. Body coordinates may be relative to
another body; the handler resolves these references before filtering.

Player spaceships are private records addressed by a UUID v4 security code:

- `POST /spaceship/register` creates a spaceship at the default Earth surface
  position with zero speed.
- `GET /spaceship/info` lazily advances an offline spaceship to the current
  time before returning it.
- `PUT /spaceship/update` validates and replaces its position, direction, and
  speed, precise relative velocity, and motion state.

The info and update routes require the security code in the
`x-spaceship-security-code` header. The update body uses integer strings for
`position.x`, `position.y`, and `speed`; `direction` is a number in the range
from 0 (inclusive) to 360 (exclusive). `velocity.x` and `velocity.y` are finite
numbers in meters per second, and `motionState` is `flying`, `landed`, or
`crashed`.

Spaceships stored relative to a planet or star include a simulation timestamp.
On the next info request, the Lambda advances flying ships under the reference
body's gravity with bounded integration steps and swept collision detection.
Impacts above 15 m/s are stored as crashes; slower impacts are landings.
Landed and crashed surface positions rotate with the reference body. This
catch-up is performed only when the spaceship is requested, so offline players
do not require scheduled Lambda invocations.

Separate planet, moon, and star updaters run every five minutes. Each selects
at most 100 bodies of its type with the oldest `updatedAt` dates, calculates
elapsed time from the EventBridge Scheduler invocation time, and rotates each
position around its orbital center using its `speed` and `clockwise`
attributes. Planets are records whose orbital center exists in `stars`; moons
are records whose orbital center exists in `planets`. Updates include the
previously read `updatedAt` value as a concurrency check, preventing
overlapping invocations from applying the same movement twice.

## Local development

Install a current AWS SAM CLI with Node.js 22 runtime support and Docker, then
run:

```sh
npm run dev --workspace lambda
```

Pass the MongoDB URI, including its database name, when starting SAM:

```sh
sam local start-api \
  --template template.yaml \
  --parameter-overrides MongoDbUri="$MONGODB_URI"
```

SAM serves `GET /world/systems` at
`http://127.0.0.1:3000/world/systems` for backend development and verification.

## Deployment

The main-branch workflow packages and deploys `template.yaml` with AWS
CloudFormation. It requires these GitHub Actions repository variables:

- `AWS_DEPLOY_ROLE_ARN`: OIDC role assumed by the workflow.
- `AWS_REGION`: deployment region.
- `AWS_LAMBDA_ARTIFACTS_BUCKET`: S3 bucket used by CloudFormation packaging.
- `AWS_LAMBDA_STACK_NAME`: CloudFormation stack name.
- `VITE_API_BASE_URL`: deployed API base URL used when building the client.

The workflow also requires a GitHub Actions secret named `MONGODB_URI`. Store
the complete MongoDB URI in a secret rather than a repository variable because
it contains credentials. The URI must include the database name used by the
Lambda.

The deployment role needs access to the artifact bucket and permission to
deploy the stack's Lambda, API Gateway, IAM, and CloudFormation resources.
