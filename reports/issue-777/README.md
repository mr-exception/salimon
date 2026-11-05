# Issue #777 — containerize the browser client

## Outcome

Added a production multi-stage Docker image for the existing React/Vite client,
a root `docker-compose.yml`, an Nginx SPA/static-asset configuration, a
small `.dockerignore`, Docker usage instructions in the root README, and a
dedicated GitHub Actions Compose/HTTP smoke workflow.

No client runtime, lockfile, game, or visual assets changed. No backend or
deployment has been configured.

## Revisions and environment

- Repository: `mr-exception/salimon`.
- Parent: PR #776, branch `issue-766-cockpit-handoff`, commit
  `a3a36ec899e5608d65cda7e848d137a6dd5bd2b5`.
- Implemented source commit: `36e7d2a9d0b319e24153355218e08207ebbbff63`. This report is a subsequent
  documentation-only commit; implementation/configuration inputs are unchanged.
- Delivery branch: `issue-777-docker-client`, targeting the parent branch.
- Local verification environment: connector-backed repository editing and
  Linux shell with Node 22.16 and Git. Docker CLI/daemon are **unavailable**;
  the shell also has no external GitHub network access.
- GitHub Actions workflow: `.github/workflows/docker-client.yml`, Ubuntu 24.04
  runner (pending first PR run).

## Criterion evidence

| Criterion / category | Verification | Status |
| --- | --- | --- |
| AC-1 / integration | `docker compose -f docker-compose.yml config --quiet` then `docker compose up --build -d --wait --wait-timeout 90` on the GitHub Actions Ubuntu runner | NOT RUN at report creation; mandatory CI |
| AC-2 / E2E | `curl` root page and hashed JS, compare fallback route with index, demand 404 for missing asset | NOT RUN at report creation; mandatory CI |
| AC-3 / documentation | Root README shows Docker prerequisites, start/rebuild, alternate host port, logs, stop, production-vs-dev behavior | PASS, source static review |
| AC-4 / unit/static | Eight repository-source assertions for locked build, image stages, Compose paths/port, Nginx fallback/404, ignore rules, README and CI coverage | PASS, 8/8 at `36e7d2a9d0b319e24153355218e08207ebbbff63` |
| Visual | No client rendering, source, style or baseline modified | N/A; existing rendered app remains unchanged |
| Manual/hardware | Local Docker execution unavailable; no physical GPU/input claim made | NOT RUN locally; CI is required integration evidence |

The static verification compared
`a3a36ec899e5608d65cda7e848d137a6dd5bd2b5...36e7d2a9d0b319e24153355218e08207ebbbff63`
and found only `.dockerignore`, the smoke workflow, root README,
`docker-compose.yml`, `docker/Dockerfile`, and `docker/nginx.conf`.
The Docker integration and HTTP behavior are **not** claimed passing from static
inspection. Review current-head CI before acceptance; any failure must be fixed.

## Active-session review

Pending actual CI results and review of the published PR's unique diff.
The implementing assistant will make a same-session evidence-based review;
no independent agent, visual-baseline approval or merge is implied.

## Limitations and follow-up

- The container is a static Nginx server. It is not a Vite development server
  and has no hot reload.
- Default exposed URL: `http://localhost:8080`; change host port with
  `SALIMON_PORT`.
- The parent PR #776 must merge before this stacked child; it remains open for
  human review. Do not deploy or merge automatically.
