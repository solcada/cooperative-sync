# Options Paper: Cross-Platform Language for Client Daemon

## Decision to make
Choose the implementation language for the desktop backup daemon that runs on macOS, Linux, and Windows.

This daemon is responsible for:
- Watching filesystem changes.
- Managing local state and upload queue.
- Handling retries, backoff, and resume.
- Performing hashing/chunking and network transfer.
- Running continuously with low CPU/memory overhead.

## Evaluation criteria
- Cross-platform parity: mature support for macOS/Linux/Windows.
- Reliability: strong type/runtime behavior for long-running background services.
- Performance: efficient I/O, hashing, and concurrency.
- Distribution: straightforward static/small binaries and update process.
- Native integration: filesystem watchers, paths/permissions, OS service managers.
- Team velocity: learning curve and hiring pool.
- Security: memory safety and low vulnerability surface.

## Option A: Rust

### Pros
- High performance with low memory overhead.
- Strong safety model (memory-safe by default, fewer runtime crashes).
- Excellent fit for daemon-style systems programming.
- Good cross-platform crates for:
  - File watching (`notify`)
  - SQLite (`rusqlite`, `sqlx`)
  - Async networking (`tokio`, `reqwest`)
  - Crypto/hashing (`ring`, `sha2`, `blake3`)
- Produces single native binaries per platform.

### Cons
- Higher learning curve.
- Slower initial feature velocity if team is new to Rust.
- Longer compile times.

### Risk profile
- Lower long-term runtime risk.
- Medium short-term delivery risk if Rust experience is limited.

## Option B: Go

### Pros
- Very fast development for backend/daemon workflows.
- Great standard library and concurrency model.
- Cross-compilation and distribution are simple.
- Large hiring pool and operational familiarity.

### Cons
- Garbage collector can introduce latency spikes under some workloads.
- Less strict correctness guarantees than Rust for complex state machines.
- Filesystem watcher ecosystem is functional but not as robustly typed/structured as Rust patterns.

### Risk profile
- Low short-term delivery risk.
- Medium long-term correctness/perf tuning risk for heavy client workloads.

## Option C: C# (.NET)

### Pros
- Strong tooling and mature runtime.
- Good Windows integration.
- Reasonable cross-platform support via .NET 8+.

### Cons
- Heavier runtime footprint vs Rust/Go native binaries.
- Linux/macOS daemon packaging story is workable but less lightweight.
- Native low-level filesystem nuances can become cumbersome.

### Risk profile
- Good enterprise maintainability.
- Medium product-fit risk for lean, always-on client daemon.

## Option D: C++

### Pros
- Maximum control and performance.
- Broad native API access.

### Cons
- Highest complexity and maintenance burden.
- Memory safety risks are materially higher.
- Slower team velocity and larger testing burden.

### Risk profile
- High implementation and security risk unless team is deeply specialized.

## Option E: Node.js/TypeScript

### Pros
- High developer velocity.
- Easy shared code with JS-based UIs.

### Cons
- Heavier runtime for background daemon.
- Less suitable for low-level file/event handling at scale.
- Packaging and service behavior can be brittle across OSes.

### Risk profile
- Good for UI/control-plane clients.
- High risk for core sync daemon correctness/performance.

## Summary comparison

| Criterion | Rust | Go | C# | C++ | Node/TS |
|---|---|---|---|---|---|
| Runtime efficiency | Excellent | Very good | Good | Excellent | Fair |
| Safety | Excellent | Good | Good | Fair | Good |
| Cross-platform daemon fit | Excellent | Very good | Good | Very good | Fair |
| Team velocity (typical) | Medium | High | High | Low | High |
| Long-term maintenance risk | Low | Medium | Medium | High | Medium-High |
| Packaging footprint | Small | Small | Medium | Small | Medium-Large |

## Recommendation
Use **Rust** for the core client daemon.

Why:
- Best balance of safety, performance, and long-running daemon reliability.
- Strong alignment with file-watching + sync-engine workload.
- Lower long-term production risk for data integrity and recovery correctness.

## Practical delivery plan

### Phase 1 (4-8 weeks): Rust MVP daemon
- Folder watch, local SQLite index, upload queue.
- Basic upload/retry/resume.
- Service mode on all three OSes.

### Phase 2: Hardening
- Conflict/tombstone edge cases.
- Telemetry/diagnostics bundle.
- Installer + auto-update integration.

### Phase 3: Performance and reliability
- Adaptive batching/chunking.
- Backpressure controls.
- Chaos and soak testing across OS matrix.

## Contingency option
If timeline pressure is extreme and Rust skills are not yet in place:
- Start with **Go** for rapid MVP.
- Keep protocol, state machine, and DB schema language-agnostic.
- Plan targeted Rust migration for critical paths only if needed.

## Final call
- Default choice: **Rust now**.
- Fallback choice: **Go now, with architecture preserving optional Rust migration**.
