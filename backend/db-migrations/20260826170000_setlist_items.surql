-- E5.6: replace canonical setlist `songs` with tagged ordered `items`.
-- Legacy song elements become `{ kind: "song", ... }` without changing order or overrides.
-- DB uses `kind` (SurrealQL `type` is reserved); the API tagged field remains `type`.

DEFINE FIELD OVERWRITE items ON setlist TYPE array<object> VALUE $value ?? $before ?? [] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.kind ON setlist TYPE string ASSERT $value IN ['song', 'media'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.id ON setlist TYPE record<song> | record<media> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.key ON setlist TYPE none | object PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.key.level ON setlist TYPE none | int ASSERT $value = NONE OR ($value >= 0 AND $value < 12) PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.nr ON setlist TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.tempo ON setlist TYPE none | int ASSERT $value = NONE OR $value >= 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.language ON setlist TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow ON setlist TYPE none | array<object> PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow.*.title ON setlist TYPE string ASSERT string::len(string::trim($value)) > 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow.*.occurrence_index ON setlist TYPE none | int ASSERT $value = NONE OR $value >= 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE items.*.flow.*.repeats ON setlist TYPE none | int ASSERT $value = NONE OR $value >= 1 PERMISSIONS FULL;

UPDATE setlist SET items = array::map(songs ?? [], |$s: any| {
	kind: 'song',
	id: $s.id,
	nr: $s.nr,
	key: $s.key,
	tempo: $s.tempo,
	language: $s.language,
	flow: $s.flow
});

REMOVE FIELD songs ON setlist;

DEFINE FUNCTION OVERWRITE fn::setlist_items_without_song($items: array, $song_id: any) {
  RETURN array::filter($items, |$e: any| $e.kind != 'song' OR $e.id != $song_id);
} PERMISSIONS FULL;

DEFINE EVENT OVERWRITE setlist_song_remove ON song WHEN $event = 'DELETE' THEN (
  UPDATE setlist SET items = fn::setlist_items_without_song(items, $before.id)
  WHERE $before.id INSIDE array::map(
    array::filter(items ?? [], |$e: any| $e.kind = 'song'),
    |$e: any| ($e.id)
  )
);
