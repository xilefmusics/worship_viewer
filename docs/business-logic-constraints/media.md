# Media

Applies to the team-owned Media library at `/api/v1/media`.

## Resource and lifecycle

- **BLC-MEDIA-001:** Every stored Media has an opaque `id`, one team `owner`, a non-empty `title`, and required tagged `content`. There is no persisted lifecycle status or incomplete upload shell. Slide decks may additionally have a status-free `pending_revision` while being edited.
- **BLC-MEDIA-002:** Content tags are `slide_deck`, `video`, `audio`, `youtube`, `livestream`, and `web_page`. JSON create accepts only URL-backed tags; uploaded content is created by the multipart upload endpoint and cannot be fabricated by clients.
- **BLC-MEDIA-003:** Deletion is permitted without reference checks, is serialized against replacement/deck mutations, and removes every owned final or staging asset.

## URL safety and normalization

- **BLC-MEDIA-004:** YouTube accepts credential-free, fragment-free HTTPS URLs on exact supported YouTube hosts in watch, shortened, embed, shorts, and live forms. The server validates the 11-character video id and stores `https://www.youtube.com/watch?v={video_id}` plus the extracted id.
- **BLC-MEDIA-005:** Livestream and web-page URLs must be absolute, credential-free, fragment-free HTTPS URLs with a host. They are validated syntactically only; the backend does not fetch, proxy, or test embeddability.
- **BLC-MEDIA-006:** Livestreams whose case-insensitive URL path ends in `.m3u8` are tagged `hls`; all other accepted livestream URLs are tagged `direct`.
- **BLC-MEDIA-007:** URL failures use stable Problem codes: `media_invalid_url` for malformed/disallowed URL structure or identifiers and `media_unsupported_url` for unsupported schemes/hosts. Parser internals are never returned.

## Uploaded audio/video (E5.4)

- **BLC-MEDIA-013:** `POST /api/v1/media/uploads?kind=video|audio|slide_deck` accepts multipart metadata plus files, processes all sources inside the request, and creates the Media only after final content and assets are complete.
- **BLC-MEDIA-014:** Audio/video creation requires exactly one source. FFprobe and FFmpeg complete before the response; success returns `201 Media`, while failure returns a Problem and leaves no Media or final assets.
- **BLC-MEDIA-015:** `PUT /api/v1/media/{id}/uploads?kind=video|audio` synchronously replaces matching uploaded content and returns the updated Media. Existing content remains unchanged on every failure.
- **BLC-MEDIA-016:** Successful replacement atomically swaps the content reference and then deletes the superseded asset. Media mutations for the same id are serialized.
- **BLC-MEDIA-017:** Processing errors expose stable codes (`media_input_invalid`, `media_input_unsupported`, `media_processing_timeout`, `media_processing_failed`) and short English detail only—never argv, paths, or raw tool output.
- **BLC-MEDIA-018:** Retry is a new request from byte zero. Request cancellation terminates child processing and cleanup guards remove temporary and partially ingested data; there is no processing-cancel endpoint.
- **BLC-MEDIA-019:** `UpdateMedia.content` is optional. Uploaded items may update title (and owner via move) without sending URL content. Sending URL content onto an uploaded item is rejected.

## Slide decks (E5.5)

- **BLC-MEDIA-020:** Multipart deck creation requires one or more files, expands every source synchronously in form order, and creates no Media if any source fails.
- **BLC-MEDIA-021:** Synchronous deck additions/replacements return a draft as `pending_revision={revision_id,pages}`. Committed `content` remains playable and unchanged until commit.
- **BLC-MEDIA-022:** `POST /api/v1/media/{id}/deck/commit` with `{ revision_id, page_ids }` atomically installs the selected non-empty order, rejects stale/unknown/duplicate ids, clears the revision, and deletes unreferenced assets.
- **BLC-MEDIA-023:** `POST /api/v1/media/{id}/deck/revisions` copies committed pages into a revision for reorder/removal without altering live content.
- **BLC-MEDIA-024:** Sources are sniffed by bytes (PNG/JPEG/sanitized SVG/PDF). Invalid, encrypted, animated, unsafe, oversized, or over-500-page inputs fail synchronously. PDF pages remain single-page PDFs without stored rasterization.
- **BLC-MEDIA-025:** Slide-deck pages may carry an optional `section_title` on the first page of a section. The title is preserved through draft revisions, commit, and duplicate, and it does not affect page identity or commit ordering.

## Access and operations

- **BLC-MEDIA-008:** Reads and list results use the caller's membership-derived readable teams. Unreadable or nonexistent ids return the same concealed `404`.
- **BLC-MEDIA-009:** Create, update, duplicate, move, and delete require library write access (`admin` or `content_maintainer`) to the owning team. Guests/read-only members may read but not mutate; denied mutations use concealed `404` responses.
- **BLC-MEDIA-010:** A move validates write access to both the current and destination teams before atomically changing `owner` and all owned `media_asset` rows. Moving to the current owner is idempotent.
- **BLC-MEDIA-011:** Duplicate requires write access to the source and destination. URL duplicates copy normalized content only. Uploaded video/audio/slide-deck duplicates copy final asset bytes to new opaque ids and rewrite `blob_id` values; deck revisions and staging are not copied.
- **BLC-MEDIA-012:** List filtering applies readable-team scope first, then optional `team` and case-insensitive/full-text title search, then pagination. Ordering is stable by title and id (or search score, title, and id for searches). Responses include `X-Total-Count` and RFC 5988 pagination links.

## Media-owned assets (E5.3+)

Uploaded and processed bytes are governed by [media-asset.md](./media-asset.md). E5.1 URL media has no owned assets.
