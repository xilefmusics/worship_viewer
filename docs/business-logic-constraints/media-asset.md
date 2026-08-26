# Media assets

Applies to authenticated staging uploads and final asset delivery under `/api/v1/media/{media_id}/…`.

See also [media.md](./media.md) for the parent Media resource.

## Ownership and storage

- **BLC-MAST-001:** Media-owned assets belong to the same team `owner` as their parent Media row. Staging and final bytes are stored under opaque generated identities; filesystem paths never appear in API responses, Problem details, or client-visible errors.
- **BLC-MAST-002:** Staging uploads return only an opaque `operation_id`. Staged bytes are not readable through delivery endpoints until promoted to `final` status by backend processing (E5.4+).
- **BLC-MAST-003:** Final asset delivery requires read access to the owning team. Cross-team, guest-write, and nonexistent ids use the same concealed `404` semantics as other library resources.

## Upload transport

- **BLC-MAST-004:** `PUT /api/v1/media/{media_id}/uploads?kind=` streams a single request body to private staging storage without buffering the entire payload in memory. The `kind` query selects the configured byte limit (`video`, `audio`, `image`, `pdf`, `svg`).
- **BLC-MAST-005:** Oversize uploads are rejected with **413** when `Content-Length` exceeds the limit and while streaming when the body exceeds the limit regardless of headers. Partial staging files are removed on disconnect, limit failure, or handler error.
- **BLC-MAST-006:** Upload requires library write access to the parent Media owner team. There is no resumable or multipart upload in E5.3.
- **BLC-MAST-012:** For `kind=video` or `kind=audio`, a successful upload starts asynchronous Media processing (E5.4). URL media rejects video/audio uploads after staging when processing cannot begin.

## Delivery

- **BLC-MAST-007:** `GET` and `HEAD /api/v1/media/{media_id}/assets/{asset_id}/data` serve only `final` assets with private caching (`Cache-Control: private, max-age=3600, immutable`), weak `ETag`, `Content-Type`, `Content-Length`, `Accept-Ranges: bytes`, conditional requests (`If-None-Match`, `If-Range`), single-range **206** responses, and **416** for unsatisfiable ranges.
- **BLC-MAST-008:** Staging assets and mismatched `media_id`/`asset_id` pairs return concealed **404**.

## Processing and finals (E5.4)

- **BLC-MAST-013:** FFmpeg processors ingest processed output as a **new** final asset (`video/mp4` or `audio/mp4`), with SHA-256 ETag. The original staging file is not identity-promoted.
- **BLC-MAST-014:** On successful Media completion, staging source bytes and superseded final assets from replacement are deleted. Stale or cancelled job output is discarded without updating Media.
- **BLC-MAST-015:** Duplicate of ready uploaded media copies final file bytes to new opaque asset ids; staging and non-final rows are not duplicated.

## Operations and readiness

- **BLC-MAST-009:** Configurable staging/final directories, per-kind limits, processing timeouts, and FFmpeg/PDF tool paths are validated at startup when media processing is enabled. Missing required tools prevent backend startup with an actionable log message that does not expose command lines to API clients.
- **BLC-MAST-010:** Startup and periodic reconciliation remove abandoned staging files and stale staging metadata older than the configured age when they are not associated with an active upload or in-flight processing operation.
- **BLC-MAST-011:** Reconciliation also fails stranded Media rows stuck in `processing` or with `pending_revision.status=processing`, deletes their staging/temp files, and sets appropriate failed lifecycle state per [media.md](./media.md) (BLC-MEDIA-016).

## Blob boundary

- **BLC-MAST-016:** Media assets are separate from image-only Blob resources. Existing Blob upload, validation, and caching behavior for avatars, covers, and song attachments is unchanged.
