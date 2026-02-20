# Cooperative Sync

Cooperative Sync is a monorepo for a cross-platform client and sync server.

## Supported Features

- `docs/client-sync.md` (client backup/sync behavior)
- `docs/website-sync-details.md` (customer website sync details and account payments UI)

## Development SDLC

Use this lifecycle for each feature:

1. Define feature scope and acceptance criteria in a feature doc.
2. Record architectural and test-topology decisions in `docs/decision-log.md`.
3. Break work into components (client, server, shared contracts) and implement behind clear interfaces.
4. Add or update unit tests for each component touched by the change.
5. Add or update regression coverage that runs client and server together in Docker.
6. Run local checks (unit + regression), then open PR with test evidence and rollout notes.
7. Merge only after review confirms behavior, test coverage, and no regressions.

## Test Strategy

- Unit tests: validate each component in isolation (client and server).
- Regression tests: run client and server together to verify end-to-end sync behavior.
- Regression environment: Docker Compose with one container per daemon (client, server).
