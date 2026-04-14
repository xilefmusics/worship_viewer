# SurrealDB 3.0 Upgrade Guide for worship_viewer

> **Date:** April 2026
> **Current setup:** SurrealDB server v2.4.0-dev (README) · Rust SDK `surrealdb` crate v2.6.5 · 23 migration files
> **Target:** SurrealDB server v3.0.5 · Rust SDK `surrealdb` crate v3.0.5 + `surrealdb-types` v3.0.5

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current Setup Inventory](#current-setup-inventory)
3. [What Changed in SurrealDB 3.0](#what-changed-in-surrealdb-30)
4. [Impact Analysis for worship_viewer](#impact-analysis-for-worship_viewer)
   - [Will Break (Critical)](#will-break--critical)
   - [Won't Break (Safe)](#wont-break--safe)
5. [Rust SDK Migration](#rust-sdk-migration)
   - [Cargo.toml Changes](#cargotoml-changes)
   - [Type System: surrealdb-types & SurrealValue](#type-system-surrealdb-types--surrealvalue)
   - [Import Path Changes](#import-path-changes)
6. [SurrealQL Migration](#surrealql-migration)
   - [Function Renames](#function-renames)
   - [SEARCH ANALYZER → FULLTEXT ANALYZER](#search-analyzer--fulltext-analyzer)
   - [Other Query Changes](#other-query-changes)
7. [Migration File Strategy](#migration-file-strategy)
8. [Step-by-Step Upgrade Plan](#step-by-step-upgrade-plan)
9. [Testing Strategy](#testing-strategy)
10. [Risk Assessment](#risk-assessment)
11. [New 3.0 Features Worth Adopting](#new-30-features-worth-adopting)

---

## Executive Summary

**Can you easily upgrade? Yes, with moderate effort.**

worship_viewer has **no** usage of futures, stored closures, record references (the new `<~` syntax), `ANALYZE` statements, like operators (`~`/`!~`), or `GROUP + SPLIT` combos — the features that would require architectural rework. The breaking changes that *do* affect this project are **mechanical renames** that can be addressed with find-and-replace:

| Change | Scope | Effort |
|--------|-------|--------|
| `type::thing` → `type::record` | 4 call sites in Rust query strings | Trivial |
| `duration::from::secs` → `duration::from_secs` | 1 call site in OTP model | Trivial |
| `type::is::none` / `type::is::array` → `type::is_none` / `type::is_array` | 4 occurrences across 3 migration files | Trivial |
| `SEARCH ANALYZER` → `FULLTEXT ANALYZER` | 7 index definitions across 2 migration files | Trivial |
| Rust SDK crate bump 2.6.5 → 3.0.5 | Cargo.toml + add `surrealdb-types` | Small |
| `surrealdb::sql::Thing` / `Datetime` import paths | ~30 files use `Thing`, ~8 use `Datetime` | Small-Medium |
| `Serialize`/`Deserialize` → optional `SurrealValue` derive | All record structs (optional, serde still works) | Optional |
| Docker image `surrealdb/surrealdb:v2.4.0-dev` → `v3.0.5` | README, deployment | Trivial |

Estimated total effort: **1–2 days** including testing.

---

## Current Setup Inventory

### Server

- **Docker image (README):** `surrealdb/surrealdb:v2.4.0-dev`
- **CI/tests:** In-memory engine via `kv-mem` feature (no external SurrealDB server)
- **No** `docker-compose.yml` or `surreal.toml` in the repo

### Rust SDK

```toml
# backend/Cargo.toml
surrealdb = { version = "2.6.5", default-features = false, features = [
    "kv-mem",
    "protocol-http",
    "protocol-ws",
    "rustls",
] }
```

### Connection & Auth

- Uses `surrealdb::engine::any::connect` with configurable `DB_ADDRESS` (default `mem://`)
- Database-level auth via `surrealdb::opt::auth::Database` (`DbAuth`)
- No SurrealDB scope auth — application manages its own sessions/OIDC/OTP

### SurrealQL Features in Use

| Feature | Usage |
|---------|-------|
| `record<table>` links | Owner, blob, song, collection references on most tables |
| Full-text search | `DEFINE ANALYZER`, `SEARCH ANALYZER`, `search::score()`, `@N@` match operator |
| `DEFINE EVENT` | Cascade deletes across user/team/blob/song/collection/setlist/like/session |
| `DEFINE FUNCTION` | Migration helpers (chordlib conversion, song validation) |
| `type::thing()` | OTP, session, collection queries |
| `duration::from::secs()` | OTP expiration |
| `type::is::none()` / `type::is::array()` | Field value guards in migrations |
| `BEGIN TRANSACTION` / `COMMIT TRANSACTION` | Migration runner wraps each `.surql` file |
| `LET`, `FOR`, `IF/ELSE`, `UPSERT`, `UPDATE ... SET ... WHERE` | Various model queries |
| `FETCH` | Fetching linked records (songs, users, teams) |
| Embedded arrays of objects | Setlist songs, team members |

### Features NOT in Use (No Migration Concern)

- No `<future>` types or `COMPUTED` fields
- No stored closures in records
- No record references (`<~` syntax)
- No `RELATE` / graph edges
- No `ANALYZE` statement
- No like operators (`~`, `!~`, `?~`, `*~`)
- No `GROUP + SPLIT` together
- No `MTREE` indexes
- No `--strict` flag
- No `array::range` calls
- No `.*` idiom dereferences
- No `$val = 10` without `LET`

---

## What Changed in SurrealDB 3.0

SurrealDB 3.0 was released on February 17, 2026. The major themes are:

### Architecture & Performance
- **New query planner:** Internal streaming execution for better performance
- **New in-memory engine:** Lock-free, MVCC-based design (replaces old `kv-mem`)
- **ID-based storage:** Internal storage model moved to ID-based keys
- **Synced writes by default:** Improved durability guarantees
- **Memory-bounded LRU cache** for HNSW vectors

### New Features
- **Extensions (Surrealism):** WASM plugins for custom logic inside SurrealQL
- **File storage:** `DEFINE BUCKET` + file pointers for in-DB file handling
- **Custom API endpoints:** `DEFINE API` for HTTP routes + middleware in SurrealQL
- **Client-side transactions:** Manage TX flow from application code
- **Record references (new):** Bidirectional `REFERENCE` tracking with `<~` operator
- **GraphQL support (stable):** No longer experimental
- **Async events:** `DEFINE EVENT ... ASYNC` for non-blocking event execution

### Breaking Changes (Full List)

| # | Change | Severity |
|---|--------|----------|
| 1 | Futures removed → `COMPUTED` fields | Will break |
| 2 | Function name renames (`::from::` → `::from_`, `type::thing` → `type::record`, etc.) | Will break |
| 3 | `array::range` argument semantics changed | Will break |
| 4 | `LET` required for parameter declarations | Will break |
| 5 | `GROUP` + `SPLIT` can't be used together | Will break |
| 6 | Like operators (`~`, `!~`, `?~`, `*~`) removed | Will break |
| 7 | `SEARCH ANALYZER` → `FULLTEXT ANALYZER` | Will break |
| 8 | Database-level strictness (no more `--strict` flag) | Will break |
| 9 | `MTREE` removed → `HNSW` | Will break |
| 10 | Stored closures in records removed | Will break |
| 11 | Record reference syntax changed | Will break |
| 12 | `ANALYZE` statement removed | Will break |
| 13 | `.*` idiom behavior changed | Can break |
| 14 | Field idiom on arrays changed | Can break |
| 15 | Idiom fetching changes | Can break |
| 16 | Optional operator `?` → `.?` | Can break |
| 17 | Parsing changes (record IDs, unicode, escaping) | Can break |
| 18 | Set type now orders + deduplicates | Can break |
| 19 | SCHEMAFULL tables reject unknown fields | Can break |
| 20 | Numeric record ID ordering changed | Can break |
| 21-29 | Various edge cases (math::sqrt NaN, mock type, etc.) | Unlikely break |

### Rust SDK Changes
- New `surrealdb-types` crate with `SurrealValue` trait (replaces serde for DB serialization)
- Types like `Thing`, `Datetime`, `RecordId` moved to / re-exported from `surrealdb-types`
- `kind!` macro for SurrealQL type matching
- Feature flags remain the same: `kv-mem`, `protocol-ws`, `protocol-http`, `rustls`
- SDK requires Rust 1.89+

---

## Impact Analysis for worship_viewer

### Will Break — Critical

These are the changes that **will** cause compilation or runtime errors if not addressed:

#### 1. Function Renames in SurrealQL Strings

**`type::thing` → `type::record`** — 4 occurrences in Rust code:

| File | Line | Current |
|------|------|---------|
| `backend/src/auth/otp/model.rs` | 19 | `type::thing('otp', $email)` |
| `backend/src/auth/otp/model.rs` | 47 | `type::thing('otp', $email)` |
| `backend/src/resources/collection/model.rs` | 225 | `type::thing("collection", $id)` |
| `backend/src/resources/user/session/model.rs` | 56 | `type::thing("session", $id)` |

**`duration::from::secs` → `duration::from_secs`** — 1 occurrence:

| File | Line | Current |
|------|------|---------|
| `backend/src/auth/otp/model.rs` | 23 | `duration::from::secs($ttl_secs)` |

**`type::is::none` → `type::is_none`** — 2 occurrences in migration files:

| File | Line | Current |
|------|------|---------|
| `backend/db-migrations/20260413120000_team.surql` | 7 | `type::is::none($value)` |
| `backend/db-migrations/20240301000009_user.surql` | 18 | `type::is::none($value)` |

**`type::is::array` → `type::is_array`** — 2 occurrences:

| File | Line | Current |
|------|------|---------|
| `backend/db-migrations/20260409120000_song_chordlib07_schema.surql` | 20 | `type::is::array($t)` |
| `backend/db-migrations/20260409120000_song_chordlib07_schema.surql` | 32 | `type::is::array($t)` |

#### 2. SEARCH ANALYZER → FULLTEXT ANALYZER

7 index definitions across 2 migration files:

**`backend/db-migrations/20260328000002_fulltext_search.surql`** (5 indexes):
```
SEARCH ANALYZER text_search BM25  →  FULLTEXT ANALYZER text_search BM25
```

**`backend/db-migrations/20260410140000_song_chordlib08_drop_scalar_metadata.surql`** (2 indexes):
```
SEARCH ANALYZER text_search BM25  →  FULLTEXT ANALYZER text_search BM25
```

#### 3. Rust SDK Import Paths

The `surrealdb::sql::Thing` and `surrealdb::sql::Datetime` types are re-exported in 3.0 but the canonical location is now `surrealdb_types`. Files affected:

| Type | Files using it |
|------|---------------|
| `Thing` | `session/model.rs`, `song/model.rs`, `database/mod.rs`, `oidc/model.rs`, `user/model.rs`, `team/model.rs`, `team/invitation_model.rs`, `collection/model.rs`, `blob/model.rs`, `setlist/model.rs` |
| `Datetime` | `session/model.rs`, `oidc/model.rs`, `user/model.rs`, `blob/model.rs`, `team/invitation_model.rs` |
| `Id` | `song/model.rs`, `database/mod.rs` |

> **Note:** The 3.0 SDK still re-exports these types from `surrealdb::sql`, so existing imports *may* continue to work. However, it's recommended to migrate to `surrealdb_types` imports for forward compatibility.

### Won't Break — Safe

These are 3.0 breaking changes that **do not affect** this project:

| Change | Why it's safe |
|--------|--------------|
| Futures → COMPUTED | No futures used anywhere |
| Stored closures removed | No closures stored in records |
| Record references syntax | No `<~` or `REFERENCE` usage |
| `ANALYZE` removed | Never used |
| Like operators removed | Never used |
| `GROUP + SPLIT` ban | Never combined |
| `MTREE` removal | No vector indexes |
| `--strict` flag | Not used |
| `array::range` changes | Not called |
| `LET` required | Already use `LET` everywhere |
| `.*` idiom changes | Not used in this pattern |
| Optional `?` → `.?` | Not used |
| Set type behavior | No `<set>` types |
| SCHEMAFULL unknown fields | Not creating records with extra fields in code |

---

## Rust SDK Migration

### Cargo.toml Changes

```toml
# BEFORE (v2)
surrealdb = { version = "2.6.5", default-features = false, features = [
    "kv-mem",
    "protocol-http",
    "protocol-ws",
    "rustls",
] }

# AFTER (v3)
surrealdb = { version = "3.0.5", default-features = false, features = [
    "kv-mem",
    "protocol-http",
    "protocol-ws",
    "rustls",
] }
surrealdb-types = "3.0.5"
```

The feature flags are the same in 3.0. The `kv-mem` feature now uses the new lock-free MVCC in-memory engine, which is a transparent upgrade.

> **Rust version requirement:** The 3.0 SDK requires Rust 1.89+. Check your toolchain with `rustc --version`. The project uses `edition = "2024"` which already implies a recent enough compiler.

### Type System: surrealdb-types & SurrealValue

The biggest Rust-side change is the new `surrealdb-types` crate with the `SurrealValue` derive macro. In v2, your record structs use `serde::Serialize`/`Deserialize`:

```rust
// v2 approach (still works in v3, but no longer canonical)
#[derive(Serialize, Deserialize)]
struct SongRecord {
    id: Option<Thing>,
    owner: Option<Thing>,
    // ...
}
```

In v3, you can optionally derive `SurrealValue` instead of or alongside serde:

```rust
// v3 approach (recommended for new code)
use surrealdb_types::SurrealValue;

#[derive(Debug, SurrealValue)]
struct SongRecord {
    id: Option<Thing>,
    owner: Option<Thing>,
    // ...
}
```

**However, serde-based serialization still works.** The SDK maintains backward compatibility. You can migrate to `SurrealValue` incrementally or not at all — it's optional.

### Import Path Changes

Types have moved. The recommended migration:

```rust
// BEFORE (v2)
use surrealdb::sql::{Thing, Datetime, Id};
use surrealdb::opt::auth::Database as DbAuth;
use surrealdb::engine::any::{Any, connect};

// AFTER (v3) — option A: use re-exports (minimal change)
use surrealdb::sql::{Thing, Datetime, Id};  // still works via re-export
use surrealdb::opt::auth::Database as DbAuth;
use surrealdb::engine::any::{Any, connect};

// AFTER (v3) — option B: use canonical paths (recommended)
use surrealdb_types::{Thing, Datetime, Id};
use surrealdb::opt::auth::Database as DbAuth;
use surrealdb::engine::any::{Any, connect};
```

The connection setup (`connect`, `signin`, `use_ns`, `use_db`, `query`, `create`, `select`, `update`, `delete`) API surface is unchanged between v2 and v3.

---

## SurrealQL Migration

### Function Renames

All occurrences to change in **Rust source files** (query strings):

```diff
# backend/src/auth/otp/model.rs (remember_otp query)
- LET $thing = type::thing('otp', $email);
+ LET $thing = type::record('otp', $email);

- expires_at: time::now() + duration::from::secs($ttl_secs),
+ expires_at: time::now() + duration::from_secs($ttl_secs),

# backend/src/auth/otp/model.rs (validate_otp query)
- LET $thing = type::thing('otp', $email);
+ LET $thing = type::record('otp', $email);

# backend/src/resources/collection/model.rs
- UPDATE type::thing("collection", $id)
+ UPDATE type::record("collection", $id)

# backend/src/resources/user/session/model.rs
- LET $sid = type::thing("session", $id);
+ LET $sid = type::record("session", $id);
```

All occurrences to change in **migration files**:

```diff
# backend/db-migrations/20260413120000_team.surql
- VALUE IF type::is::none($value) THEN NONE ELSE ...
+ VALUE IF type::is_none($value) THEN NONE ELSE ...

# backend/db-migrations/20240301000009_user.surql
- VALUE IF type::is::none($value) THEN NONE ELSE ...
+ VALUE IF type::is_none($value) THEN NONE ELSE ...

# backend/db-migrations/20260409120000_song_chordlib07_schema.surql (2 occurrences)
- IF type::is::array($t) AND ...
+ IF type::is_array($t) AND ...
```

### SEARCH ANALYZER → FULLTEXT ANALYZER

All occurrences:

```diff
# backend/db-migrations/20260328000002_fulltext_search.surql (5 indexes)
- FIELDS data.title SEARCH ANALYZER text_search BM25;
+ FIELDS data.title FULLTEXT ANALYZER text_search BM25;
# (repeat for all 5 DEFINE INDEX statements)

# backend/db-migrations/20260410140000_song_chordlib08_drop_scalar_metadata.surql (2 indexes)
- FIELDS data.titles SEARCH ANALYZER text_search BM25;
+ FIELDS data.titles FULLTEXT ANALYZER text_search BM25;
# (repeat for both DEFINE INDEX statements)
```

### Other Query Changes

**No other SurrealQL changes required.** Specifically:

- `search::score()` is unchanged
- The `@N@` match operator is unchanged
- `DEFINE EVENT ... WHEN ... THEN` syntax is unchanged
- `DEFINE FUNCTION` syntax is unchanged
- `FETCH` clause is unchanged
- `LET`, `FOR`, `IF/ELSE`, `UPSERT`, `BEGIN TRANSACTION / COMMIT TRANSACTION` are unchanged
- `record<table>` type syntax is unchanged
- `array::append`, `array::len`, `array::flatten`, `array::filter` are unchanged
- `string::join`, `string::len` are unchanged
- `time::now()` is unchanged
- `object::from_entries` is unchanged

---

## Migration File Strategy

The migration system applies `.surql` files sequentially and tracks them by filename + checksum. Since migration files are **already applied** data, there are two strategies:

### Strategy A: Modify Existing Migrations (Recommended for Fresh Deploys)

If you only run against fresh databases (e.g., tests use `mem://`, no persistent production data), you can modify the migration files in-place. The migration runner checks checksums, but since this is always a fresh database, there's no conflict.

**This is the recommended approach** since:
- CI runs against `mem://` (fresh each time)
- The README instructs local dev to start with a fresh `memory` database
- If there is no persistent production database yet, there's nothing to migrate

### Strategy B: Add a New Migration (Required if Production Data Exists)

If you have a persistent production database with data, you **cannot** modify already-applied migrations (the checksum check would fail). Instead:

1. Keep old migrations untouched
2. Add a new migration file (e.g., `20260414000000_surrealdb_v3_compat.surql`) that redefines all affected objects:

```sql
-- Redefine indexes with FULLTEXT ANALYZER
DEFINE INDEX OVERWRITE song_data_titles_search_idx ON song
    FIELDS data.titles FULLTEXT ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE song_data_artists_search_idx ON song
    FIELDS data.artists FULLTEXT ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE song_content_search_idx ON song
    FIELDS search_content FULLTEXT ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE setlist_title_search_idx ON setlist
    FIELDS title FULLTEXT ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE collection_title_search_idx ON collection
    FIELDS title FULLTEXT ANALYZER text_search BM25;

-- Redefine fields using renamed functions
DEFINE FIELD OVERWRITE owner ON team TYPE option<record<user>>
    VALUE IF type::is_none($value) THEN NONE ELSE $value ?? $before ?? NONE END;

DEFINE FIELD OVERWRITE default_collection ON user TYPE option<record<collection>>
    VALUE IF type::is_none($value) THEN NONE ELSE $value ?? $before ?? NONE END;
```

For the DEFINE FUNCTION statements in migration 20260409120000, since they're helpers used during that migration's `UPDATE` statements and not called at runtime, they don't need to be redefined — they were only used once.

### Data Export/Import Alternative

If you have a persistent v2 database, SurrealDB provides tooling:

1. **Surrealist migration diagnostics** (requires SurrealDB >= 2.6.1): visual tool that scans your schema and flags issues
2. **CLI export** (requires SurrealDB >= 3.0.3):
   ```bash
   surreal v2 export --v3 \
     --namespace app --database app \
     --endpoint ws://localhost:8000 \
     --token <token> \
     v2_export.surql
   ```
   This auto-converts `SEARCH ANALYZER` → `FULLTEXT ANALYZER` and renames functions.
3. Import into v3:
   ```bash
   surreal import \
     --namespace app --database app \
     --endpoint ws://localhost:8000 \
     --token <token> \
     v2_export.surql
   ```

---

## Step-by-Step Upgrade Plan

### Phase 1: Prepare (on a feature branch)

1. **Update Rust toolchain** to >= 1.89
   ```bash
   rustup update stable
   ```

2. **Update Cargo.toml**
   ```toml
   surrealdb = { version = "3.0.5", default-features = false, features = [
       "kv-mem",
       "protocol-http",
       "protocol-ws",
       "rustls",
   ] }
   surrealdb-types = "3.0.5"
   ```

3. **Run `cargo update`** to resolve the full dependency tree

### Phase 2: Fix Rust Compilation

4. **Update imports** (minimal approach — update only if `surrealdb::sql::*` no longer compiles):
   ```rust
   // If needed, change:
   use surrealdb::sql::{Thing, Datetime, Id};
   // To:
   use surrealdb_types::{Thing, Datetime, Id};
   ```

5. **Check for any API changes** in `.query()`, `.create()`, `.select()`, `.update()`, `.delete()` — these are expected to be compatible but verify with `cargo check`.

### Phase 3: Fix SurrealQL Queries

6. **Rename functions in Rust query strings:**
   - `type::thing` → `type::record` (4 sites)
   - `duration::from::secs` → `duration::from_secs` (1 site)

7. **Rename functions in migration files:**
   - `type::is::none` → `type::is_none` (2 sites)
   - `type::is::array` → `type::is_array` (2 sites)

8. **Update index syntax in migration files:**
   - `SEARCH ANALYZER` → `FULLTEXT ANALYZER` (7 sites)

### Phase 4: Update Docker / Deployment

9. **Update README** Docker command:
   ```bash
   # BEFORE
   docker run --rm -p 8000:8000 surrealdb/surrealdb:v2.4.0-dev start --log debug --user root --pass root memory
   
   # AFTER
   docker run --rm -p 8000:8000 surrealdb/surrealdb:v3.0.5 start --log debug --user root --pass root memory
   ```

10. **Update any deployment scripts** / Dockerfiles referencing the SurrealDB server version.

### Phase 5: Verify

11. **Run `cargo build`** — should compile cleanly
12. **Run `cargo test`** — all tests against in-memory engine
13. **Run `cargo clippy`** — no new warnings
14. **Manual smoke test** with the full stack:
    - Start SurrealDB v3.0.5 server
    - Run the backend
    - Verify CRUD for songs, setlists, collections, blobs
    - Verify full-text search works
    - Verify OTP creation/validation
    - Verify team management and cascades
    - Verify OIDC flow

---

## Testing Strategy

### Automated

All existing tests run against `mem://` with a fresh database each time. After applying the changes, `cargo test` should catch:
- Query syntax errors (renamed functions)
- SDK API incompatibilities
- Serialization/deserialization mismatches

### Manual Verification Checklist

- [ ] Create a song → verify data persisted correctly
- [ ] Search songs by title/artist → verify `FULLTEXT ANALYZER` indexes work
- [ ] Search setlists and collections → verify search scores
- [ ] Create/delete a user → verify cascade events fire
- [ ] Create/delete a team → verify content reassignment
- [ ] OTP flow → verify `type::record` and `duration::from_secs` work
- [ ] Session middleware → verify `type::record` session lookup
- [ ] Collection song append → verify `type::record` in UPDATE
- [ ] Blob upload/download → verify record creation and retrieval

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Renamed functions missed somewhere | Low | Medium (runtime error) | Grep for all old names before merge |
| SDK import paths break | Low | Low (compile-time) | `cargo check` catches immediately |
| In-memory engine behavior differs | Very Low | Medium | Full test suite covers this |
| Migration checksum mismatch (production) | Medium (if prod exists) | High | Use Strategy B or export/import |
| `search::score` / `@N@` behavior change | Very Low | Medium | Not documented as changed; test search |
| `DEFINE EVENT` behavior differs | Very Low | Low | Events syntax unchanged, new `ASYNC` is opt-in |
| SCHEMAFULL strictness rejects data | Very Low | Medium | Tables aren't SCHEMAFULL with unexpected fields |
| Performance regression | Very Low | Low | 3.0 is generally faster; monitor |

**Overall risk: Low.** The changes are mechanical and well-scoped.

---

## New 3.0 Features Worth Adopting

These are optional improvements that could benefit the project post-upgrade:

### 1. Async Events

Current events run synchronously in the triggering transaction. 3.0 adds `ASYNC` for non-blocking execution:

```sql
DEFINE EVENT OVERWRITE team_personal_blob_cascade ON team
    ASYNC
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN (DELETE blob WHERE owner = $before.id);
```

**Benefit:** Cascade deletes won't slow down the parent DELETE operation.

### 2. Client-Side Transactions

Instead of wrapping queries in `BEGIN TRANSACTION` / `COMMIT TRANSACTION` strings, 3.0 supports transactions at the SDK level:

```rust
let tx = db.transaction().await?;
tx.query("UPDATE ...").await?;
tx.query("DELETE ...").await?;
tx.commit().await?;
```

**Benefit:** Better error handling and more natural Rust control flow.

### 3. Custom API Endpoints (`DEFINE API`)

For simple CRUD operations, you could define API endpoints directly in SurrealDB:

```sql
DEFINE API "/songs/search" FOR get
    MIDDLEWARE api::timeout(100ms)
    THEN {
        { body: SELECT * FROM song WHERE data.titles @0@ $request.query.q }
    };
```

**Benefit:** Could simplify the Actix REST layer for certain endpoints. However, this would be a larger architectural change and may not be desirable for a project that already has a well-structured REST API.

### 4. Record References

3.0 introduces bidirectional references with `REFERENCE`:

```sql
DEFINE FIELD OVERWRITE owner ON song TYPE record<team> REFERENCE;
-- Now you can query: SELECT <~song FROM team:xxx
```

**Benefit:** Could replace some of the `WHERE owner IN $teams` patterns with reverse lookups. Be aware of the [backfill bug](https://github.com/surrealdb/surrealdb/issues/7144) in v3.0.4 when adding `REFERENCE` to existing data.

### 5. File Storage (`DEFINE BUCKET`)

If blob storage is currently handled externally, 3.0's file storage could potentially consolidate it:

```sql
DEFINE BUCKET song_files;
-- Store files directly in SurrealDB
```

**Benefit:** Fewer moving parts. Worth evaluating depending on current blob storage architecture.

### 6. SurrealValue Derive Macro

Replace serde with native SurrealDB serialization for record structs:

```rust
#[derive(Debug, SurrealValue)]
struct SongRecord {
    id: Option<Thing>,
    owner: Option<Thing>,
    // ...
}
```

**Benefit:** Better type safety, native SurrealQL type mapping, and forward-compatible with future SDK changes.

---

## Quick Reference: All Files to Change

```
backend/Cargo.toml                                          # bump surrealdb, add surrealdb-types
backend/src/auth/otp/model.rs                               # type::thing → type::record, duration::from::secs → duration::from_secs
backend/src/resources/collection/model.rs                   # type::thing → type::record
backend/src/resources/user/session/model.rs                 # type::thing → type::record
backend/db-migrations/20260328000002_fulltext_search.surql  # SEARCH ANALYZER → FULLTEXT ANALYZER (5x)
backend/db-migrations/20260410140000_song_chordlib08_drop_scalar_metadata.surql  # SEARCH ANALYZER → FULLTEXT ANALYZER (2x)
backend/db-migrations/20260413120000_team.surql             # type::is::none → type::is_none
backend/db-migrations/20240301000009_user.surql             # type::is::none → type::is_none
backend/db-migrations/20260409120000_song_chordlib07_schema.surql  # type::is::array → type::is_array (2x)
README.md                                                   # Docker image version
```

**Optional (import paths):**
```
backend/src/resources/song/model.rs
backend/src/resources/blob/model.rs
backend/src/resources/setlist/model.rs
backend/src/resources/team/model.rs
backend/src/resources/team/invitation_model.rs
backend/src/resources/user/model.rs
backend/src/auth/oidc/model.rs
backend/src/database/mod.rs
```
