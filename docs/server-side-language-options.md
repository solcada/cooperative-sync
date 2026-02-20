# Options Paper: Server-Side Language Selection

## Decision to make
Choose the primary language for the cloud backend that powers v1 one-way backup, restore workflows, auth, metadata APIs, and background jobs.

## Context from current product direction
- v1 is cloud-first backup (client uploads to server; no bi-directional sync yet).
- Core backend responsibilities are API handling, metadata persistence, restore job orchestration, and reliable async workers.
- Early success depends on shipping quickly without creating long-term reliability debt.

## Evaluation criteria
- Delivery speed: ability to ship v1 quickly with a small team.
- Reliability: predictable behavior under retries, partial failures, and backpressure.
- Performance: efficient API and worker throughput without large infra spend.
- Operability: observability, profiling, and straightforward on-call debugging.
- Hiring and maintainability: ease of staffing and long-term ownership.
- Ecosystem fit: mature libraries for PostgreSQL/Redis/queues/object storage/auth.

## Option A: Go

### Pros
- High delivery velocity for APIs and background workers.
- Strong standard library and concurrency model for I/O-heavy services.
- Simple deployment model (single binaries, lightweight containers).
- Mature ecosystem for HTTP, SQL, tracing, and cloud SDKs.
- Good operational ergonomics for small infra teams.

### Cons
- Less compile-time strictness than Rust for complex state machines.
- GC tuning may be needed at very high throughput.

### Risk profile
- Low short-term delivery risk.
- Medium long-term risk if domain invariants are not enforced with strong testing.

## Option B: Rust

### Pros
- Excellent performance and strong memory safety guarantees.
- Strong fit for high-integrity services and resource-efficient workers.
- Good async and networking ecosystem.

### Cons
- Higher implementation complexity and onboarding cost.
- Slower initial feature velocity for teams not already Rust-heavy.

### Risk profile
- Medium short-term schedule risk.
- Low long-term runtime and correctness risk.

## Option C: Java/Kotlin (JVM)

### Pros
- Very mature server ecosystem, tooling, and frameworks.
- Strong observability and enterprise-grade libraries.
- Kotlin improves ergonomics while keeping JVM strengths.

### Cons
- Heavier runtime and container footprint.
- More framework overhead than needed for a lean v1.

### Risk profile
- Low technical risk, medium product-fit risk for a small fast-moving team.

## Option D: C# (.NET)

### Pros
- Excellent developer experience and strong web framework support.
- Good performance in modern .NET versions.
- Strong type system and tooling quality.

### Cons
- Slightly heavier operational footprint than Go for minimal services.
- Team familiarity often varies more across startup hiring pools.

### Risk profile
- Low technical risk, medium execution risk depending on team mix.

## Option E: Node.js/TypeScript

### Pros
- Very fast initial product iteration.
- Shared language with frontend code.
- Large package ecosystem.

### Cons
- Higher runtime overhead for worker-heavy backends.
- More frequent dependency/security churn.
- Can become harder to control correctness in distributed job logic at scale.

### Risk profile
- Low early execution risk, medium-high reliability/maintenance risk as complexity grows.

## Summary comparison

| Criterion | Go | Rust | Java/Kotlin | C#/.NET | Node/TS |
|---|---|---|---|---|---|
| v1 delivery speed | Excellent | Medium | Good | Good | Excellent |
| Runtime efficiency | Very good | Excellent | Good | Good | Fair |
| Reliability for async workers | Very good | Excellent | Very good | Very good | Good |
| Operability (small team) | Excellent | Good | Good | Good | Good |
| Hiring flexibility | High | Medium | High | Medium | High |
| Long-term maintenance risk | Medium-Low | Low | Medium | Medium | Medium-High |

## Recommendation
Use **Go** as the primary server-side language for v1.

### Why
- Best balance of speed to market and operational simplicity.
- Strong fit for the actual workload: API + async job orchestration + storage integration.
- Lower staffing risk than Rust while still giving good performance and reliability.

## Implementation strategy

### Default stack
- Language/runtime: Go 1.23+.
- API: REST/JSON over `net/http` (or a minimal router).
- Storage: PostgreSQL for metadata, Redis for ephemeral coordination, object storage for file blobs.
- Async: queue-based workers with explicit idempotency keys and retry policies.
- Observability: OpenTelemetry traces/metrics + structured logs.

### Guardrails for correctness
- Enforce idempotency for upload and restore job handlers.
- Use state-machine tests for restore lifecycle transitions.
- Add contract tests around object-storage operations and partial-failure recovery.

### Migration posture
- Keep service boundaries and wire protocols language-neutral.
- If a hotspot emerges (CPU-heavy chunking/dedup), isolate and rewrite that component in Rust without replatforming the whole backend.

## Final call
- Default choice: **Go now**.
- Secondary choice: **Rust** if the team already has strong Rust backend experience and can absorb slower MVP velocity.
