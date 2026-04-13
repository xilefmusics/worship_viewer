# API documentation (OpenAPI)

## When / then

- **BLC-DOCS-001:** WHEN **`GET /api/docs/openapi.json`** runs **without** authentication THEN the API responds **200** and returns the OpenAPI schema for the HTTP API (exact wire format MAY follow the generator’s JSON layout).

This route is **outside** the **`/api/v1`** authenticated surface; see [authentication.md](./authentication.md).
