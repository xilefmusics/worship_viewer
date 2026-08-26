# Media

Applies to the team-owned Media library at `/api/v1/media`.

## Resource and lifecycle

- **BLC-MEDIA-001:** Media has an opaque `id`, one team `owner`, a non-empty `title`, `status` (`processing`, `ready`, or `failed`), optional active tagged `content`, optional `pending_revision`, and optional `declared_kind` (`video` or `audio`) for upload shells without active content. URL media created in E5.1 is validated synchronously and is returned as `ready`.
- **BLC-MEDIA-002:** Active content is tagged by `type`. The stable tags are `slide_deck`, `video`, `audio`, `youtube`, `livestream`, and `web_page`. Create bodies accept `youtube`, `livestream`, `web_page`, and unit `video` / `audio` shells; clients cannot fabricate processed upload content or slide decks.
- **BLC-MEDIA-003:** Deletion is permitted without reference checks. Future stale links remain the referencing resource's concern. Deleting media cancels in-flight processing and removes owned final and staging assets.

## URL safety and normalization

- **BLC-MEDIA-004:** YouTube accepts credential-free, fragment-free HTTPS URLs on exact supported YouTube hosts in watch, shortened, embed, shorts, and live forms. The server validates the 11-character video id and stores `https://www.youtube.com/watch?v={video_id}` plus the extracted id.
- **BLC-MEDIA-005:** Livestream and web-page URLs must be absolute, credential-free, fragment-free HTTPS URLs with a host. They are validated syntactically only; the backend does not fetch, proxy, or test embeddability.
- **BLC-MEDIA-006:** Livestreams whose case-insensitive URL path ends in `.m3u8` are tagged `hls`; all other accepted livestream URLs are tagged `direct`.
- **BLC-MEDIA-007:** URL failures use stable Problem codes: `media_invalid_url` for malformed/disallowed URL structure or identifiers and `media_unsupported_url` for unsupported schemes/hosts. Parser internals are never returned.

## Uploaded audio/video (E5.4)

- **BLC-MEDIA-013:** `POST /api/v1/media` with `content.type` `video` or `audio` creates a processing shell: `status=processing`, `content=null`, and `declared_kind` set so list and editor surfaces know the intended kind.
- **BLC-MEDIA-014:** After a successful `PUT /uploads?kind=video|audio`, the server returns `{ operation_id }` immediately and spawns asynchronous FFmpeg processing. Upload kind must match `declared_kind` or the active uploaded content type; URL media rejects uploads with **400**.
- **BLC-MEDIA-015:** Initial upload sets `pending_revision` with `status=processing` and the operation id. Replacement of ready uploaded media keeps `status=ready` and active `content` while `pending_revision` tracks the new operation. A newer upload supersedes in-flight work; stale job completion is ignored and its temp output is discarded.
- **BLC-MEDIA-016:** Success ingests a new final asset and sets `content` to `video` or `audio` with `blob_id`, duration, and video dimensions when applicable; `status=ready`; clears `pending_revision`; deletes the staging source and any superseded final asset. Initial failure sets `status=failed` with no content. Replacement failure leaves ready content unchanged and sets `pending_revision.status=failed` with a safe `processing_error`.
- **BLC-MEDIA-017:** Processing errors expose stable codes (`media_input_invalid`, `media_input_unsupported`, `media_processing_timeout`, `media_processing_failed`) and short English detail only—never argv, paths, or raw tool output.
- **BLC-MEDIA-018:** Retry is always a new upload from byte zero (new operation). `POST /api/v1/media/{id}/processing/cancel` clears replacement `pending_revision`, cancels the job, and does not delete ready media.
- **BLC-MEDIA-019:** `UpdateMedia.content` is optional. Uploaded items may update title (and owner via move) without sending URL content. Sending URL content onto an uploaded item is rejected.

## Access and operations

- **BLC-MEDIA-008:** Reads and list results use the caller's membership-derived readable teams. Unreadable or nonexistent ids return the same concealed `404`.
- **BLC-MEDIA-009:** Create, update, duplicate, move, and delete require library write access (`admin` or `content_maintainer`) to the owning team. Guests/read-only members may read but not mutate; denied mutations use concealed `404` responses.
- **BLC-MEDIA-010:** A move validates write access to both the current and destination teams before atomically changing `owner` and all owned `media_asset` rows. Moving to the current owner is idempotent.
- **BLC-MEDIA-011:** Duplicate requires write access to the source and destination. URL duplicates copy normalized content only. Ready uploaded video/audio duplicates copy final asset bytes to new opaque ids and rewrite `blob_id` in content; `pending_revision` and staging are not copied.
- **BLC-MEDIA-012:** List filtering applies readable-team scope first, then optional `team` and case-insensitive/full-text title search, then pagination. Ordering is stable by title and id (or search score, title, and id for searches). Responses include `X-Total-Count` and RFC 5988 pagination links.

## Media-owned assets (E5.3+)

Uploaded and processed bytes are governed by [media-asset.md](./media-asset.md). E5.1 URL media has no owned assets.
