//! Shared numeric limits for API request bodies (see BLC / validation in handlers).

/// Maximum length of a shared team name (trimmed), aligned with persistence and OpenAPI docs.
pub const MAX_TEAM_NAME_LEN: usize = 256;

/// Maximum blob id references attached to a single song on create/update.
pub const MAX_BLOBS_PER_SONG: usize = 64;

/// Maximum additional member entries in create/update team payloads (excluding the creating user).
pub const MAX_TEAM_MEMBER_INPUTS: usize = 500;
