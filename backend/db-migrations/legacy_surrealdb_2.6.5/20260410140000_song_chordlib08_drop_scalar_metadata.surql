-- chordlib 0.8 / issue #50: remove legacy scalar metadata on song.data; full-text indexes on titles/artists arrays.

REMOVE INDEX IF EXISTS song_data_title_search_idx ON song;
REMOVE INDEX IF EXISTS song_data_artist_search_idx ON song;

UPDATE song SET data = data;

DEFINE FUNCTION OVERWRITE fn::song_normalize_metadata_array($value: any, $before: any) -> array {
    LET $arr = ($value ?? $before) ?? [];
    LET $ne = fn::song_nonempty_strs($arr);
    IF array::len($ne) > 0 {
        RETURN $ne;
    };
    RETURN [""];
};

DEFINE FIELD OVERWRITE data.titles ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_normalize_metadata_array($value, $before)
    ASSERT array::len($value) > 0;

DEFINE FIELD OVERWRITE data.artists ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_normalize_metadata_array($value, $before)
    ASSERT array::len($value) > 0;

DEFINE FIELD OVERWRITE data.languages ON song TYPE array<string>
    DEFAULT ALWAYS []
    VALUE fn::song_normalize_metadata_array($value, $before)
    ASSERT array::len($value) > 0;

UPDATE song SET data = data;

REMOVE FIELD IF EXISTS data.title ON song;
REMOVE FIELD IF EXISTS data.artist ON song;
REMOVE FIELD IF EXISTS data.language ON song;

UPDATE song SET data = data;

REMOVE FUNCTION IF EXISTS fn::song_coalesce_str_array;

DEFINE INDEX OVERWRITE song_data_titles_search_idx ON song
    FIELDS data.titles SEARCH ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE song_data_artists_search_idx ON song
    FIELDS data.artists SEARCH ANALYZER text_search BM25;
