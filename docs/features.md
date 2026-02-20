# Product Features

## Supported Features
- `docs/client-sync.md`
- `docs/website-sync-details.md`

## v1 Service: Simple Backup (One-Way Sync)

### Summary
- User selects a local folder.
- Client continuously uploads changes to the server.
- No server-to-client sync in v1.
- Traffic model is ingress-heavy (customer uploads), enabling lowest-cost tier.

### Why this is first
- Smallest implementation surface area.
- Clear customer value: disaster recovery.
- Predictable infrastructure cost profile.

### In Scope (v1)
- Folder selection and onboarding on macOS, Linux, Windows.
- Continuous upload for create/update/delete.
- Retention window for deleted/previous versions (policy TBD).
- Restore UI to recover files after device failure.

### Out of Scope (v1)
- Bi-directional sync between devices.
- Real-time collaboration semantics.
- LAN peer-to-peer transfer.

## Failure Recovery: How users get files back

### Recovery paths we should support
- Fast path: download selected files/folders from web restore UI.
- Full-machine path: request bulk restore artifact(s) when dataset is large.
- Time-based restore: recover state from a selected point in time.

### v1 restore UX requirements
- Web console with file browser and search.
- Point-in-time selector (snapshot date/time).
- Multi-select restore request + progress/status page.
- Email/in-app notification when restore is ready.

## Backblaze reference model (verified February 20, 2026)

Backblaze Computer Backup currently exposes three restore methods:
- Download (free): ZIP-based restore, limited to 500 GB per request and up to 20 active ZIP requests.
- Save Files to B2: sends restore data to B2 storage (up to 10 TB restore size).
- USB hard drive restore: shipped restore media, up to 8 TB.

Backblaze also runs a return/refund model for restore drives:
- Drive can be returned within 30 days for refund.
- Limit of five refunded returned drives per account per 12 months.
- Customer pays return shipping.

## Recommended approach for our v1
- Start with web download restores for small/medium recovery jobs.
- Add "bulk restore job" packaging for large restores (split into multiple archives if needed).
- Defer shipped-physical-drive restore until post-v1, but design restore pipeline so this can be added later.
- Keep restore as simple as backup: folder tree, point-in-time pick, one-click request, clear status.

## Open product decisions
- Max restore archive size per request.
- Max concurrent restore jobs per user.
- Default retention period and pricing tiers.
- Whether to offer a "restore-by-mail" add-on in v1.1/v2.
