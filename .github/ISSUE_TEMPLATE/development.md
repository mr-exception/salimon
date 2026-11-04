---
name: Development task or bug
about: Observable acceptance and verification for a feature, fix or documentation task
title: ''
labels: auto-verification
---

## Problem and expected result

Describe the trigger, current behavior and observable result. Link approved design
and explicitly identify any superseded requirement. For bugs include reproduction steps.

## Scope and prerequisites

List included/excluded scope, affected production owners/guides, formal dependencies
and textual prerequisites. Verify required code, not only closed issue status.
Identify the actual PR stack parent when implementation starts.

## Acceptance criteria

- AC-1: Observable result, including relevant boundaries/failure behavior.

## Verification matrix

Follow [the policy](https://github.com/mr-exception/salimon/blob/main/docs/acceptance-policy.md). Fill every category; use N/A
with a reason. Mark mandatory checks before implementation; never mark an unrun check PASS.

| Category | Criterion IDs / production path | Exact command/procedure and expected assertion | Mandatory? / N/A reason | Evidence and actual result |
| --- | --- | --- | --- | --- |
| Unit | | | | NOT RUN |
| Integration | | | | NOT RUN |
| E2E | | | | NOT RUN |
| Visual | | | | NOT RUN |
| Manual/hardware qualification | | | | NOT RUN |

## Evidence and delivery

Record tested SHA, final PR head/base and parent, environment/backend/input,
viewport/DPR/seed/clock as applicable, exact commands/results and original
screenshots/traces/raw logs in `reports/issue-N/README.md`. Retain failures and
unverified limits. Link separately owned release qualification with exact scope.

Status: implementation in progress / QA pending / changes requested / QA accepted
/ merged / closed. Record independent reviewer identity/capability, cycle, verdict
and criterion evidence. Keep open until delivering PR merges with mandatory
acceptance satisfied; use `Closes #N` in that PR from creation.
