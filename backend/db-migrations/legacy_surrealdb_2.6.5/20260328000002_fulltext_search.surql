DEFINE FIELD OVERWRITE search_content ON song TYPE string
    DEFAULT ''
    VALUE (string::join(' ', array::flatten(array::flatten(
        array::flatten(data.sections.*.lines).*.parts
    ).*.languages) ?? []) ?? '');

DEFINE INDEX OVERWRITE song_data_title_search_idx ON song
    FIELDS data.title SEARCH ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE song_data_artist_search_idx ON song
    FIELDS data.artist SEARCH ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE song_content_search_idx ON song
    FIELDS search_content SEARCH ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE setlist_title_search_idx ON setlist
    FIELDS title SEARCH ANALYZER text_search BM25;

DEFINE INDEX OVERWRITE collection_title_search_idx ON collection
    FIELDS title SEARCH ANALYZER text_search BM25;
