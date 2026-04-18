# API 2.0 breaking changes (`/api/v1`)

OpenAPI `info.version` is **2.0.0**. Wire-format changes below apply to the same `/api/v1` prefix unless noted.

## `PlayerItem` (player JSON)

- **Before:** Externally tagged variants with PascalCase keys, e.g. `{ "Blob": "id" }`, `{ "Chords": { …song } }`.
- **After:** Internally tagged `{ "type": "blob", "blob_id": "…" }` and `{ "type": "chords", "song": { … } }`.

Affected responses: `GET …/songs/{id}/player`, `GET …/collections/{id}/player`, `GET …/setlists/{id}/player`.

## `Song.blobs`

- **Before:** `blobs: string[]` (raw blob ids).
- **After:** `blobs: { "id": string }[]` ([`BlobLink`](../shared/src/blob/blob.rs)).

## Sessions

- **Wire type:** Responses use [`SessionBody`](../shared/src/user/session.rs) (OpenAPI component), not the internal storage shape.
- **Default `user`:** `id` + `email` only (link).
- **Embed full user:** `GET`/`POST` session endpoints accept `expand=user` (comma-separated list). The bundled API client appends `expand=user` for session calls so existing tooling still receives full `User` JSON by default.

## Problem (errors)

- **Removed:** Top-level `error` (duplicate of `detail`). Clients must use `detail` and `code`.

## PUT request bodies

Dedicated OpenAPI types (same JSON shape as before, new schema names):

| Resource    | PUT body schema   |
|------------|-------------------|
| Song       | `UpdateSong`      |
| Collection | `UpdateCollection`|
| Setlist    | `UpdateSetlist`   |
| Blob       | `UpdateBlob`      |

`PUT /songs/{id}` remains **upsert** (201 + `Location` when created). Other PUTs are **replace** on existing ids only (404 when missing).

## Verification

- Rust guard: `cargo test -p backend blc_docs_004` (schema property keys snake_case).
- CI also runs [Spectral](https://stoplight.io/open-source/spectral) on generated OpenAPI (ruleset [`.spectral.yaml`](../.spectral.yaml)).
- Regenerate OpenAPI locally: `cargo run --manifest-path backend/Cargo.toml --example print_openapi`.
