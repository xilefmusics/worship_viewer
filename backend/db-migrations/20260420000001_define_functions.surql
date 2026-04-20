-- Helpers referenced by DEFINE FIELD / DEFINE EVENT in later migrations (events, song metadata, search).

DEFINE FUNCTION OVERWRITE fn::blob_array_without_id($blobs: array, $rid: any) {
  IF array::len($blobs) = 0 {
    RETURN [];
  };
  LET $head = array::first($blobs);
  LET $n = array::len($blobs);
  LET $tail = (
    IF $n > 1 {
      array::slice($blobs, 1, $n - 1)
    } ELSE {
      []
    }
  );
  LET $rest = fn::blob_array_without_id($tail, $rid);
  IF $head = $rid {
    RETURN $rest;
  };
  RETURN array::concat([$head], $rest);
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::song_link_array_without_song($links: array, $song_id: any) {
  IF array::len($links) = 0 {
    RETURN [];
  };
  LET $head = array::first($links);
  LET $n = array::len($links);
  LET $tail = (
    IF $n > 1 {
      array::slice($links, 1, $n - 1)
    } ELSE {
      []
    }
  );
  LET $rest = fn::song_link_array_without_song($tail, $song_id);
  IF $head.id = $song_id {
    RETURN $rest;
  };
  RETURN array::concat([$head], $rest);
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::song_nonempty_strs($arr: array) -> array {
  RETURN array::filter($arr, |$x: any| string::len($x) > 0);
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::song_normalize_metadata_array($value: any, $before: any) -> array {
  LET $arr = $value ?? $before ?? [];
  LET $ne = fn::song_nonempty_strs($arr);
  IF array::len($ne) > 0 {
    RETURN $ne;
  };
  RETURN [''];
} PERMISSIONS FULL;

DEFINE FUNCTION OVERWRITE fn::song_search_content_from_data($data: object) {
  LET $tokens = array::flatten(
    array::map($data.sections ?? [], |$sec: any|
      array::flatten(
        array::map($sec.lines ?? [], |$line: any|
          array::flatten(
            array::map($line.parts ?? [], |$part: any|
              ($part.languages ?? [])
            )
          )
        )
      )
    )
  );
  RETURN string::join(' ', $tokens ?? []);
} PERMISSIONS FULL;
