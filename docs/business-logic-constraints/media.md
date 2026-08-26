# Media

Applies to the team-owned Media library at `/api/v1/media`.

## Resource and lifecycle

- **BLC-MEDIA-001:** Media has an opaque `id`, one team `owner`, a non-empty `title`, `status` (`processing`, `ready`, or `failed`), optional active tagged `content`, and optional `pending_revision`. URL media created in E5.1 is validated synchronously and is returned as `ready`.
- **BLC-MEDIA-002:** Active content is tagged by `type`. The stable tags are `slide_deck`, `video`, `audio`, `youtube`, `livestream`, and `web_page`. E5.1 create/update bodies accept only `youtube`, `livestream`, and `web_page`; clients cannot fabricate uploaded/deck content.
- **BLC-MEDIA-003:** Deletion is permitted without reference checks. Future stale links remain the referencing resource's concern.

## URL safety and normalization

- **BLC-MEDIA-004:** YouTube accepts credential-free, fragment-free HTTPS URLs on exact supported YouTube hosts in watch, shortened, embed, shorts, and live forms. The server validates the 11-character video id and stores `https://www.youtube.com/watch?v={video_id}` plus the extracted id.
- **BLC-MEDIA-005:** Livestream and web-page URLs must be absolute, credential-free, fragment-free HTTPS URLs with a host. They are validated syntactically only; the backend does not fetch, proxy, or test embeddability.
- **BLC-MEDIA-006:** Livestreams whose case-insensitive URL path ends in `.m3u8` are tagged `hls`; all other accepted livestream URLs are tagged `direct`.
- **BLC-MEDIA-007:** URL failures use stable Problem codes: `media_invalid_url` for malformed/disallowed URL structure or identifiers and `media_unsupported_url` for unsupported schemes/hosts. Parser internals are never returned.

## Access and operations

- **BLC-MEDIA-008:** Reads and list results use the caller's membership-derived readable teams. Unreadable or nonexistent ids return the same concealed `404`.
- **BLC-MEDIA-009:** Create, update, duplicate, move, and delete require library write access (`admin` or `content_maintainer`) to the owning team. Guests/read-only members may read but not mutate; denied mutations use concealed `404` responses.
- **BLC-MEDIA-010:** A move validates write access to both the current and destination teams before atomically changing `owner`. Moving to the current owner is idempotent.
- **BLC-MEDIA-011:** Duplicate requires write access to the source and destination and creates a distinct Media row with copied normalized content. In E5.1 there are no owned assets to copy; subsequent edits or deletion are independent.
- **BLC-MEDIA-012:** List filtering applies readable-team scope first, then optional `team` and case-insensitive/full-text title search, then pagination. Ordering is stable by title and id (or search score, title, and id for searches). Responses include `X-Total-Count` and RFC 5988 pagination links.

## Media-owned assets (E5.3+)

Uploaded and processed bytes are governed by [media-asset.md](./media-asset.md). E5.1 URL media has no owned assets.
