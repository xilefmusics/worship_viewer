DEFINE TABLE OVERWRITE media_asset TYPE NORMAL SCHEMAFULL PERMISSIONS NONE;

DEFINE FIELD OVERWRITE owner ON media_asset TYPE record<team> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE media_id ON media_asset TYPE record<media> ASSERT $value != NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE kind ON media_asset TYPE string ASSERT $value IN ['video', 'audio', 'image', 'pdf', 'svg'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE content_type ON media_asset TYPE string ASSERT string::len($value) > 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE byte_length ON media_asset TYPE int ASSERT $value >= 0 PERMISSIONS FULL;
DEFINE FIELD OVERWRITE status ON media_asset TYPE string ASSERT $value IN ['staging', 'final'] PERMISSIONS FULL;
DEFINE FIELD OVERWRITE operation_id ON media_asset TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE etag ON media_asset TYPE none | string PERMISSIONS FULL;
DEFINE FIELD OVERWRITE created_at ON media_asset TYPE datetime READONLY VALUE $before ?? time::now() PERMISSIONS FULL;

DEFINE INDEX OVERWRITE media_asset_owner_idx ON media_asset FIELDS owner CONCURRENTLY;
DEFINE INDEX OVERWRITE media_asset_media_idx ON media_asset FIELDS media_id CONCURRENTLY;
DEFINE INDEX OVERWRITE media_asset_operation_idx ON media_asset FIELDS operation_id CONCURRENTLY;
