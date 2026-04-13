-- Personal team for each resource owner (team.owner = user). Requires prior personal-team backfill.
-- One map user id string -> personal team, then FOR loops with O(1) map lookups per row.
-- Note: `UPDATE tbl SET ... WHERE $map[type::string(owner)] ...` fails in SurrealDB 2.6 transactions
-- (`owner` is not row-bound in that WHERE), so we use explicit `$rec.owner` from SELECT.

LET $user_team_entries = SELECT VALUE [type::string(owner), id] FROM team WHERE owner IS NOT NONE;
LET $user_to_team = object::from_entries($user_team_entries);

DEFINE FIELD OVERWRITE teamowner ON blob TYPE option<record<team>>;

FOR $rec IN (SELECT id, owner FROM blob) {
    LET $t = $user_to_team[type::string($rec.owner)];
    IF $t IS NOT NONE {
        UPDATE $rec.id SET teamowner = $t;
    };
};

DEFINE FIELD OVERWRITE teamowner ON blob TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE teamowner ON song TYPE option<record<team>>;

FOR $rec IN (SELECT id, owner FROM song) {
    LET $t = $user_to_team[type::string($rec.owner)];
    IF $t IS NOT NONE {
        UPDATE $rec.id SET teamowner = $t;
    };
};

DEFINE FIELD OVERWRITE teamowner ON song TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE teamowner ON collection TYPE option<record<team>>;

FOR $rec IN (SELECT id, owner FROM collection) {
    LET $t = $user_to_team[type::string($rec.owner)];
    IF $t IS NOT NONE {
        UPDATE $rec.id SET teamowner = $t;
    };
};

DEFINE FIELD OVERWRITE teamowner ON collection TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE teamowner ON setlist TYPE option<record<team>>;

FOR $rec IN (SELECT id, owner FROM setlist) {
    LET $t = $user_to_team[type::string($rec.owner)];
    IF $t IS NOT NONE {
        UPDATE $rec.id SET teamowner = $t;
    };
};

DEFINE FIELD OVERWRITE teamowner ON setlist TYPE record<team>
    ASSERT $value != NONE;
