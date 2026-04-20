DEFINE FIELD OVERWRITE oauth_picture_url ON user TYPE option<string> VALUE $value ?? $before ?? NONE PERMISSIONS FULL;
DEFINE FIELD OVERWRITE oauth_avatar_blob ON user TYPE option<record<blob>> VALUE (IF type::is_none($value) THEN NONE ELSE $value ?? $before ?? NONE END) PERMISSIONS FULL;
DEFINE FIELD OVERWRITE avatar_blob ON user TYPE option<record<blob>> VALUE (IF type::is_none($value) THEN NONE ELSE $value ?? $before ?? NONE END) PERMISSIONS FULL;

DEFINE EVENT OVERWRITE blob_user_avatar_unset ON blob WHEN $event = 'DELETE' THEN (UPDATE user SET avatar_blob = NONE WHERE avatar_blob = $before.id);
DEFINE EVENT OVERWRITE blob_user_oauth_avatar_unset ON blob WHEN $event = 'DELETE' THEN (UPDATE user SET oauth_avatar_blob = NONE WHERE oauth_avatar_blob = $before.id);
