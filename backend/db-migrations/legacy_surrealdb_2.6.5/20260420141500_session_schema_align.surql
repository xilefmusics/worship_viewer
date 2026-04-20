-- Align `session` with `20240301000006_session.surql` and
-- `20260418120000_http_request_audit.surql` for SurrealDB 2.6.x (`COLUMNS` index,
-- unparenthesized ASSERT / WHERE). Normalizes 3.x exports (`FIELDS`, `($before).id`,
-- `TYPE NORMAL`).

DEFINE FIELD OVERWRITE user ON session TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_at ON session TYPE datetime
    DEFAULT time::now()
    VALUE $before ?? $value
    READONLY;

DEFINE FIELD OVERWRITE expires_at ON session TYPE datetime;

DEFINE INDEX OVERWRITE session_user_idx ON session COLUMNS user;

DEFINE EVENT OVERWRITE http_request_audit_session_clear ON session
    WHEN $event = "DELETE"
    THEN UPDATE http_request_audit SET session = NONE WHERE session = $before.id;
