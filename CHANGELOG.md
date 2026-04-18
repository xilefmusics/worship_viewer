# Changelog

All notable API and packaging changes for Worship Viewer are documented here. The API version in OpenAPI `info.version` marks **wire-format** generations for `/api/v1`.

## 2.0.0 — 2026-04-18

Breaking HTTP/API changes (paths remain under `/api/v1`). See [docs/api-breaking-2-0.md](docs/api-breaking-2-0.md) for migration detail.

- **Player responses:** `PlayerItem` is internally tagged (`type` + `blob_id` or `song`).
- **Songs:** `blobs` are link objects (`BlobLink`: `{ "id" }`), not bare strings.
- **Sessions:** wire model is `SessionBody`; narrow `user` by default; use `expand=user` for full user JSON.
- **Errors:** `Problem` no longer includes legacy `error`; use `detail` and `code`.
- **PUT bodies:** OpenAPI names `UpdateSong`, `UpdateCollection`, `UpdateSetlist`, and `UpdateBlob` (same JSON shapes as the former create types where applicable).
- **OpenAPI:** chord payload component is consistently named `SongDataSchema` (fixes invalid `$ref` in the published document).
