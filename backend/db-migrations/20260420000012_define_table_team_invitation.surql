DEFINE TABLE OVERWRITE team_invitation TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE created_at ON team_invitation TYPE datetime DEFAULT time::now() READONLY VALUE $before ?? $value PERMISSIONS FULL;
DEFINE FIELD OVERWRITE created_by ON team_invitation TYPE record<user> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE team ON team_invitation TYPE record<team> ASSERT $value != NONE PERMISSIONS FULL;

DEFINE INDEX OVERWRITE team_invitation_team_idx ON team_invitation FIELDS team CONCURRENTLY;