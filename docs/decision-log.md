# Decision Log

## DL-0001: Regression Test Topology (Client + Server)

- Date: 2026-02-20
- Status: Accepted

### Context

For regression testing, we need to validate the integrated behavior of the client daemon and server daemon. The open question is whether tests should:
- run each daemon in its own Docker container, or
- run both daemons on the same host process space using different ports.

### Decision

Use **two containers** for the canonical regression suite: one for client, one for server.

### Rationale

- Better production parity: independent process boundaries and network path.
- Stronger isolation: fewer hidden dependencies on host state.
- Cleaner CI reproducibility: the same compose topology runs locally and in CI.
- Easier fault simulation: network interruptions and restart behavior are more realistic.

### Consequences

- Regression setup is slightly heavier than same-host daemon tests.
- We should still keep a fast local smoke mode where both daemons run on the same box with different ports for quick iteration.
- Pass criteria for merge should rely on the two-container regression run.
