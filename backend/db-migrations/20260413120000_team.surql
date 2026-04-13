DEFINE TABLE OVERWRITE team SCHEMAFULL;

DEFINE FIELD OVERWRITE name ON team TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE owner ON team TYPE option<record<user>>
    VALUE IF type::is::none($value) THEN NONE ELSE $value ?? $before ?? NONE END;

DEFINE FIELD OVERWRITE members ON team TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE members.*.user ON team TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE members.*.role ON team TYPE string
    ASSERT $value INSIDE ["guest", "content_maintainer", "admin"];

DEFINE EVENT OVERWRITE team_owner_user_cascade ON user
    WHEN $event = "DELETE"
    THEN DELETE team WHERE owner = $before.id;

CREATE team:public SET name = 'Public', owner = NONE, members = [];
