-- Align `migration_script` with `20260320000000_change_db_migration_mechanism.surql` and
-- `migrations::ensure_migration_table` for SurrealDB 2.6.x (`COLUMNS` index). Normalizes
-- 3.x exports (`FIELDS`, `TYPE NORMAL`, `READONLY VALUE` ordering).

DEFINE FIELD OVERWRITE script_name ON migration_script TYPE string;

DEFINE FIELD OVERWRITE checksum ON migration_script TYPE string;

DEFINE FIELD OVERWRITE executed_at ON migration_script TYPE datetime VALUE time::now() READONLY;

DEFINE INDEX OVERWRITE migration_script_script_name_unique ON migration_script COLUMNS script_name UNIQUE;
