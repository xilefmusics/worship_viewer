-- App serializes `provider` on OIDC CSRF state records (`OidcStateRecord`); SCHEMAFULL
-- `oidc_state` must declare it. Additive for DBs that already ran `20260420000006_define_table_oidc_state.surql`.
DEFINE FIELD OVERWRITE provider ON oidc_state TYPE none | string PERMISSIONS FULL;
