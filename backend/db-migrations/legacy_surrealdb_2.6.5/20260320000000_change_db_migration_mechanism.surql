REMOVE TABLE script_migration;

DEFINE TABLE OVERWRITE migration_script SCHEMAFULL;
DEFINE FIELD OVERWRITE script_name ON migration_script TYPE string;
DEFINE FIELD OVERWRITE checksum ON migration_script TYPE string;
DEFINE FIELD OVERWRITE executed_at ON migration_script TYPE datetime VALUE time::now() READONLY;
DEFINE INDEX OVERWRITE migration_script_script_name_unique ON migration_script COLUMNS script_name UNIQUE;
