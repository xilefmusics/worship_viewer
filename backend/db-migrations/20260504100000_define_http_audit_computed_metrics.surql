-- Live request_count and last_used_at from http_request_audit (COMPUTED = re-evaluated on SELECT).
-- COMPUTED `$this` is the document shape; predicates use stable `id` (record refs).

DEFINE FUNCTION OVERWRITE fn::http_audit_count_for_user($rid: record<user>) -> int {
  LET $rows = SELECT count FROM (SELECT count() AS count FROM http_request_audit WHERE user = $rid GROUP ALL);
  IF array::len($rows) = 0 {
    RETURN 0;
  };
  RETURN array::first($rows).count;
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::http_audit_last_used_at_for_user($rid: record<user>) -> option<datetime> {
  LET $rows = SELECT created_at FROM http_request_audit WHERE user = $rid ORDER BY created_at DESC LIMIT 1;
  IF array::len($rows) = 0 {
    RETURN NONE;
  };
  RETURN array::first($rows).created_at;
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::http_audit_count_for_session($rid: record<session>) -> int {
  LET $rows = SELECT count FROM (SELECT count() AS count FROM http_request_audit WHERE session = $rid GROUP ALL);
  IF array::len($rows) = 0 {
    RETURN 0;
  };
  RETURN array::first($rows).count;
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::http_audit_last_used_at_for_session($rid: record<session>) -> option<datetime> {
  LET $rows = SELECT created_at FROM http_request_audit WHERE session = $rid ORDER BY created_at DESC LIMIT 1;
  IF array::len($rows) = 0 {
    RETURN NONE;
  };
  RETURN array::first($rows).created_at;
} PERMISSIONS FULL;

DEFINE FIELD OVERWRITE request_count ON user TYPE int COMPUTED fn::http_audit_count_for_user(id) PERMISSIONS FULL;
DEFINE FIELD OVERWRITE last_used_at ON user TYPE option<datetime> COMPUTED fn::http_audit_last_used_at_for_user(id) PERMISSIONS FULL;

DEFINE FIELD OVERWRITE request_count ON session TYPE int COMPUTED fn::http_audit_count_for_session(id) PERMISSIONS FULL;
DEFINE FIELD OVERWRITE last_used_at ON session TYPE option<datetime> COMPUTED fn::http_audit_last_used_at_for_session(id) PERMISSIONS FULL;
