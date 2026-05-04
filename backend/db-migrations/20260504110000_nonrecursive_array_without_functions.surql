-- `fn::song_link_array_without_song` and `fn::blob_array_without_id` were implemented as linear
-- recursion (one nested call per array element). That hits SurrealDB's computation depth cap when
-- trimming long `collection.songs` / `setlist.songs` arrays or large `song.blobs` lists — e.g.
-- deleting a song would fail in `collection_song_remove` / `setlist_song_remove`.

DEFINE FUNCTION OVERWRITE fn::blob_array_without_id($blobs: array, $rid: any) {
  RETURN array::filter($blobs, |$b: any| $b != $rid);
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::song_link_array_without_song($links: array, $song_id: any) {
  RETURN array::filter($links, |$e: any| $e.id != $song_id);
} PERMISSIONS FULL;
