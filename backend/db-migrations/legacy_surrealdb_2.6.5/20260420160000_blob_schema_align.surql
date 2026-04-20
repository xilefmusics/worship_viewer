-- Align `blob` with `20240301000000_blob.surql`, `20260418080000_blob_svg_xml_mime.surql`,
-- and `20260413160000_resource_owner_rename_from_teamowner.surql` (`owner` = record<team>),
-- plus blob events from `20240301000001_collection.surql` and
-- `20260418130000_surrealql_idiom_v3_compat.surql`. Normalizes 3.x exports (`TYPE NORMAL`,
-- parenthesized ASSERT / VALUE / WHERE).
--
-- Legacy `teamowner`: clear document data, then remove field definition if present.

UPDATE blob UNSET teamowner;
REMOVE FIELD IF EXISTS teamowner ON blob;

DEFINE FIELD OVERWRITE owner ON blob TYPE record<team>
    ASSERT $value != NONE;

DEFINE FIELD OVERWRITE file_type ON blob TYPE string
    ASSERT $value INSIDE ["image/png", "image/jpeg", "image/svg", "image/svg+xml"];

DEFINE FIELD OVERWRITE width ON blob TYPE int
    ASSERT $value >= 0;

DEFINE FIELD OVERWRITE height ON blob TYPE int
    ASSERT $value >= 0;

DEFINE FIELD OVERWRITE ocr ON blob TYPE string
    VALUE $value ?? $before ?? "";

DEFINE FIELD OVERWRITE created_at ON blob TYPE datetime
    VALUE $before ?? time::now()
    READONLY;

DEFINE EVENT OVERWRITE collection_cover_blob_cascade ON blob
    WHEN $event = "DELETE"
    THEN DELETE collection WHERE cover = $before.id;

DEFINE EVENT OVERWRITE song_blob_remove ON blob
    WHEN $event = "DELETE"
    THEN (
        UPDATE song
            SET blobs = fn::blob_array_without_id(blobs, $before.id)
            WHERE $before.id INSIDE blobs
    );
