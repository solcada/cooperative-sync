# Cross-Platform File Sync Design Document

## Overview
This document defines the architecture and implementation plan for a product that synchronizes files across macOS, Linux, and Windows.

## Goals
- Near real-time file synchronization across supported platforms.
- Reliable conflict handling and recovery from network interruptions.
- Secure transport and storage of metadata.
- Usable CLI and desktop client experience.

## Non-Goals
- Full backup/versioning product in v1.
- Collaborative editing semantics in v1.

## Target Platforms
- macOS (latest two major versions)
- Linux (major distros)
- Windows 10/11

## High-Level Architecture
- Local file watcher/agent per device.
- Local metadata/index store.
- Sync engine with change detection and reconciliation.
- Cloud coordination service for metadata and transfer orchestration.
- Optional direct peer transfer when available.

## Core Components
### 1. File Watcher
- Platform-specific adapters:
  - macOS: FSEvents
  - Linux: inotify
  - Windows: ReadDirectoryChangesW
- Normalizes events to a shared internal format.

### 2. Local Index
- Tracks file paths, hashes, mtimes, inode/file IDs, and sync state.
- Persists checkpoint/cursor for crash-safe resume.

### 3. Sync Engine
- Batches local changes.
- Computes content hashes and chunk manifests.
- Uploads/downloads deltas.
- Reconciles remote and local state.

### 4. Conflict Resolution
- Detects divergent updates.
- Default policy: keep both copies with deterministic suffixing.
- User-configurable policies in future release.

### 5. Transport and Security
- TLS for all network communication.
- Auth via short-lived tokens.
- Optional end-to-end encryption roadmap.

### 6. Service APIs
- Device registration and auth.
- Change feed and checkpoint APIs.
- Chunk upload/download APIs.
- Health and diagnostics endpoints.

## Sync Model
- Logical clock/version vector per file.
- At-least-once event processing with idempotent operations.
- Eventual consistency target with bounded convergence time.

## Data Model (Draft)
- `FileRecord`: file_id, path, size, hash, mtime, version, tombstone
- `ChunkRecord`: chunk_id, file_id, offset, length, hash
- `DeviceRecord`: device_id, platform, last_seen, capabilities

## Failure Modes
- Network loss: queue and retry with exponential backoff.
- Partial transfer: resume using chunk checkpoints.
- Crashes: restore from local index checkpoint.
- Clock skew: avoid time-only conflict checks; prefer version/hash.

## Performance Targets (Initial)
- Detect local file changes within 1s on idle systems.
- Propagate small-file updates within 5s median on stable networks.
- Handle repositories with 100k+ files without UI lockups.

## Observability
- Structured logs with correlation IDs.
- Metrics: queue depth, sync latency, retry rates, conflict rates.
- Diagnostic bundle export for support.

## Testing Strategy
- Cross-platform integration tests for watcher behavior.
- Property tests for reconciliation logic.
- Chaos tests for retries, packet loss, and process restarts.

## Open Questions
- Should v1 support LAN peer-to-peer transfer?
- What are storage/provider requirements for backend deployment?
- Is selective sync required in v1?
- What level of offline history should be retained locally?

## Milestones (Draft)
1. Local watcher + index prototype on all platforms.
2. Basic cloud sync for create/update/delete.
3. Conflict handling + retries + checkpointing.
4. Security hardening + observability + beta rollout.
