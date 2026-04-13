# Cross-resource HTTP contract

Rules that apply across several **`/api/v1`** resources (path validation, idempotent deletes). Resource-specific visibility (**404** vs **403**) stays in each resource’s doc.

## Path parameters

- **BLC-HTTP-001:** WHEN a path segment that MUST match the API’s resource **id** format IS syntactically invalid THEN the API responds **400** (same class of validation as list query integers in **BLC-LP-004**; see [list-pagination.md](./list-pagination.md)).

## Idempotent DELETE

- **BLC-HTTP-002:** WHEN **DELETE** on a resource succeeds and the client issues the same **DELETE** again for that **id** THEN the API responds **404** (same pattern as **BLC-USER-014** for users).
