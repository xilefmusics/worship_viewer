-- Align `team_invitation` with `20260413170000_team_invitation.surql` for databases
-- exported from SurrealDB 3.x (`FIELDS` index syntax, `TYPE NORMAL`, parenthesized
-- ASSERT). Uses `COLUMNS` for the index so SurrealDB 2.6.x matches the rest of
-- `backend/db-migrations/`.

DEFINE FIELD OVERWRITE team ON team_invitation TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_by ON team_invitation TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_at ON team_invitation TYPE datetime
    DEFAULT time::now()
    VALUE $before ?? $value
    READONLY;

DEFINE INDEX OVERWRITE team_invitation_team_idx ON team_invitation COLUMNS team;
