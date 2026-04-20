-- Align `like` with `20240301000002_like.surql` for SurrealDB 2.6.x (`COLUMNS` unique index).
-- Normalizes 3.x exports (`FIELDS`, `TYPE NORMAL`, parenthesized ASSERT / VALUE).
-- Cascade events (`like_user_cascade`, `like_song_cascade`) stay on `user` / `song`.

DEFINE FIELD OVERWRITE owner ON like TYPE record<user>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE song ON like TYPE record<song>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE created_at ON like TYPE datetime
    DEFAULT time::now()
    VALUE $before ?? $value
    READONLY;

DEFINE INDEX OVERWRITE like_owner_song_unique ON like COLUMNS owner, song UNIQUE;
