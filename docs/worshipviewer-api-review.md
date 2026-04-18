# Worship Viewer REST API — Design Review

**Source:** `https://app.worshipviewer.com/api/docs/openapi.json`
**Spec version:** OpenAPI 3.0.3 · API `1.0.0`
**Scope:** Design, consistency, and best-practice review of the public contract. No live server traffic was made; findings are derived strictly from the OpenAPI document.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Scorecard](#2-scorecard)
3. [What's Working Well](#3-whats-working-well)
4. [Findings by Topic](#4-findings-by-topic)
   1. [Resource & URL Design](#41-resource--url-design)
   2. [Identifier & Reference Style (the `blobs` question)](#42-identifier--reference-style-the-blobs-question)
   3. [Naming Conventions](#43-naming-conventions)
   4. [Enums & Value Casing](#44-enums--value-casing)
   5. [Error Model](#45-error-model)
   6. [HTTP Status Coverage](#46-http-status-coverage)
   7. [Conditional Requests / Concurrency](#47-conditional-requests--concurrency)
   8. [Pagination](#48-pagination)
   9. [Sorting & Filtering](#49-sorting--filtering)
   10. [PATCH & Partial Update Semantics](#410-patch--partial-update-semantics)
   11. [PUT / Create / Update Shapes](#411-put--create--update-shapes)
   12. [Polymorphism (`PlayerItem`)](#412-polymorphism-playeritem)
   13. [Reference vs Expansion Strategy](#413-reference-vs-expansion-strategy)
   14. [Binary / Blob Handling](#414-binary--blob-handling)
   15. [Auth Endpoints](#415-auth-endpoints)
   16. [Security, CSRF, Rate Limits](#416-security-csrf-rate-limits)
   17. [Dates, Numbers, Constraints](#417-dates-numbers-constraints)
   18. [Domain Fields Without Documentation](#418-domain-fields-without-documentation)
   19. [OpenAPI Hygiene](#419-openapi-hygiene)
   20. [Missing / Asymmetric Operations](#420-missing--asymmetric-operations)
5. [Prioritized Action List](#5-prioritized-action-list)
6. [Appendix A — Schema Diffs (Illustrative)](#6-appendix-a--schema-diffs-illustrative)
7. [Appendix B — Checklist Against Common Standards](#7-appendix-b--checklist-against-common-standards)

---

## 1. Executive Summary

The Worship Viewer API is a **well-structured, versioned REST API** with clear resource boundaries, good pagination basics, ETag support for reads, and an explicit auth story. The spec is detailed enough that it could drive usable client generation today.

However, several **cross-cutting consistency issues** reduce client quality and make the API harder to evolve:

- **Two competing error schemas** (`ErrorResponse` and `ProblemDetails`) are used on different routes for the same error class.
- **Enum casing styles disagree** across the spec (UPPERCASE, PascalCase, snake_case, lowercase).
- **Reference strategy is mixed**: some relations embed full objects, some use `{id, …}` links, some use bare ID strings.
- **ETag is read-only**: no `If-Match` / `412 Precondition Failed` for writes, so optimistic concurrency is incomplete.
- **Pagination rules differ by route** (defaults vs "omit for full list").
- **Auth endpoint parameters look misdeclared** (`in: path` for `code`, `state`, `redirect_to`, `provider`).
- **`PlayerItem` polymorphism** uses externally-tagged Rust-style JSON that is unfriendly for typical JSON consumers and OpenAPI codegen.

None of these are fatal. Each has a conservative fix that can land in v1 without breaking clients, and a preferred fix that is best scheduled for v2.

---

## 2. Scorecard

| Area | Grade | One-line summary |
|------|:----:|------------------|
| Resource modeling | B+ | Clear nouns, good sub-resources; a few asymmetric paths. |
| Naming (keys) | B | Mostly snake_case; `PlayerItem` breaks it. |
| Naming (enums) | C | Four different casing conventions coexist. |
| Error model | C- | Two schemas, not actually served as `application/problem+json`. |
| Status codes | B- | Good coverage, but 404/403/412 inconsistently declared. |
| Pagination | B | Solid basics; semantics drift between endpoints. |
| Filtering/Sorting | C | Only songs filter; custom sort tokens; no general pattern. |
| Concurrency / ETag | C+ | Read-side only; no write-side `If-Match`. |
| Polymorphism | C- | `oneOf` without discriminator; externally-tagged keys. |
| Binary handling | B- | Two-step upload is fine; content types need tightening. |
| Security scheme | B+ | Cookie + bearer declared; CSRF discussed. |
| Docs quality | A- | Good prose intro; per-route descriptions are useful. |
| Evolvability | B- | `additionalProperties: false` on inputs is good; some structural choices will bite v2. |

---

## 3. What's Working Well

- **Versioned base path** (`/api/v1`) and explicit "auth lives at `/auth/*` and is unversioned" declaration. Removes a common source of ambiguity.
- **Stable resource set** — `users`, `songs`, `collections`, `setlists`, `blobs`, `teams` — each with uniform CRUD.
- **Conditional GET** via weak ETag + `If-None-Match` on singleton reads.
- **Pagination skeleton**: `page`, `page_size`, `X-Total-Count` header — familiar, scales, avoids envelope churn.
- **Security declaration** is explicit per-operation (cookie + bearer), and `CSRF` notes appear in the intro.
- **`additionalProperties: false`** on all `Create*` / `Patch*` / `Update*` inputs — catches client typos at validation time, excellent choice.
- **Documented three-state semantics** on `PatchSongData` (`Missing` / `Null` / `Value(v)`) — this is the single clearest piece of schema documentation in the spec, and it should be the template for the rest of the API.
- **Separate input vs output types** for teams (`TeamUserRef` vs `TeamUser`, `TeamMemberInput` vs `TeamMember`) — correctly prevents clients from being forced to send server-owned fields.

---

## 4. Findings by Topic

### 4.1 Resource & URL Design

**Observations**

- Plural nouns, `/collection/{id}/songs`, `/setlists/{id}/player`, `/songs/{id}/like` — standard REST.
- Path parameter names drift: `{id}`, `{team_id}`, `{user_id}`, `{invitation_id}`. Functionally equivalent but inconsistent.
- `/api/v1/invitations/{invitation_id}/accept` is a **top-level** invitation route, while invitations are otherwise modeled as a sub-resource of a team (`/teams/{team_id}/invitations/{invitation_id}`). The accept action therefore lives on a path the rest of the API does not otherwise expose.
- `POST /api/v1/users/{user_id}/sessions` exists alongside `POST /auth/otp/verify` (which also creates a session). Two entry points for the same domain object.
- `/users/me` and `/users/{id}` coexist — fine and idiomatic, though `/users/me/sessions` duplicates `/users/{user_id}/sessions` with a self shortcut.

**Recommendations**

- **Choose one path-parameter style**: either always `{id}` scoped by its position in the path, or always `{resource_id}`. The former is slightly more common in OpenAPI toolchains; the latter is friendlier when the same path has two IDs (see `/teams/{team_id}/invitations/{invitation_id}`).
- **Make invitation acceptance symmetric** with the rest of invitations:
  - Preferred: `POST /api/v1/teams/{team_id}/invitations/{invitation_id}/accept`.
  - Or: introduce a top-level `/invitations/{id}` GET that returns the invitation resource so clients can fetch context before accepting.
- Consider whether `POST /users/{user_id}/sessions` is still needed now that OTP is the documented flow; if it is admin impersonation, rename it explicitly (`/users/{user_id}/impersonation-sessions`).

---

### 4.2 Identifier & Reference Style (the `blobs` question)

This is the specific question you raised: `Song.blobs: string[]` vs a nested object.

**Today**

- `Song.blobs: array<string>` — bare blob IDs.
- `Collection.songs: array<SongLink>` where `SongLink = { id, key?, nr? }`.
- `Setlist.songs: array<SongLink>`.
- `Session.user: User` (full embed).
- `Team.owner: TeamUser`, `Team.members: TeamMember[]` (partial embed).
- `PlayerItem.Blob: string` (bare blob ID again, but PascalCase key).

**Analysis — three options**

| Option | Pros | Cons | When it's right |
|--------|------|------|-----------------|
| `string[]` of IDs | Smallest payload, cheapest encoding/decoding | Name doesn't signal "this is an ID", no room for per-link metadata, inconsistent with `SongLink` | Only when relation is **ordered list of pure foreign keys** and will stay that way |
| `{ id: string }[]` (minimal ref object) | Matches `SongLink`; extensible without breaking change | Slightly more verbose JSON | When the relation **might** grow per-link attributes |
| Full expanded `Blob[]` | Single round trip | Coupling: every song read fetches every blob metadata; write payloads get ambiguous | Only for small, read-heavy aggregates |

**Recommendation**

Pick one of the first two and **apply the same rule for every cross-resource reference**:

- If you stay with bare IDs, **rename the property** so the contract is self-describing: `blob_ids: string[]`. Do the same for `PlayerItem`'s `Blob` variant (see §4.12).
- If you want ever to attach per-blob metadata (page number, role, crop, alt text, display order, annotation overlay…), introduce **`BlobLink = { id: string }`** *now*. Adding `BlobLink.page` later is a non-breaking additive change; adding a field to a `string` is a breaking change.

The larger point is **consistency of pattern**: today `Song → Blob` uses one shape and `Collection → Song` uses another. Clients and SDKs need one mental model ("every cross-resource link is an object with `id` plus optional link metadata") to generate clean code.

A pragmatic resolution:

- **v1 (additive):** keep `blobs: string[]`, rename the field in docs/schema to `blob_ids`, and ship a new `blob_links: BlobLink[]` as an optional, forward-compatible field.
- **v2:** drop `blob_ids` in favor of `blob_links`.

---

### 4.3 Naming Conventions

- JSON property names are **almost entirely snake_case** (`created_at`, `default_collection`, `not_a_song`, `file_type`) — this is the house style.
- `PlayerItem` breaks the rule with **PascalCase** keys (`Blob`, `Chords`). This is the serialization signature of Rust `serde` externally-tagged enums; it should be translated at the API boundary.
- `operationId`s are snake_case (`create_session_for_user`) — matches the rest.
- Tag names are TitleCase (`Songs`, `Auth`) — also fine, tags are presentation-only.

**Recommendation**

Adopt one rule: *"All JSON object keys in request and response bodies are `snake_case`."* Encode it as a lint (Spectral's `oas3-schema` + a custom `camelCase`/`snake_case` rule) and run it in CI. `PlayerItem` is currently the only violator but future code gen will add more unless guarded.

---

### 4.4 Enums & Value Casing

Current enum values:

| Schema | Values | Style |
|--------|--------|-------|
| `FileType` | `PNG`, `JPEG`, `SVG` | UPPERCASE |
| `Orientation` | `Portrait`, `Landscape` | PascalCase |
| `ScrollType` | `OnePage`, `HalfPage`, `TwoPage`, `Book`, `TwoHalfPage` | PascalCase |
| `Role` | `default`, `admin` | lowercase |
| `TeamRole` | `guest`, `content_maintainer`, `admin` | snake_case |

Four different conventions in one spec. Clients that want to model these as typed unions/enums must now codify five different formatters.

**Recommendation**

Standardize on **`lower_snake_case`** for enum values across the API (it matches JSON keys and is the least controversial choice). Examples:

- `FileType`: `png`, `jpeg`, `svg`
- `Orientation`: `portrait`, `landscape`
- `ScrollType`: `one_page`, `half_page`, `two_page`, `book`, `two_half_page`

If backward compatibility matters, accept both casings on input and emit the new casing on output, with a deprecation window.

Bonus: `FileType` doubles as a MIME hint. Consider making it an actual MIME string (`image/png`, `image/jpeg`, `image/svg+xml`) — note the current `image/svg` is not a registered MIME type; the correct value is `image/svg+xml`.

---

### 4.5 Error Model

The intro promises RFC 7807:

> Error responses use `application/problem+json` (RFC 7807) with `type`, `title`, `status`, `detail`, and `code`.

Reality in the spec:

- Two schemas are defined:
  - `ErrorResponse` — `{ code, error }` only.
  - `ProblemDetails` — `{ type, title, status, detail, code, error, instance? }`.
- **Most endpoints** (blobs, collections, setlists, teams, users, auth) declare `application/json` responses with `$ref: ErrorResponse`.
- **Songs** and **player** endpoints declare `application/json` responses with `$ref: ProblemDetails`.
- **No endpoint** declares `application/problem+json` as the response content type.

Consequences:

1. Clients cannot rely on a single decode path; SDKs end up modeling a union.
2. Observability tooling that filters by MIME type sees everything as `application/json`, including problem documents — defeating one of RFC 7807's benefits.
3. The `error` field in `ProblemDetails` is already marked "legacy alias" — shipping it in a fresh v1 locks in debt.

**Recommendations**

- **Single canonical error schema**: `ProblemDetails` (keep it), and drop `ErrorResponse` in favor of it. Or, if you truly want two shapes, define `ErrorResponse` as a strict projection and document precisely which routes use which.
- **Serve problem documents as `application/problem+json`** on 4xx/5xx, and declare that content type in the spec.
- **Registered `code` values**: expose an enum (or at minimum a documented list) of valid `code` strings so clients can switch on them safely.
- Remove the `error` legacy alias or scope it to the one client that needs it with a sunset date.

---

### 4.6 HTTP Status Coverage

**Representative gaps**

- `PUT /api/v1/setlists/{id}` documents 200/400/401/500 but **not 404**, while the analogous collection and song endpoints do include 404.
- Mutating endpoints (`PUT`, `PATCH`, `DELETE`) that touch resources owned by other users **should** document `403` (cross-user access denied). Many only have 401/404.
- No endpoint documents `412 Precondition Failed`, despite ETag support (§4.7).
- `POST` endpoints conflate schema-invalid bodies and domain-invalid bodies as `400`. Consider `422 Unprocessable Entity` for semantically valid JSON that violates business rules.
- `500` is listed on almost every operation. That's realistic but noisy; some teams prefer to document it only on operations that can fail in a specific, actionable way.
- `GET /api/v1/users/me` doesn't list `403` — fine — but it also doesn't list `404` for "no current user resource", which is technically possible for deleted-but-session-alive states.

**Recommendation**

Define a **response inclusion policy** and apply it uniformly:

- Always list: `400` (or `422`), `401`, `403` (where the resource can be owned), `404` (where the resource is addressable), `409` (conflict / concurrency), `412` (if ETag-aware), `429` (if rate-limited).
- `500` is optional but document it consistently (all or none).

---

### 4.7 Conditional Requests / Concurrency

- Singleton `GET` operations document weak ETag + `If-None-Match` → 304. Good.
- **No** write-side documentation of `If-Match` + `412 Precondition Failed`.

Without `If-Match`, two clients editing the same song can silently clobber each other's changes (last-write-wins). For a collaborative worship tool where multiple team members may edit a setlist or song, this is worth closing.

**Recommendation**

- Document `If-Match` on `PUT`, `PATCH`, `DELETE` for all resources that expose an ETag.
- Add `412 Precondition Failed` to those operations.
- Add `428 Precondition Required` if you want to **enforce** clients to supply `If-Match` (optional, strict mode).
- Strong or weak ETag is fine; be consistent.

---

### 4.8 Pagination

- Common pattern: `page` (zero-based), `page_size` (1–500), `X-Total-Count` header.
- **Inconsistency #1:** Some endpoints say "defaults to 0 / 50". Others say "omit `page` and `page_size` for full list". Those behaviors are fundamentally different and clients must choose which to assume.
- **Inconsistency #2:** `page_size.minimum` is `0` in the schema but the description is `1–500`. Schema and prose disagree.
- **Inconsistency #3:** Some list endpoints expose `q` as "Reserved; not used". Declaring unused parameters adds noise and tempts future undocumented behavior.
- No `Link` header (RFC 5988) with `next`/`prev` URIs — clients must construct pages themselves.
- No cursor pagination option — problematic for large exports or frequently mutated lists where offset pagination skips/duplicates rows.

**Recommendations**

- Pick one semantic: *"Pagination is always on; defaults are page=0, page_size=50."* Return a full list only via an explicit `all=true` or, better, never.
- Fix `page_size.minimum` to `1` and add `maximum: 500`.
- Remove `q` from endpoints that don't use it. Add it only when it does something.
- Add `Link` headers (`rel="next"`, `rel="prev"`, `rel="first"`, `rel="last"`) — cheap to implement, standard to consume.
- Consider cursor pagination (`cursor`, `next_cursor`) for `/songs` and `/setlists/{id}/songs`, which are the highest-cardinality lists.

---

### 4.9 Sorting & Filtering

- Only `/api/v1/songs` supports `sort`, `lang`, `tag`.
- `sort` uses custom tokens like `id_desc`, `title_asc`, `relevance`. Common alternatives:
  - Single field with `-` prefix for descending: `sort=-id`, `sort=title`. (JSON:API flavor.)
  - `sort` + `order`: `sort=id&order=desc`. (Classic.)
  - Multi-field: `sort=-id,title`.
- `tag` filter is documented as "case-insensitive substring match on stringified `data.tags`" — this **leaks the server's internal storage representation** into the public contract. If storage ever changes (say, `data.tags` becomes a typed object), this filter's behavior changes.
- No sort/filter options on collections, setlists, teams, users.

**Recommendations**

- Define a single sort syntax for the whole API and apply it everywhere a list is sortable.
- Rephrase `tag` filtering in terms of the domain, not the storage: e.g., "songs whose tag set contains a key/value where key equals `tag`".
- Where filtering makes sense for other resources, add it (e.g., `collections?owner=<id>`, `setlists?owner=<id>`).

---

### 4.10 PATCH & Partial Update Semantics

- `PatchSongData` documents the three-state merge semantics clearly — **this is excellent** and should be the model for every `Patch*`.
- `PatchSong` itself has `required: ["data"]`. Forcing `data` to be present on a partial update is unusual — callers who only want to flip `not_a_song` must still include `data`.
- The content type for PATCH is `application/json`, not `application/merge-patch+json` or `application/json-patch+json`. That is a valid choice, but it means the server is defining a custom merge format; document that format explicitly and link it from every `PATCH` route.

**Recommendations**

- Drop `required: ["data"]` from `PatchSong` (and audit the rest of the `Patch*` schemas for the same issue).
- Either:
  - Serve `application/merge-patch+json` and mention RFC 7396, or
  - Keep `application/json` and publish one short section titled "How PATCH works" that defines: "fields absent → unchanged; fields set to `null` on nullable properties → cleared; all other fields → replaced wholesale".

---

### 4.11 PUT / Create / Update Shapes

- Most resources reuse `Create<X>` for both `POST` (create) and `PUT` (replace). That's unusual but defensible if the two really are identical.
- `Team` uses **three** types: `CreateTeam`, `UpdateTeam`, `PatchTeam`. Good when semantics differ; worth asking whether the gap is real.
- `User` creation uses `CreateUserRequest` (suffix `Request`) — naming drift from every other `Create<X>`.
- `PUT /api/v1/songs/{id}` documents upsert: *"if the id does not exist, creates the song"*. That is RFC-correct PUT behavior, but **no other resource documents this**, so clients cannot tell whether collections/setlists/blobs behave the same way.

**Recommendations**

- Rename `CreateUserRequest` → `CreateUser` for consistency.
- Document upsert explicitly on every `PUT` that actually upserts, or explicitly deny upsert and return `404` when the `id` doesn't exist.
- Consider whether `CreateTeam` and `UpdateTeam` can be unified; if not, add the same split to other resources whose create/update surface differs.

---

### 4.12 Polymorphism (`PlayerItem`)

```json
"PlayerItem": {
  "oneOf": [
    { "type": "object", "required": ["Blob"],   "properties": { "Blob":   { "type": "string" } } },
    { "type": "object", "required": ["Chords"], "properties": { "Chords": { "$ref": "#/components/schemas/Song" } } }
  ]
}
```

Issues:

1. **Externally-tagged** format — the variant name *is* the JSON key. This is Rust `serde` default; it's legal JSON but it means consumers have to probe for keys to discriminate.
2. **PascalCase** keys break the rest of the API.
3. **No `discriminator`** on the `oneOf` — OpenAPI code generators cannot produce clean typed unions.
4. The `Blob` variant is a bare string — confusing because `Blob` is also a full schema elsewhere.

**Recommended shape (internally-tagged with a discriminator):**

```json
"PlayerItem": {
  "oneOf": [
    { "$ref": "#/components/schemas/PlayerBlobItem" },
    { "$ref": "#/components/schemas/PlayerChordsItem" }
  ],
  "discriminator": { "propertyName": "type", "mapping": {
    "blob":   "#/components/schemas/PlayerBlobItem",
    "chords": "#/components/schemas/PlayerChordsItem"
  }}
}

"PlayerBlobItem":   { "type": "object", "required": ["type","blob_id"],
                      "properties": { "type": { "const": "blob" }, "blob_id": { "type": "string" } } }
"PlayerChordsItem": { "type": "object", "required": ["type","song"],
                      "properties": { "type": { "const": "chords" }, "song":    { "$ref": "#/components/schemas/Song" } } }
```

This is a breaking change for existing clients; schedule for v2 or run both during a transition.

---

### 4.13 Reference vs Expansion Strategy

The spec mixes four strategies for cross-resource relationships:

| Relationship | Strategy today |
|---|---|
| `Song.blobs` | Bare ID strings |
| `Collection.songs`, `Setlist.songs` | `SongLink = { id, key?, nr? }` |
| `Team.owner`, `Team.members[].user` | Expanded `TeamUser` |
| `Session.user` | Full `User` |
| `TeamInvitation.created_by` | `TeamUser` |
| `PlayerItem.Blob` | Bare ID string (in PascalCase) |

There are legitimate reasons for each choice, but the design is undiscoverable: a client cannot predict the shape of a new relationship without reading the schema.

**Recommendation**

Adopt one of these documented policies:

- **Always-reference policy:** every relation is `{ id, …link-metadata }`. Use a query parameter `?expand=owner,members.user` when the client wants the embedded object.
- **Size-based policy:** relations with $\leq N$ fields embed, larger ones reference. Document the N and the list of embedded types.

Then migrate gradually. An `?expand=` mechanism is ideal because it leaves today's responses compatible.

---

### 4.14 Binary / Blob Handling

- Two-step create: `POST /blobs` (metadata) → `PUT /blobs/{id}/data` (bytes). Clean, idempotent, cacheable.
- `GET /blobs/{id}/data` returns `image/*`. Stated variants: `image/png`, `image/jpeg`, `image/svg` — the last one should be **`image/svg+xml`**.
- `413 Payload Too Large` documented on upload. Good.
- No `ETag`/`Last-Modified`/`Cache-Control` documented on the binary route — typical CDN wins are left on the table.
- No `Range` / `206 Partial Content` — fine for images, but worth noting if you ever store large assets (PDF, audio).
- No `Content-Length` expectation documented; no checksum (`Content-Digest`, RFC 9530) support.

**Recommendations**

- Fix `image/svg` → `image/svg+xml`.
- Add `ETag` and `Cache-Control: private, max-age=…` guidance to the binary GET.
- For upload robustness, accept `Content-Digest: sha-256=:…:` optionally and reject mismatches with `400`.
- Consider a single-step upload via `multipart/form-data` as a convenience alternative.

---

### 4.15 Auth Endpoints

- `/auth/callback` declares `code` and `state` as `in: path`.
- `/auth/login` declares `redirect_to` and `provider` as `in: path`.

OIDC implementations almost universally pass these as **query parameters**, and inspecting the routing shape of `/auth/callback` (no path segments for them) confirms they cannot be path parameters. Either:

- The OpenAPI generator output is wrong, or
- The API is implemented unconventionally.

If the former (most likely), this is a spec bug that makes generated clients and mock servers diverge from reality.

**Recommendation**

- Change `in: path` → `in: query` for `code`, `state`, `redirect_to`, `provider`.
- Mark `provider` with an enum of supported providers.
- Document `redirect_to` validation (allow-list) — open redirectors are a classic auth bug.

Also: `/auth/otp/verify` returns `200` with `Session` and is then supposed to set a cookie. Document the `Set-Cookie` header in the response (`headers:` section) so SDKs can treat it correctly.

---

### 4.16 Security, CSRF, Rate Limits

- `SessionCookie` (apiKey in cookie) and `SessionToken` (apiKey in `Authorization` header). The latter is effectively bearer-style; using **`type: http, scheme: bearer`** is slightly more semantic and unlocks OpenAPI tooling (Swagger UI "Authorize" UX).
- CSRF is discussed in prose. Consider defining a `CsrfToken` security scheme or explicitly marking which operations require a CSRF header when cookie auth is used.
- `429` is documented on `/auth/login` and `/auth/otp/request`, but **not on any `/api/v1/*` operation**. If the API is rate-limited (it almost certainly is), document that uniformly plus `Retry-After` and `X-RateLimit-*` response headers.

---

### 4.17 Dates, Numbers, Constraints

- `date-time` used consistently. **Document the timezone policy** ("all timestamps are UTC, rendered with `Z` suffix") so clients don't guess.
- IDs are `type: string` with no `format`/`pattern`. Pick one (`uuid`, `ulid`, or an opaque printable subset) and declare it: `{ type: string, format: uuid }`. Helps codegen, validation, and fuzzing.
- `tempo`: integer, no unit. Document "BPM, integer".
- `time`: `integer[]`, undocumented — presumably time signature `[numerator, denominator]`. Either document that or upgrade to `{ numerator, denominator }`.
- `languages`: `string[]` — document whether these are **BCP 47**, **ISO 639-1**, or free-form. Add `pattern` or `enum`.
- `SongLink.key`: musical key, undocumented format. Document the grammar ("note (`A`-`G`), optional accidental (`#`/`b`), optional mode (`m`/`maj`/`min`)").
- No `maxLength` on free-text fields (`title`, `ocr`, `copyright`, `subtitle`). Add sane caps — helps validation and protects against DoS.
- No `maxItems` on arrays (`blobs`, `songs`, `members`). Same reasoning.

---

### 4.18 Domain Fields Without Documentation

Beyond §4.17, several fields carry product-specific meaning that the spec does not explain:

- `Song.not_a_song: boolean` — the name reads like a hack; the docs don't explain why a `/songs` resource has a "this isn't a song" flag.
- `Song.data: object` — untyped. `PatchSongData` has much more structure; expose it as `SongData` for GET/POST so generated clients aren't stuck with `unknown`.
- `Song.user_specific_addons: { liked: bool }` — good pattern but the name `user_specific_addons` is awkward. Consider `personalization` or inlining `liked` at the top level.
- `Collection.cover: string` — blob ID? URL? Tell the reader.
- `Player.scroll_type_cache_other_orientation` — leaking an internal cache concept to the public API. Consider computing it server-side and exposing a cleaner derived property, or dropping it.
- `Player.between_items: bool` — meaning not documented.
- `Blob.ocr: string` — OCR'd text from the blob image, presumably. Document explicitly; also consider whether it should be `string | null` (empty string vs no OCR yet).
- `TocItem.nr: string` — song number? chapter number? Document.
- `TocItem.id: string | null` — nullable because…? Document when it's null.
- `User.request_count: int64` — exposing a usage counter on the public user object is unusual. Intentional?

---

### 4.19 OpenAPI Hygiene

- No `info.contact`, no `info.termsOfService`. Nice-to-have.
- `servers` has only `/`. Add staging/production if both are real.
- The intro says *"See schema example fields on core DTOs in the components section"*, but the component schemas don't carry `example` / `examples`. Either add them or remove the promise.
- No `externalDocs` per operation or tag.
- Descriptions are generally good; a few operations have only a title ("Creates a new user") — add the interesting details (side effects, emitted events, idempotency).
- **Spectral** (the community OpenAPI linter) would catch most of the issues in this review automatically; adding it to CI prevents regression.

---

### 4.20 Missing / Asymmetric Operations

- **Invitation decline**: there is `.../accept` but no `.../decline`. Clients have to `DELETE` via a different path.
- **Song likes discoverability**: you can like/unlike a song and check like status, but there is no `GET /users/me/liked-songs`. For UX, that's usually important.
- **Team membership**: `POST /teams` and `PATCH /teams/{id}` can change the full membership list, but there is no `POST /teams/{id}/members` / `DELETE /teams/{id}/members/{user_id}` for single-member moves. "Replace-all" and "per-member" endpoints typically coexist.
- **Bulk operations**: no `POST /songs/batch` or equivalent. Fine if not in scope, but worth a note.
- **Export / download**: no "export all my data" endpoint — worth considering for GDPR-like requests.
- **Search**: `q` is declared "Reserved" on several sub-resources; if search will never live there, remove the parameter; if it will, schedule it.

---

## 5. Prioritized Action List

### Ship now (spec-only, no server changes)

1. **Unify error handling**: pick `ProblemDetails` as the one schema, reference it from every 4xx/5xx, and change the content type to `application/problem+json`. Drop or deprecate `ErrorResponse`.
2. **Fix auth parameter locations**: `in: path` → `in: query` for `code`, `state`, `redirect_to`, `provider`.
3. **Fix `page_size.minimum`**: `0` → `1`. Add `maximum: 500`.
4. **Fix MIME**: `image/svg` → `image/svg+xml`.
5. **Drop the `required: ["data"]`** on `PatchSong` (and audit sibling `Patch*`).
6. **Add missing responses**: `404` on `PUT /setlists/{id}`; audit every mutating endpoint for `403`, `412`, `409`.
7. **Rename `CreateUserRequest` → `CreateUser`**.
8. **Document domain fields**: `not_a_song`, `Player.between_items`, `Player.scroll_type_cache_other_orientation`, `TocItem.nr`, `SongLink.key`, units for `tempo`, format for `languages`.
9. **Remove unused `q` parameters** from sub-resources that say "Reserved".
10. **Add timezone / ID format documentation** in the intro.

### Ship this quarter (minor breaking but scoped)

11. **Expose `SongData`** as a real schema instead of `object`.
12. **Standardize enum casing** on `lower_snake_case`; accept both old and new values on input; emit new on output.
13. **Define and implement `If-Match` / `412 Precondition Failed`** on all mutating endpoints that support ETag.
14. **Add `Link` pagination headers**; keep `X-Total-Count`.
15. **Define one `sort` syntax** (recommend `sort=-id,title` style) and apply it everywhere.
16. **Make invitation acceptance symmetric** (`POST /teams/{team_id}/invitations/{invitation_id}/accept`).
17. **Add rate-limit documentation** (`429`, `Retry-After`, `X-RateLimit-*`) to `/api/v1/*`.
18. **Add `ETag`, `Cache-Control` on blob `GET … /data`**.

### Schedule for v2 (breaking)

19. **Redesign `PlayerItem`** using internally-tagged `{ type, … }` + a `discriminator`.
20. **Unify reference shapes** across the API with `{ id, … }` links; add `?expand=` for optional embedding. Migrate `Song.blobs` to `BlobLink[]`.
21. **Drop `error` legacy alias** from `ProblemDetails`.
22. **Drop `PascalCase` from `PlayerItem` and any other key** that doesn't follow `snake_case`.
23. **Consolidate create/update types** (rename `CreateUserRequest`, unify `Create*`/`Update*` where semantics match).

---

## 6. Appendix A — Schema Diffs (Illustrative)

### 6.1 `Song.blobs` → `Song.blob_links`

**Before**

```json
"Song": {
  "required": ["id","owner","not_a_song","blobs","data","user_specific_addons"],
  "properties": {
    "blobs": { "type": "array", "items": { "type": "string" } }
  }
}
```

**After (non-breaking additive)**

```json
"Song": {
  "required": ["id","owner","not_a_song","blobs","data","user_specific_addons"],
  "properties": {
    "blobs":      { "type": "array", "items": { "type": "string" },
                    "deprecated": true,
                    "description": "Deprecated; use blob_links. Will be removed in v2." },
    "blob_links": { "type": "array", "items": { "$ref": "#/components/schemas/BlobLink" } }
  }
},
"BlobLink": {
  "type": "object",
  "required": ["id"],
  "additionalProperties": false,
  "properties": { "id": { "type": "string" } }
}
```

### 6.2 Error envelope

**Before (split)**

```json
"ErrorResponse":   { "required": ["code","error"],                 "properties": { "code":{}, "error":{} } }
"ProblemDetails":  { "required": ["type","title","status","detail","code","error"], ... }
```

**After (single)**

```json
"Problem": {
  "type": "object",
  "required": ["type","title","status","code"],
  "properties": {
    "type":     { "type": "string", "format": "uri" },
    "title":    { "type": "string" },
    "status":   { "type": "integer", "minimum": 400, "maximum": 599 },
    "code":     { "type": "string", "description": "Stable machine-readable code" },
    "detail":   { "type": "string" },
    "instance": { "type": "string", "format": "uri" }
  },
  "additionalProperties": true
}
```

All error responses:

```json
"400": {
  "description": "Validation failure",
  "content": { "application/problem+json": { "schema": { "$ref": "#/components/schemas/Problem" } } }
}
```

### 6.3 `PlayerItem`

**Before**

```json
"PlayerItem": {
  "oneOf": [
    { "type": "object", "required": ["Blob"],   "properties": { "Blob":   { "type": "string" } } },
    { "type": "object", "required": ["Chords"], "properties": { "Chords": { "$ref": "#/components/schemas/Song" } } }
  ]
}
```

**After**

```json
"PlayerItem": {
  "oneOf": [
    { "$ref": "#/components/schemas/PlayerBlobItem" },
    { "$ref": "#/components/schemas/PlayerChordsItem" }
  ],
  "discriminator": {
    "propertyName": "type",
    "mapping": {
      "blob":   "#/components/schemas/PlayerBlobItem",
      "chords": "#/components/schemas/PlayerChordsItem"
    }
  }
},
"PlayerBlobItem": {
  "type": "object",
  "required": ["type","blob_id"],
  "additionalProperties": false,
  "properties": {
    "type":    { "type": "string", "enum": ["blob"] },
    "blob_id": { "type": "string" }
  }
},
"PlayerChordsItem": {
  "type": "object",
  "required": ["type","song"],
  "additionalProperties": false,
  "properties": {
    "type": { "type": "string", "enum": ["chords"] },
    "song": { "$ref": "#/components/schemas/Song" }
  }
}
```

### 6.4 `page_size`

**Before**

```json
{ "name": "page_size", "schema": { "type": "integer", "nullable": true, "minimum": 0 } }
```

**After**

```json
{ "name": "page_size",
  "description": "Items per page. Must be 1–500. Defaults to 50.",
  "schema": { "type": "integer", "nullable": true, "minimum": 1, "maximum": 500, "default": 50 } }
```

---

## 7. Appendix B — Checklist Against Common Standards

| Standard / Guideline | Status |
|---|---|
| OpenAPI 3.0.3 validity | ✅ Valid |
| Consistent JSON property casing | ⚠️ Mostly snake_case; `PlayerItem` breaks |
| Consistent enum value casing | ❌ Four styles |
| Single error model | ❌ Two schemas in use |
| RFC 7807 media type (`application/problem+json`) | ❌ Not served |
| Stable error code catalog | ⚠️ Promised, not listed |
| ETag + `If-None-Match` (reads) | ✅ On singletons |
| ETag + `If-Match` + 412 (writes) | ❌ Missing |
| 422 vs 400 distinction | ❌ All collapsed to 400 |
| Pagination `Link` header (RFC 5988) | ❌ Missing |
| Pagination consistency | ⚠️ Two semantics coexist |
| Rate-limit headers / 429 on API | ❌ Auth only |
| `Set-Cookie` modeled in responses | ❌ Not declared |
| Binary caching headers | ❌ Not declared |
| Correct MIME for SVG | ❌ `image/svg` → should be `image/svg+xml` |
| `discriminator` on polymorphic unions | ❌ Missing on `PlayerItem` |
| `additionalProperties: false` on inputs | ✅ |
| `format` on IDs | ❌ All untyped strings |
| Timezone policy documented | ❌ |
| CI linting (Spectral or equivalent) | ❓ Not visible; recommend adding |

---

### TL;DR on the blobs question

Bare `string[]` of IDs **is fine** if the relation is pure foreign keys. What *isn't* fine is using bare strings for `Song → Blob` while using `SongLink` objects for `Collection → Song` and full objects for `Session → User`. Either rename the field to `blob_ids` and keep it, or ship a `BlobLink = { id }` alongside and migrate. The consistency is the win; the specific shape is secondary.
