---
name: generate-issue
description: Investigate a user-described bug, feature, refactor, or maintenance task in the Salimon codebase, clarify missing requirements, draft an implementation-ready GitHub issue, and create it in mr-exception/salimon. Use when the user asks to generate, draft, file, open, or create a repository issue from a task description.
---

# Generate Issue

Turn a task description into a grounded, actionable issue for `mr-exception/salimon`.
Inspect the repository before asking questions or writing the issue.

## Workflow

1. Parse the task description into the observed problem or desired outcome.
2. Inspect the working tree and repository guidance:
   - Read the applicable `AGENTS.md` files.
   - Locate relevant code with `rg` and `rg --files`.
   - Trace the current behavior, affected modules, state, types, and public exports.
   - Read relevant configuration, workflows, and package scripts when applicable.
   - Do not modify source files; this workflow investigates and files an issue only.
3. Inspect open issues in `mr-exception/salimon` for likely duplicates. Prefer the
   connected GitHub tools; use `gh` when connector coverage is unavailable.
4. Separate verified facts from assumptions. Ask the user only for information that
   cannot be derived from the codebase and would materially change scope or acceptance
   criteria. Group the required questions into one concise message where practical.
   Common examples are:
   - Expected behavior when multiple valid product choices exist.
   - Reproduction details or environment data absent from the repository.
   - Explicit scope boundaries with significantly different implementation costs.
   Do not ask for title wording, code locations, implementation details, labels,
   assignees, or milestones when they can be inferred or safely omitted.
5. If a likely duplicate exists, show the matching issue and ask whether to update the
   existing issue or create a distinct one. Do not create a duplicate silently.
6. Draft the issue using this structure, omitting sections that add no value:

```markdown
## Summary

<What should change and why>

## Current behavior

<What the code currently does, including reproduction steps for bugs>

## Expected behavior

<Observable outcome>

## Scope

- <Concrete in-scope work>

## Acceptance criteria

- [ ] <Verifiable outcome>

## Technical context

- `<path>`: <relevant current behavior or constraint>
```

7. Choose a concise imperative title. Keep acceptance criteria observable and avoid
   prescribing an implementation unless the existing architecture makes it a firm
   constraint. Never include secrets, local absolute paths, unsupported claims, or
   private conversation details.
8. Create the issue in `mr-exception/salimon` once all required answers are available.
   The user's invocation authorizes this issue creation; do not require a redundant
   confirmation unless the user requested a draft only. Apply labels only when their
   exact repository labels and applicability are verified. Do not assign people or set
   milestones without explicit user direction.
9. Return the created issue title and URL. Briefly note any intentionally omitted
   metadata or remaining assumptions.

## Failure handling

- If GitHub authentication or issue-creation tools are unavailable, return the final
  title and Markdown body ready to paste, plus the exact blocker.
- If the task remains materially ambiguous after clarification, do not invent product
  requirements or create a misleading issue.
- If repository evidence contradicts the initial task description, explain the
  discrepancy and clarify before creating the issue.
