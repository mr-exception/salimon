# Lambda

AWS Lambda backend for Salimon. Each HTTP endpoint is implemented by a
dedicated handler under `src/functions`.

`npm run build --workspace lambda` produces `dist/world.zip`. The archive has
the deployable handler as `index.js` at its root together with `world.json`.

## Local development

Install a current AWS SAM CLI with Node.js 22 runtime support and Docker, then
run:

```sh
npm run dev --workspace lambda
```

SAM serves `GET /world` at `http://127.0.0.1:3000/world` for backend
development and verification.

## Deployment

The main-branch workflow packages and deploys `template.yaml` with AWS
CloudFormation. It requires these GitHub Actions repository variables:

- `AWS_DEPLOY_ROLE_ARN`: OIDC role assumed by the workflow.
- `AWS_REGION`: deployment region.
- `AWS_LAMBDA_ARTIFACTS_BUCKET`: S3 bucket used by CloudFormation packaging.
- `AWS_LAMBDA_STACK_NAME`: CloudFormation stack name.
- `CLIENT_ORIGIN`: deployed client origin allowed by API CORS.

The deployment role needs access to the artifact bucket and permission to
deploy the stack's Lambda, API Gateway, IAM, and CloudFormation resources.
