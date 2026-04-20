-- Align `http_request_audit` with `20260418120000_http_request_audit.surql` for SurrealDB 2.6.x
-- (`type::is::none`, `COLUMNS` indexes). Normalizes 3.x exports (`type::is_none`, `FIELDS`,
-- parenthesized ASSERT / VALUE).

DEFINE FIELD OVERWRITE request_id ON http_request_audit TYPE string ASSERT $value != NONE;

DEFINE FIELD OVERWRITE method ON http_request_audit TYPE string ASSERT $value != NONE;

DEFINE FIELD OVERWRITE path ON http_request_audit TYPE string ASSERT $value != NONE;

DEFINE FIELD OVERWRITE status_code ON http_request_audit TYPE int ASSERT $value != NONE;

DEFINE FIELD OVERWRITE duration_ms ON http_request_audit TYPE int ASSERT $value != NONE;

DEFINE FIELD OVERWRITE user ON http_request_audit TYPE option<record<user>>
    VALUE IF type::is::none($value) THEN NONE ELSE $value ?? $before ?? NONE END;

DEFINE FIELD OVERWRITE session ON http_request_audit TYPE option<record<session>>
    VALUE IF type::is::none($value) THEN NONE ELSE $value ?? $before ?? NONE END;

DEFINE FIELD OVERWRITE created_at ON http_request_audit TYPE datetime
    DEFAULT time::now()
    VALUE $before ?? $value
    READONLY;

DEFINE INDEX OVERWRITE http_request_audit_created_at_idx ON http_request_audit COLUMNS created_at;

DEFINE INDEX OVERWRITE http_request_audit_user_idx ON http_request_audit COLUMNS user;

DEFINE EVENT OVERWRITE http_request_audit_user_clear ON user
    WHEN $event = "DELETE"
    THEN UPDATE http_request_audit SET user = NONE WHERE user = $before.id;

DEFINE EVENT OVERWRITE http_request_audit_session_clear ON session
    WHEN $event = "DELETE"
    THEN UPDATE http_request_audit SET session = NONE WHERE session = $before.id;
