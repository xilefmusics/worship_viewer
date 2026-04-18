# Business logic constraints for monitoring (admin metrics)

## Static

- **BLC-MON-001:** Every HTTP request handled by the backend results in **one** `http_request_audit` row (including `/auth/*`, `/api/docs/*`, and static asset routes), written **asynchronously** with best-effort persistence (logging failures MUST NOT change the HTTP response).
- **BLC-MON-002:** Authenticated `/api/v1/*` requests that pass session validation populate `user` and `session` record links on the audit row; requests without a validated session (or outside `/api/v1`) store **no** user/session links (`NONE`).
- **BLC-MON-003:** When a **user** or **session** row is **deleted**, existing `http_request_audit` rows remain; the corresponding `user` and/or `session` link fields are cleared so no dangling record references remain.
- **BLC-MON-004:** `GET /api/v1/monitoring/http-audit-logs` is **admin-only**: an authenticated non-admin receives **403**; no session receives **401**.

## Notes

- Additional monitoring endpoints (for example monthly active users) SHOULD live under the **`/api/v1/monitoring/`** prefix and follow the same admin-only pattern unless explicitly documented otherwise.
