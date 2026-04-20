DEFINE TABLE OVERWRITE http_request_audit TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE created_at ON http_request_audit TYPE datetime DEFAULT time::now() READONLY VALUE $before ?? $value PERMISSIONS FULL;
DEFINE FIELD OVERWRITE duration_ms ON http_request_audit TYPE int ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE method ON http_request_audit TYPE string ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE path ON http_request_audit TYPE string ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE request_id ON http_request_audit TYPE string ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE session ON http_request_audit TYPE none | record<session> VALUE (IF type::is_none($value) THEN NONE ELSE $value ?? $before ?? NONE END) PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status_code ON http_request_audit TYPE int ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE user ON http_request_audit TYPE none | record<user> VALUE (IF type::is_none($value) THEN NONE ELSE $value ?? $before ?? NONE END) PERMISSIONS FULL;

DEFINE INDEX OVERWRITE http_request_audit_created_at_idx ON http_request_audit FIELDS created_at CONCURRENTLY;
DEFINE INDEX OVERWRITE http_request_audit_user_idx ON http_request_audit FIELDS user CONCURRENTLY;