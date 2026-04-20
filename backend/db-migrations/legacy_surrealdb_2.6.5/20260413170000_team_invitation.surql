DEFINE TABLE OVERWRITE team_invitation SCHEMAFULL;

DEFINE FIELD OVERWRITE team ON team_invitation TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_by ON team_invitation TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_at ON team_invitation TYPE datetime
    DEFAULT time::now()
    VALUE $before ?? $value
    READONLY;

DEFINE INDEX OVERWRITE team_invitation_team_idx ON team_invitation COLUMNS team;

DEFINE EVENT OVERWRITE team_invitation_team_cascade ON team
    WHEN $event = "DELETE"
    THEN DELETE team_invitation WHERE team = $before.id;
