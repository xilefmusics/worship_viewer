DEFINE TABLE OVERWRITE team TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE members ON team TYPE array<object> VALUE $value ?? $before ?? [] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE members.*.role ON team TYPE string ASSERT $value INSIDE ['guest', 'content_maintainer', 'admin'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE members.*.user ON team TYPE record<user> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE name ON team TYPE string ASSERT string::len(string::trim($value)) > 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE owner ON team TYPE none | record<user> VALUE (IF type::is_none($value) THEN NONE ELSE $value ?? $before ?? NONE END) PERMISSIONS FULL;

DEFINE EVENT OVERWRITE team_invitation_team_cascade ON team WHEN $event = 'DELETE' THEN (DELETE team_invitation WHERE team = $before.id);
DEFINE EVENT OVERWRITE team_personal_blob_cascade ON team WHEN $event = 'DELETE' AND $before.owner != NONE THEN (DELETE blob WHERE owner = $before.id);
DEFINE EVENT OVERWRITE team_personal_collection_cascade ON team WHEN $event = 'DELETE' AND $before.owner != NONE THEN (DELETE collection WHERE owner = $before.id);
DEFINE EVENT OVERWRITE team_personal_setlist_cascade ON team WHEN $event = 'DELETE' AND $before.owner != NONE THEN (DELETE setlist WHERE owner = $before.id);
DEFINE EVENT OVERWRITE team_personal_song_cascade ON team WHEN $event = 'DELETE' AND $before.owner != NONE THEN (DELETE song WHERE owner = $before.id);
