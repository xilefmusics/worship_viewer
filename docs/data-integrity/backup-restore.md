# Backup, restore, and rollback

Worship Viewer persistence spans **SurrealDB** (metadata, ACL, audit), the **`BLOB_DIR`** filesystem tree (image/blob uploads), and the **`MEDIA_FINAL_DIR`** filesystem tree (uploaded media). Treat all three as a unit when planning disaster recovery. `MEDIA_STAGING_DIR` contains temporary uploads and does not need to be backed up.

## What to back up

| Asset | Location | Notes |
|-------|----------|-------|
| Database | SurrealDB namespace/database (`DB_NAMESPACE`, `DB_DATABASE`) | Export via SurrealDB backup tools or vendor snapshot |
| Blob files | `BLOB_DIR` (default `backend/blobs`) | Must stay consistent with `blob` table rows |
| Media files | `MEDIA_FINAL_DIR` (default `backend/media-assets`) | Must stay consistent with `media_asset` table rows |
| Configuration | Secrets manager / env | Not in git; document separately |

## Backup procedure (operator)

1. **Quiesce writes** — scale to one instance or enable maintenance mode if available.
2. **Export SurrealDB** — use your deployment’s supported export (`surreal export`, volume snapshot, or managed backup).
3. **Archive `BLOB_DIR` and `MEDIA_FINAL_DIR`** — `tar` or replicate both directories atomically relative to the DB snapshot time.
4. **Record** backend image tag / `git_commit` from `GET /api/v1/about`.

## Restore procedure

1. Deploy the **same or compatible** backend version that understands the schema revision in the backup.
2. Restore SurrealDB into a fresh namespace/database (or follow vendor restore docs).
3. Restore `BLOB_DIR` and `MEDIA_FINAL_DIR` to their configured paths.
4. Start a **single** backend instance first so migrations can run (or confirm `migration_script` matches shipped checksums).
5. Verify: health/readiness (when exposed), login, list songs, fetch a blob URL.

## Rollback policy

- **Application rollback:** redeploy the previous container image tag. Forward-only DB migrations mean the database may be **newer** than the rolled-back binary — only roll back app versions that remain compatible with the current schema.
- **Migration rollback:** **not supported.** Migrations are forward-only with checksum locking. To undo a schema change, ship a new migration that reverses the effect.
- **Data rollback:** restore from backup; do not rely on down-migrations.

## Development

Local `mem://` databases are ephemeral. For persistent dev data, run SurrealDB separately (see README) and set `DB_ADDRESS`, `DB_USERNAME`, `DB_PASSWORD`.

## Related

- [`backend/db-migrations/README.md`](../backend/db-migrations/README.md)
- [`forward-only-migrations.md`](forward-only-migrations.md)
- [`legacy-upgrade.md`](legacy-upgrade.md)
