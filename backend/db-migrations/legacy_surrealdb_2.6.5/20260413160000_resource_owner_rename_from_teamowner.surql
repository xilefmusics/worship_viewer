-- Rename content team scope field teamowner -> owner (record<team>). Legacy user owner was removed in 20260413150000.

REMOVE EVENT IF EXISTS team_personal_blob_cascade ON team;
REMOVE EVENT IF EXISTS team_personal_song_cascade ON team;
REMOVE EVENT IF EXISTS team_personal_collection_cascade ON team;
REMOVE EVENT IF EXISTS team_personal_setlist_cascade ON team;

-- Rows can still lack teamowner after 20260413140000 (e.g. legacy/import data where the user→team map
-- lookup missed). Backfill from orphan user `owner` on the document, then team:public for catalog-style assets.
LET $user_team_entries = SELECT VALUE [type::string(owner), id] FROM team WHERE owner IS NOT NONE;
LET $user_to_team = object::from_entries($user_team_entries);

FOR $rec IN (SELECT id, owner, teamowner FROM blob) {
    IF $rec.teamowner IS NONE AND $rec.owner IS NOT NONE {
        LET $t = $user_to_team[type::string($rec.owner)];
        IF $t IS NOT NONE {
            UPDATE $rec.id SET teamowner = $t;
        };
    };
};
FOR $rec IN (SELECT id, teamowner FROM blob) {
    IF $rec.teamowner IS NONE {
        UPDATE $rec.id SET teamowner = team:public;
    };
};

FOR $rec IN (SELECT id, owner, teamowner FROM song) {
    IF $rec.teamowner IS NONE AND $rec.owner IS NOT NONE {
        LET $t = $user_to_team[type::string($rec.owner)];
        IF $t IS NOT NONE {
            UPDATE $rec.id SET teamowner = $t;
        };
    };
};
FOR $rec IN (SELECT id, teamowner FROM song) {
    IF $rec.teamowner IS NONE {
        UPDATE $rec.id SET teamowner = team:public;
    };
};

FOR $rec IN (SELECT id, owner, teamowner FROM collection) {
    IF $rec.teamowner IS NONE AND $rec.owner IS NOT NONE {
        LET $t = $user_to_team[type::string($rec.owner)];
        IF $t IS NOT NONE {
            UPDATE $rec.id SET teamowner = $t;
        };
    };
};
FOR $rec IN (SELECT id, teamowner FROM collection) {
    IF $rec.teamowner IS NONE {
        UPDATE $rec.id SET teamowner = team:public;
    };
};

FOR $rec IN (SELECT id, owner, teamowner FROM setlist) {
    IF $rec.teamowner IS NONE AND $rec.owner IS NOT NONE {
        LET $t = $user_to_team[type::string($rec.owner)];
        IF $t IS NOT NONE {
            UPDATE $rec.id SET teamowner = $t;
        };
    };
};
FOR $rec IN (SELECT id, teamowner FROM setlist) {
    IF $rec.teamowner IS NONE {
        UPDATE $rec.id SET teamowner = team:public;
    };
};

DEFINE FIELD OVERWRITE owner ON blob TYPE option<record<team>>;
UPDATE blob SET owner = teamowner;
REMOVE FIELD IF EXISTS teamowner ON blob;
DEFINE FIELD OVERWRITE owner ON blob TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE owner ON song TYPE option<record<team>>;
UPDATE song SET owner = teamowner;
REMOVE FIELD IF EXISTS teamowner ON song;
DEFINE FIELD OVERWRITE owner ON song TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE owner ON collection TYPE option<record<team>>;
UPDATE collection SET owner = teamowner;
REMOVE FIELD IF EXISTS teamowner ON collection;
DEFINE FIELD OVERWRITE owner ON collection TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE owner ON setlist TYPE option<record<team>>;
UPDATE setlist SET owner = teamowner;
REMOVE FIELD IF EXISTS teamowner ON setlist;
DEFINE FIELD OVERWRITE owner ON setlist TYPE record<team>
    ASSERT $value != NONE;

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
