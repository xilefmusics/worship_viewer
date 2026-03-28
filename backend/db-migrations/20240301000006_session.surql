DEFINE TABLE OVERWRITE session SCHEMAFULL;

DEFINE FIELD OVERWRITE user ON session TYPE record<user>
  ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_at ON session TYPE datetime
  DEFAULT time::now()
  VALUE $before ?? $value
  READONLY;

DEFINE FIELD OVERWRITE expires_at ON session TYPE datetime;

DEFINE INDEX OVERWRITE session_user_idx ON session COLUMNS user;

DEFINE EVENT OVERWRITE session_user_cascade ON user
    WHEN $event = "DELETE"
    THEN DELETE session WHERE user = $before.id;
