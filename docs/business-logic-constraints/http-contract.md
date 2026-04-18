# Cross-resource HTTP contract

Rules that apply across several **`/api/v1`** resources (path validation, idempotent deletes). Resource-specific visibility (**404** vs **403**) stays in each resource’s doc.

## Path parameters

- **BLC-HTTP-001:** WHEN a path segment that MUST match the API’s resource **id** format IS syntactically invalid THEN the API responds **400** (same class of validation as list query integers in **BLC-LP-004**; see [list-pagination.md](./list-pagination.md)).

## Validation status codes

- **BLC-HTTP-003:** The API does **not** use **422 Unprocessable Entity**. Invalid JSON, unknown fields (`deny_unknown_fields`), and other request validation failures are **400 Bad Request** with `application/problem+json`.

## Idempotent DELETE

- **BLC-HTTP-002:** WHEN **DELETE** on a resource succeeds and the client issues the same **DELETE** again for that **id** THEN the API responds **404** (same pattern as **BLC-USER-014** for users).

## API rate limiting (`/api/v1/*`)

- **BLC-HTTP-004:** Versioned **`/api/v1/*`** routes are rate-limited **per client IP** (see `backend` settings **`API_RATE_LIMIT_RPS`** and **`API_RATE_LIMIT_BURST`**, defaults **50** RPS and burst **200**). WHEN the limit IS exceeded THEN the API responds **429 Too Many Requests** with **`Retry-After`** and **`X-RateLimit-*`** headers ([`actix-governor`](https://docs.rs/actix-governor/latest/actix_governor/)).
