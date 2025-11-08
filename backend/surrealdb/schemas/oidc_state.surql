DEFINE TABLE OVERWRITE oidc_state SCHEMAFULL;

DEFINE FIELD OVERWRITE pkce_verifier ON oidc_state TYPE string;
DEFINE FIELD OVERWRITE nonce ON oidc_state TYPE string;
DEFINE FIELD OVERWRITE redirect_to ON oidc_state TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON oidc_state TYPE datetime
    VALUE $before ?? time::now()
    READONLY;
DEFINE FIELD OVERWRITE expires_at ON oidc_state TYPE datetime;

DEFINE INDEX OVERWRITE oidc_state_expires_idx ON oidc_state COLUMNS expires_at;
