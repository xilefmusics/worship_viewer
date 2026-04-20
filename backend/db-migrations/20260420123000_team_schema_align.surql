-- Align `team` with the repo schema (personal vs shared teams, content `owner` =
-- record<team>). Normalizes field definitions and cascade events for databases
-- exported from SurrealDB 3.x (`TYPE NORMAL`, `type::is_none`, parenthesized
-- `THEN` blocks) onto the same SurrealQL as `20260413120000_team.surql`,
-- `20260413160000_resource_owner_rename_from_teamowner.surql`, and
-- `20260413170000_team_invitation.surql` — using `type::is::none` for
-- SurrealDB 2.6.x (see `docs/surrealdb-3.0-upgrade.md` when on 3.x only).

DEFINE FIELD OVERWRITE name ON team TYPE string
    ASSERT string::len(string::trim($value)) > 0;

DEFINE FIELD OVERWRITE owner ON team TYPE option<record<user>>
    VALUE IF type::is::none($value) THEN NONE ELSE $value ?? $before ?? NONE END;

DEFINE FIELD OVERWRITE members ON team TYPE array<object>
    VALUE $value ?? $before ?? [];

DEFINE FIELD OVERWRITE members.* ON team TYPE object;

DEFINE FIELD OVERWRITE members.*.user ON team TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE members.*.role ON team TYPE string
    ASSERT $value INSIDE ["guest", "content_maintainer", "admin"];

DEFINE EVENT OVERWRITE team_invitation_team_cascade ON team
    WHEN $event = "DELETE"
    THEN DELETE team_invitation WHERE team = $before.id;

DEFINE EVENT OVERWRITE team_personal_blob_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE blob WHERE owner = $before.id;

DEFINE EVENT OVERWRITE team_personal_song_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE song WHERE owner = $before.id;

DEFINE EVENT OVERWRITE team_personal_collection_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE collection WHERE owner = $before.id;

DEFINE EVENT OVERWRITE team_personal_setlist_cascade ON team
    WHEN $event = "DELETE" AND $before.owner IS NOT NONE
    THEN DELETE setlist WHERE owner = $before.id;
