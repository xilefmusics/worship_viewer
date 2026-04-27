-- Client attribution for `http_request_audit` (see `client_attribution` in `http_audit`).
DEFINE FIELD OVERWRITE client_origin ON http_request_audit
  TYPE string
  VALUE $value ?? $before ?? 'unknown'
  ASSERT $value IN ['frontend', 'cli', 'swagger', 'unknown']
  PERMISSIONS FULL;
DEFINE FIELD OVERWRITE client_version ON http_request_audit TYPE option<string> PERMISSIONS FULL;
