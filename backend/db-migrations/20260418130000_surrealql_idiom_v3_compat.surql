-- SurrealQL idiom fixes for 2.6.x semantics that remain valid on 3.x:
-- - Replace array filters using $this + record id comparisons with recursive helpers (no $this in events).
-- - Replace INSIDE songs.*.id with $before.id INSIDE array::map(songs, |$e| $e.id) (no .* chain).
-- - Use DEFINE FUNCTION helpers for link/array filtering (array::filter closures do not capture params/locals here).
-- - Replace search_content VALUE / backfill logic that used data.sections.*.lines / .*.parts / .*.languages
--   with explicit array::map + array::flatten (no all-idiom chains on arrays).
--
-- Note: DEFINE FIELD ... songs.*.id and data.sections.*... schema paths are unchanged; 3.x warnings
-- for those refer mainly to evaluation of .* chains in expressions, not the field-definition syntax.

DEFINE FUNCTION OVERWRITE fn::blob_array_without_id($blobs: array, $rid: any) {
    IF array::len($blobs) = 0 {
        RETURN [];
    };
    LET $head = array::first($blobs);
    LET $n = array::len($blobs);
    LET $tail = IF $n > 1 {
        array::slice($blobs, 1, $n - 1)
    } ELSE {
        []
    };
    LET $rest = fn::blob_array_without_id($tail, $rid);
    IF $head = $rid {
        RETURN $rest;
    };
    RETURN array::concat([$head], $rest);
};

DEFINE FUNCTION OVERWRITE fn::song_link_array_without_song($links: array, $song_id: any) {
    IF array::len($links) = 0 {
        RETURN [];
    };
    LET $head = array::first($links);
    LET $n = array::len($links);
    LET $tail = IF $n > 1 {
        array::slice($links, 1, $n - 1)
    } ELSE {
        []
    };
    LET $rest = fn::song_link_array_without_song($tail, $song_id);
    IF $head.id = $song_id {
        RETURN $rest;
    };
    RETURN array::concat([$head], $rest);
};

DEFINE FUNCTION OVERWRITE fn::song_search_content_from_data($data: object) {
    LET $tokens = array::flatten(
        array::map($data.sections ?? [], |$sec|
            array::flatten(
                array::map($sec.lines ?? [], |$line|
                    array::flatten(
                        array::map($line.parts ?? [], |$part|
                            $part.languages ?? []
                        )
                    )
                )
            )
        )
    );
    RETURN string::join(' ', $tokens ?? []);
};

DEFINE FIELD OVERWRITE search_content ON song TYPE string
    DEFAULT ''
    VALUE fn::song_search_content_from_data(data);

DEFINE EVENT OVERWRITE song_blob_remove ON blob
    WHEN $event = "DELETE"
    THEN (
        UPDATE song
            SET blobs = fn::blob_array_without_id(blobs, $before.id)
            WHERE $before.id INSIDE blobs
    );

DEFINE EVENT OVERWRITE collection_song_remove ON song
    WHEN $event = "DELETE"
    THEN (
        UPDATE collection
            SET songs = fn::song_link_array_without_song(songs, $before.id)
            WHERE $before.id INSIDE array::map(songs, |$e| $e.id)
    );

DEFINE EVENT OVERWRITE setlist_song_remove ON song
    WHEN $event = "DELETE"
    THEN (
        UPDATE setlist
            SET songs = fn::song_link_array_without_song(songs, $before.id)
            WHERE $before.id INSIDE array::map(songs, |$e| $e.id)
    );

UPDATE song SET search_content = fn::song_search_content_from_data(data);
