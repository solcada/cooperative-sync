# Rust Client (Prototype)

This is the first Rust-based client prototype for the sync platform.

## What this prototype does

- Checks sync server health (`GET /v1/health`)
- Sends a file sync event (`POST /v1/sync`)
- Adds a local sync manager with a retryable queue
- Supports queue snapshot/restore to simulate recovery after restart
- Uses only Rust standard library networking for now
- Includes tests with in-process mocks (no real server required)

## Assumptions (for mocking and early development)

Because the real sync server is not integrated yet, this client assumes:

1. The server is reachable over `http://` (no TLS in this prototype).
2. Health endpoint is `GET /v1/health`.
3. Sync endpoint is `POST /v1/sync`.
4. Sync request body is plain text:

```text
path=<relative-path>
hash=<content-hash>
```

5. Success responses:
- `200 OK` means accepted and processed immediately.
- `202 Accepted` means accepted for async processing.

6. Any other status from `/v1/sync` is treated as an error.

## Project layout

- `src/lib.rs`: core client, sync manager, and tests
- `src/main.rs`: CLI wrapper
- `features.md`: story order + edge-case checklist
- `Cargo.toml`: crate definition

## Where the sync server code is

- Server backend folder: `/Users/andrew/Desktop/cooperative-sync/apps/sync-server/backend`
- Current runnable server for development: `/Users/andrew/Desktop/cooperative-sync/apps/sync-server/backend/mock_sync_server.py`
- In addition to that mock server, unit tests also use in-process mocks in `src/lib.rs`.

## Run both (sync server + client)

Use two terminals.

Terminal 1, start mock sync server:

```bash
cd /Users/andrew/Desktop/cooperative-sync/apps/sync-server/backend
python3 mock_sync_server.py --host 127.0.0.1 --port 8080
```

Terminal 2, run Rust client against it:

```bash
cd /Users/andrew/Desktop/cooperative-sync/apps/client/backend/rust-client
cargo run -- health --server http://127.0.0.1:8080
cargo run -- sync --server http://127.0.0.1:8080 --path notes/todo.txt --hash abc123
```

To simulate failures for recovery testing, start server in fail mode:

```bash
cd /Users/andrew/Desktop/cooperative-sync/apps/sync-server/backend
python3 mock_sync_server.py --host 127.0.0.1 --port 8080 --fail-sync
```

## Prerequisites

- Rust toolchain (`cargo`, `rustc`) version 1.74+ recommended

## Install Rust by environment

### macOS

Using Homebrew + rustup:

```bash
brew install rustup-init
rustup-init
source "$HOME/.cargo/env"
```

Alternative official installer:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Linux

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

### Windows

1. Download and run `rustup-init.exe` from [https://rustup.rs](https://rustup.rs)
2. Reopen terminal (PowerShell or CMD)
3. Verify:

```powershell
cargo --version
rustc --version
```

## Build and run

From this directory:

```bash
cd /Users/andrew/Desktop/cooperative-sync/apps/client/backend/rust-client
cargo build
```

Run health check:

```bash
cargo run -- health --server http://127.0.0.1:8080
```

Run sync request:

```bash
cargo run -- sync --server http://127.0.0.1:8080 --path notes/todo.txt --hash abc123
```

## Run tests (no real server required)

```bash
cargo test
```

The tests are self-documenting and cover:

- Single-file sync story
- Directory sync story
- Large-file sync story
- Failure and recovery story (queue retry + snapshot/restore)
- HTTP protocol behavior with mock server responses

## Install binary locally

Install into Cargo's bin path:

```bash
cargo install --path .
```

Then run:

```bash
rust-client health --server http://127.0.0.1:8080
```

## Next steps

- Replace plain-text payload with JSON + versioned schema
- Add TLS and token auth
- Add persistent on-disk queue
- Add integration tests against the real sync server once available
