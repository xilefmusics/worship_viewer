-- Align `user` with the team-based authorization model: drop legacy ACL columns
-- (`read` / `write`, including `read.*` / `write.*` exports) and redefine the
-- remaining fields to match `UserRecord`. Uses the same SurrealQL helpers as
-- `20240301000009_user.surql` (`string::is::email`, `type::is::none`) for
-- compatibility with SurrealDB 2.6.x; after upgrading to 3.x, apply the renames
-- from `docs/surrealdb-3.0-upgrade.md` if the server requires them.
--
-- Safe only after `20260413130000_team_personal_backfill` has been applied (ACL
-- data copied into teams). `REMOVE FIELD` drops only the schema; clear stored
-- values first with `UNSET` (see SurrealDB REMOVE FIELD docs).

UPDATE user UNSET read, write;

REMOVE FIELD IF EXISTS read ON user;
REMOVE FIELD IF EXISTS write ON user;

DEFINE FIELD OVERWRITE email ON user TYPE string
    VALUE string::lowercase(string::trim($value))
    ASSERT string::is::email($value);

DEFINE FIELD OVERWRITE role ON user TYPE string
    VALUE string::lowercase(($value ?? $before) ?? "default")
    ASSERT $value INSIDE ["default", "admin"];

DEFINE FIELD OVERWRITE default_collection ON user TYPE option<record<collection>>
    VALUE IF type::is::none($value) THEN NONE ELSE ($value ?? $before) ?? NONE END;

DEFINE FIELD OVERWRITE created_at ON user TYPE datetime
    VALUE $before ?? time::now()
    READONLY;

DEFINE FIELD OVERWRITE last_login_at ON user TYPE option<datetime>
    VALUE ($value ?? $before) ?? NONE;

DEFINE FIELD OVERWRITE request_count ON user TYPE int
    VALUE ($value ?? $before) ?? 0;

DEFINE INDEX OVERWRITE user_email_unique ON user COLUMNS email UNIQUE;
