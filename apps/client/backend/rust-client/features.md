# Client Sync Features

## Scope
This document defines product-facing and engineering features for the Rust client sync component.

## Story Order (Current Implementation Track)
1. Sync a single file.
2. Sync a directory.
3. Sync a large file.
4. Handle failures and recovery.

## v0 Prototype (Current)

### Implemented
- CLI entrypoint with explicit commands:
  - `health --server <url>`
  - `sync --server <url> --path <path> --hash <hash>`
- HTTP health probe to sync server (`GET /v1/health`).
- HTTP sync enqueue call (`POST /v1/sync`).
- Error mapping for invalid URL, protocol, network, and server status failures.
- `SyncManager` queue with snapshot/restore for recovery testing.
- Self-documenting tests using in-process mocks (no external server needed).

### Assumed Contract (Mocked)
- Base URL format: `http://host:port`
- `GET /v1/health` returns 200 when healthy.
- `POST /v1/sync` accepts text payload:

```text
path=<relative-path>
hash=<content-hash>
```

- `200` or `202` are success for sync requests.

## Test Coverage Mapped to Stories

### Story 1: Single File Sync
- Queues one file and syncs successfully.
- Validates error when a requested file path is missing.

### Story 2: Directory Sync
- Recursively queues nested files.
- Processes all queued directory files.
- Handles partial failures and retries remaining files.

### Story 3: Large File Sync
- Queues and syncs large test payloads.
- Uses streaming hash computation (chunked reads).

### Story 4: Failure + Recovery
- Retains failed sync items in queue.
- Retries on later flush rounds.
- Restores queue snapshot after simulated restart and completes sync.

## Edge Cases To Test (Documented and Tracked)

### File System Edge Cases
- File missing between detection and sync attempt.
- Permission denied while reading file.
- Symlink handling policy (follow vs ignore).
- Unicode/special characters in path names.
- Very long path names (platform constraints).
- File modified while hashing.
- File deleted mid-sync.

### Directory Traversal Edge Cases
- Empty directory.
- Deeply nested directory trees.
- Hidden/system files.
- Directory with mixed readable/unreadable files.
- Non-deterministic `read_dir` ordering effects.

### Large File Edge Cases
- Multi-GB file hashing performance.
- Memory pressure during large-file handling.
- Interrupted read stream.

### Transport and Protocol Edge Cases
- DNS/connection failures.
- Timeout on request/response.
- 4xx client errors (auth/config failures).
- 5xx server errors with retry.
- Malformed HTTP response.
- Unexpected response body/content-length mismatch.

### Queue and Recovery Edge Cases
- Process crash after enqueue but before flush.
- Crash during flush with partial success.
- Duplicate delivery after retry (idempotency requirement).
- Queue growth/backpressure handling under burst changes.

## Out of Scope for v1
- Bi-directional sync from cloud to local device.
- Multi-device conflict resolution UI.
- LAN peer-to-peer transfer.

## Acceptance Criteria (v1)
- Client recovers unsent changes after restart/crash.
- Small file sync success median under 5 seconds on stable network.
- Retries eventually succeed or surface clear terminal error state.
- Critical sync paths covered by automated tests.

## Open Decisions
- Queue persistence format (SQLite vs embedded log).
- Max batch size and flush interval defaults.
- Token refresh behavior during offline periods.
- Backpressure strategy for very large local change bursts.
