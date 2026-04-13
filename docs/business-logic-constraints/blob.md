# Business logic constraints for the blob resource

## Static

- **BLC-BLOB-001:** Every blob belongs to exactly one **owning team** (the **`owner`** in responses).
- **BLC-BLOB-002:** Listing, fetching metadata, and downloading bytes require the caller to be allowed to **read that team’s library**; mutating or deleting a blob requires **library edit** rights on that team. Platform **admin** does **not** gain blob edit solely by role.
- **BLC-BLOB-003:** **`PUT`** MUST NOT change **`owner`**.
- **BLC-BLOB-004:** New blobs are created as metadata records; **GET …/data** MAY serve empty or placeholder bytes until binary content is supplied outside this HTTP API.
- **BLC-BLOB-005:** **`file_type`** on create/update MUST be among the image types the API accepts (e.g. **`image/png`**, **`image/jpeg`**); unsupported values THEN **400**.

## List pagination

- **`GET /blobs`** supports **`page`** and **`page_size`** per [list-pagination.md](./list-pagination.md).

## When / then

- **BLC-BLOB-006:** WHEN the caller may not read the owning team’s library THEN blob **GET** / list / **…/data** respond **404** (not **403**).
- **BLC-BLOB-007:** WHEN the caller has **guest**-level membership on the owning team and attempts **PUT** or **DELETE** THEN the API responds **404**.
- **BLC-BLOB-008:** WHEN the caller is the personal-team **owner**, or **admin** / **content_maintainer** on the owning team, THEN **PUT** and **DELETE** are allowed (subject to validation).
- **BLC-BLOB-009:** WHEN **POST** creates a blob THEN **`owner`** IS ALWAYS the caller’s **personal** team.
- **BLC-BLOB-010:** WHEN **GET /blobs** or **GET /blobs/{id}** runs THEN only blobs whose **`owner`** team the caller may read are included or returned; catalog-wide readable material MAY appear without team membership where the product exposes it.
- **BLC-BLOB-011:** WHEN **GET …/data** runs THEN the same visibility rules as metadata **GET** apply; IF bytes are available THEN they are served.
- **BLC-BLOB-012:** WHEN **PUT** runs THEN only **`file_type`**, **`width`**, **`height`**, and **`ocr`** may change.
- **BLC-BLOB-013:** WHEN **DELETE** succeeds THEN the blob no longer appears in the API and associated stored bytes MAY be removed.

## Cascading deletes and dependents

- **BLC-BLOB-014:** WHEN a blob used as a collection **`cover`** IS **DELETE**d THEN **GET** that collection MAY return **404** even if the collection id still exists until it is updated or removed.
- **BLC-BLOB-015:** WHEN a **user** account IS deleted THEN blobs owned by their **personal** team disappear with that team (see [user.md](./user.md)).
