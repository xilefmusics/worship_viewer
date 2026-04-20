-- Align `otp` with `20240301000004_otp.surql` and `20260417000000_otp_failed_attempts.surql`
-- for SurrealDB 2.6.x (`COLUMNS` index). Normalizes 3.x exports (`FIELDS`, `TYPE NORMAL`).

DEFINE FIELD OVERWRITE code ON otp TYPE string;

DEFINE FIELD OVERWRITE created_at ON otp TYPE datetime
    VALUE $before ?? time::now()
    READONLY;

DEFINE FIELD OVERWRITE expires_at ON otp TYPE datetime;

DEFINE FIELD OVERWRITE failed_attempts ON otp TYPE int DEFAULT 0;

DEFINE INDEX OVERWRITE otp_expires_idx ON otp COLUMNS expires_at;
