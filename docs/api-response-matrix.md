# API response status matrix (Phase 1)

Hand-maintained checklist: each `/api/v1/*` and `/auth/*` operation should document every **non-5xx** status the handler can return. Error bodies use `Problem` with `Content-Type: application/problem+json`.

| Area | Typical 4xx |
|------|----------------|
| Auth `/auth/login`, `/auth/callback` | 400 (bad state / config), 401 (claims), 429 (rate limit) |
| Resources | 400 (validation), 401 (auth), 403 (ACL), 404 (missing), 406 (Accept), 409 (conflict), 413 (upload size), 429 (rate limit) |

**Validation policy:** The API does not use **422**; request validation failures are **400** with `code: invalid_request` or `invalid_page_size` (see [http-contract.md](business-logic-constraints/http-contract.md)).

Regenerate from code when touching handlers: trace `AppError` from each `#[utoipa::path]` handler and align `responses(...)`.
