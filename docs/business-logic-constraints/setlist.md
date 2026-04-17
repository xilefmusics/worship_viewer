# Business logic constraints for the setlist resource

## Static

- **BLC-SETL-001:** Every setlist belongs to exactly one **owning team** (**`owner`** in responses).
- **BLC-SETL-002:** Reads (metadata, songs, player) require **read** access to that team’s library; **PUT** and **DELETE** require **library edit** access. Platform **admin** MAY read but MUST NOT mutate setlists solely by admin role.
- **BLC-SETL-003:** **`PUT`** MUST NOT change **`owner`**; it replaces **title**, ordered **songs**, and related fields exposed by the API.

## Create payload validation

- **BLC-SETL-004:** **`POST`** MUST include a non-empty **`title`** and a **`songs`** array; missing **`title`**, empty **`title`**, or missing **`songs`** THEN **400**.

## List pagination and search

- **BLC-SETL-005:** **`GET /setlists`** supports **`page`**, **`page_size`**, optional **`q`** (title search), and [list-pagination.md](./list-pagination.md) (including whitespace-only **`q`** as no filter).

## When / then

- **BLC-SETL-006:** WHEN the caller may not read the owning team’s library THEN setlist reads respond **404**.
- **BLC-SETL-007:** WHEN the caller is **guest** on the owning team and attempts **PUT** or **DELETE** THEN the API responds **404**.
- **BLC-SETL-008:** WHEN the caller is the personal-team **owner**, or **admin** / **content_maintainer** on the owning team, THEN **PUT**/**DELETE** are allowed (subject to validation).
- **BLC-SETL-009:** WHEN **POST** creates a setlist THEN **`owner`** IS ALWAYS the caller’s **personal** team.
- **BLC-SETL-010:** WHEN **GET /setlists** runs THEN only setlists whose **`owner`** team the caller may read are returned; optional **`q`** filters by **title**.
- **BLC-SETL-011:** WHEN **GET /setlists/{id}**, **…/songs**, or **…/player** runs THEN visibility matches **GET /setlists/{id}**.
- **BLC-SETL-012:** WHEN **DELETE** succeeds THEN the setlist no longer appears under the same read rules.

## Cascading deletes

- **BLC-SETL-013:** WHEN a **user** account IS deleted THEN setlists owned by their **personal** team are removed with that team ([user.md](./user.md)).
- **BLC-SETL-014:** WHEN a **song** in **`songs`** IS deleted THEN setlist payloads MAY retain stale ids until **PUT** ([song.md](./song.md)).
