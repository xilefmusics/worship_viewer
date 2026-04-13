# List endpoints: query parameters and pagination

Applies to **GET** list routes that support paging: **`/users`** (admin), **`/blobs`**, **`/collections`**, **`/setlists`**, **`/songs`**.

## Query parameters

- **BLC-LP-001:** **`page`** — 0-based index of the page.
- **BLC-LP-002:** **`page_size`** — maximum number of items per page.
- **BLC-LP-003:** **`q`** — optional search filter on **`/songs`**, **`/collections`**, and **`/setlists`** only (not on **`/users`** or **`/blobs`**). Collection and setlist lists match **title**; song list also matches **artists** and lyric text per the product’s list-search rules (including stemming where applicable).

## Validation

- **BLC-LP-004:** WHEN **`page`** or **`page_size`** is present but not a valid integer THEN the API responds **400**.
- **BLC-LP-005:** WHEN **`q`** IS only whitespace THEN it IS treated as absent: the same result as omitting **`q`**, after applying visibility rules for the caller.

## Pagination behavior

- **BLC-LP-006:** WHEN **`page_size`** IS **`0`** THEN no page-size cap IS applied for that request (all items for the current **`page`** index that match visibility and **`q`** are returned).
- **BLC-LP-007:** WHEN only **`page`** OR only **`page_size`** IS supplied THEN the omitted parameter uses a server default; the request still succeeds (**200**).
- **BLC-LP-008:** WHEN **`page`** IS beyond the last page THEN the API responds **200** with an **empty array** (not **404**).
- **BLC-LP-009:** WHEN **`q`** IS combined with **`page`** / **`page_size`** THEN filtering runs first, then pagination over those results.

List responses documented here do not define a separate total-count field; clients infer “no more pages” from an empty page or a page shorter than **`page_size`** where applicable.
