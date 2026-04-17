# Business logic constraints for the song resource

## Static

- **BLC-SONG-001:** Every song belongs to exactly one **owning team** (**`owner`** in responses).
- **BLC-SONG-002:** Listing, single-song **GET**, player, and like endpoints require **read** access to that team’s library; **PUT** and **DELETE** require **library edit** access. Platform **admin** does **not** gain song edit solely by role.
- **BLC-SONG-003:** **`PUT`** MUST NOT change **`owner`** except where the API explicitly allows ownership moves.
- **BLC-SONG-004:** **Like** state IS per **current user** and **song**; anyone who may read the song MAY read or update likes via **GET**/**PUT** `/songs/{id}/likes`.

## List pagination and search

- **BLC-SONG-005:** **`GET /songs`** supports **`page`**, **`page_size`**, optional **`q`**, and the shared rules in [list-pagination.md](./list-pagination.md) (including whitespace-only **`q`** treated as no filter).

## When / then

- **BLC-SONG-006:** WHEN the caller may not read the owning team’s library THEN song reads respond **404** (not **403**).
- **BLC-SONG-007:** WHEN the caller is **guest** on the owning team and attempts **PUT** or **DELETE** THEN the API responds **404**.
- **BLC-SONG-008:** WHEN the caller is the personal-team **owner**, or **admin** / **content_maintainer** on the owning team, THEN **PUT**/**DELETE** are allowed (subject to validation).
- **BLC-SONG-009:** WHEN **POST** creates a song THEN **`owner`** IS ALWAYS the caller’s **personal** team.
- **BLC-SONG-010:** WHEN **POST** completes THEN IF **`default_collection`** is set THEN the new song IS appended there; OTHERWISE a **“Default”** collection is created, set as default, and the song IS placed there.
- **BLC-SONG-011:** WHEN **GET /songs** runs THEN only songs whose **`owner`** team the caller may read are returned; optional **`q`** matches **title**, **artists**, and lyric text as defined by the list-search behavior (stemmed where applicable).
- **BLC-SONG-012:** WHEN **GET /songs/{id}** runs THEN visibility matches the list rule AND the response includes **`liked`** for the current user.
- **BLC-SONG-013:** WHEN **GET …/player** runs THEN visibility matches **GET /songs/{id}**.
- **BLC-SONG-014:** WHEN **DELETE /songs/{id}** succeeds THEN the song no longer appears via the API under the same access rules as **PUT**.
- **BLC-SONG-017:** WHEN **PUT /songs/{id}** body fails validation (e.g. empty **`data`**, or wrong types for fields such as **`tempo`** / **`time`**) THEN **400**.
- **BLC-SONG-018:** WHEN **PUT /songs/{id}** uses an **`{id}`** that does not yet refer to an existing song THEN the API MAY create the song (**200**) with that **id** and **`owner`** the caller’s **personal** team, subject to **BLC-SONG-007** and **BLC-SONG-008** for **guest** vs **edit** rights on that team.

## Cascading deletes and collection/setlist references

- **BLC-SONG-015:** WHEN a song IS deleted THEN collections and setlists MAY still list its id until updated; **POST**/**PUT** MAY accept unknown ids; **GET …/collections/{id}/songs`** MAY return **500** or incomplete entries if a slot no longer resolves; clients SHOULD refresh after deletes.
- **BLC-SONG-016:** WHEN a **user** account IS deleted THEN songs owned by their **personal** team are removed with that team ([user.md](./user.md)).
