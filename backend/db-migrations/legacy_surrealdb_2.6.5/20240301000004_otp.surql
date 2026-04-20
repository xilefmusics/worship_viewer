DEFINE TABLE OVERWRITE otp SCHEMAFULL;

DEFINE FIELD OVERWRITE code ON otp TYPE string;
DEFINE FIELD OVERWRITE created_at ON otp TYPE datetime
    VALUE $before ?? time::now()
    READONLY;
DEFINE FIELD OVERWRITE expires_at ON otp TYPE datetime;

DEFINE INDEX OVERWRITE otp_expires_idx ON otp COLUMNS expires_at;
