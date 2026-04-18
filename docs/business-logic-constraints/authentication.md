# Authentication for `/api/v1`

Applies to routes under **`/api/v1`** that require an authenticated user session (most mutating and private reads). The unauthenticated OpenAPI document is specified in [api-documentation.md](./api-documentation.md) (**BLC-DOCS-001**). Other public routes (e.g. health checks) are deployment-specific and are **not** covered here.

## When / then

- **BLC-AUTH-001:** WHEN a caller uses a route that **requires authentication** without an **`Authorization`** header whose value is interpreted as a **Bearer** session token (see **BLC-USER-006** for alternate accepted forms on **`GET /users/me`**) THEN the API responds **401**.
- **BLC-AUTH-002:** WHEN **`Authorization: Bearer <token>`** is present but **`<token>`** IS NOT a valid, active session THEN the API responds **401** before evaluating resource rules that would yield **403** or **404**.

## Relation to sessions

Session lifecycle and **404**/**403** on **`/users/.../sessions`** are in [session.md](./session.md). **BLC-AUTH-002** applies when the token never identifies a session at all; after **BLC-SESS-008**/**BLC-SESS-009**, a once-valid token MAY also yield **401** on subsequent calls.

## Email OTP

- **BLC-AUTH-OTP-001:** **`POST /auth/otp/request`** stores a short-lived hashed code and sends it out-of-band. Per-IP rate limits apply (see server **`auth_rate_limit_*`** settings).
- **BLC-AUTH-OTP-002:** **`POST /auth/otp/verify`** validates the code; after too many failures the code is invalidated (**429** / request a new code — see server **`otp_max_attempts`**).
- **BLC-AUTH-OTP-003:** When **`WORSHIP_OTP_ALLOW_SELF_SIGNUP`** is unset or true, a successful verify MAY create a user for a previously unknown email (same as historical behavior). When **`WORSHIP_OTP_ALLOW_SELF_SIGNUP`** is **`false`**/**`0`**, verify succeeds only if the user already exists; otherwise **400** with a stable message (`invalid_request` / no self-signup).
