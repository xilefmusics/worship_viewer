DEFINE TABLE OVERWRITE user SCHEMAFULL;

DEFINE FIELD OVERWRITE email ON user TYPE string
    VALUE string::lowercase(string::trim($value))
    ASSERT string::is::email($value);

DEFINE FIELD OVERWRITE role ON user TYPE string
    VALUE string::lowercase($value ?? $before ?? "default")
    ASSERT $value INSIDE ["default", "admin"];

DEFINE FIELD OVERWRITE read ON user TYPE array<record>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE write ON user TYPE array<record>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE created_at ON user TYPE datetime
    VALUE $before ?? time::now()
    READONLY;

DEFINE FIELD OVERWRITE last_login_at ON user TYPE option<datetime>
    VALUE $value ?? $before ?? NONE;

DEFINE FIELD OVERWRITE request_count ON user TYPE int
    VALUE $value ?? $before ?? 0;

DEFINE INDEX OVERWRITE user_email_unique ON user COLUMNS email UNIQUE;
