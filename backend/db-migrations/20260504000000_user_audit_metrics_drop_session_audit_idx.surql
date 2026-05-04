-- User activity metrics derive from http_request_audit; drop persisted counters/timestamps.
REMOVE FIELD IF EXISTS request_count ON user;
REMOVE FIELD IF EXISTS last_login_at ON user;

-- Speed session-scoped aggregates on http_request_audit.
DEFINE INDEX OVERWRITE http_request_audit_session_idx ON http_request_audit FIELDS session CONCURRENTLY;
