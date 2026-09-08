# Core

Reserved for the future authoritative backend/runtime service. Phase 0 is
entirely client-side, so this directory intentionally contains documentation only.

Do not add a Cargo package, backend behavior, networking, or persistence here for
Phase 0. Portable client domain modules should remain separable so later phases
can reuse their source without making the client depend on a backend shell.
