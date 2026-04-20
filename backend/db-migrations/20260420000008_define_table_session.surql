DEFINE TABLE OVERWRITE session TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE created_at ON session TYPE datetime DEFAULT time::now() READONLY VALUE $before ?? $value PERMISSIONS FULL;
DEFINE FIELD OVERWRITE expires_at ON session TYPE datetime PERMISSIONS FULL;
DEFINE FIELD OVERWRITE user ON session TYPE record<user> ASSERT $value != NONE PERMISSIONS FULL;

DEFINE INDEX OVERWRITE session_user_idx ON session FIELDS user CONCURRENTLY;
DEFINE EVENT OVERWRITE http_request_audit_session_clear ON session WHEN $event = 'DELETE' THEN (UPDATE http_request_audit SET session = NONE WHERE session = $before.id);
