# List endpoints: query parameters and pagination

Applies to **GET** list routes that support paging: **`/users`** (admin), **`/blobs`**, **`/collections`**, **`/setlists`**, **`/songs`**.

## Query parameters

- **BLC-LP-001:** **`page`** — 0-based index of the page.
- **BLC-LP-002:** **`page_size`** — maximum number of items per page.
- **BLC-LP-003:** **`q`** — optional search filter: **`/songs`** (full-text: titles, artists, lyrics per analyzer rules), **`/collections`** and **`/setlists`** (**title**), **`/users`** (admin list: **email** substring, case-insensitive), **`/blobs`** (**OCR** substring, case-insensitive). Whitespace-only **`q`** is treated as absent everywhere.

## Validation

- **BLC-LP-004:** WHEN **`page`** or **`page_size`** is present but not a valid integer THEN the API responds **400**.
- **BLC-LP-004a:** WHEN **`page_size`** IS **`0`** THEN the API responds **400** (zero is not a valid page size) with Problem `code` **`invalid_page_size`**.
- **BLC-LP-004b:** WHEN **`page_size`** EXCEEDS **500** THEN the API responds **400** with Problem `code` **`invalid_page_size`**.
- **BLC-LP-005:** WHEN **`q`** IS only whitespace THEN it IS treated as absent: the same result as omitting **`q`**, after applying visibility rules for the caller.

## Pagination behavior

- **BLC-LP-006:** ~~WHEN **`page_size`** IS **`0`** THEN no page-size cap IS applied~~ — **removed**: `page_size=0` is now rejected with `400` (see BLC-LP-004a).
- **BLC-LP-007:** WHEN **`page`** is absent it defaults to **0**. WHEN **`page_size`** is absent it defaults to **50**. The server hard cap is **500**.
- **BLC-LP-008:** WHEN **`page`** IS beyond the last page THEN the API responds **200** with an **empty array** (not **404**).
- **BLC-LP-009:** WHEN **`q`** IS combined with **`page`** / **`page_size`** THEN filtering runs first, then pagination over those results.

**Track A (current):** All list responses include an **`X-Total-Count`** header: the **total number of matching records after filters and before pagination**. Clients detect the last page when the returned **`items.len() < page_size`** or the page is **empty** (including “beyond last page” pages).
