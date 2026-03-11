//! CLI argument definitions (clap). No side effects.

use clap::{Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Debug, Parser)]
#[command(name = "worship-viewer", version, about = "CLI for the Worship Viewer REST API")]
pub struct Cli {
    /// Base URL of the Worship Viewer backend.
    ///
    /// Precedence:
    /// - flag `--base-url`
    /// - env `WORSHIP_VIEWER_BASE_URL`
    /// - config file `~/.worshipviewer/config.toml` (`base_url`)
    /// - default `http://127.0.0.1:8080`
    #[arg(long)]
    pub base_url: Option<String>,

    /// Session cookie value for the backend.
    ///
    /// The backend expects an `sso_session` cookie; this CLI will send
    /// `Cookie: sso_session=<value>` when configured.
    ///
    /// Precedence:
    /// - flag `--sso-session`
    /// - env `WORSHIP_VIEWER_SSO_SESSION`
    /// - config file `~/.worshipviewer/config.toml` (`sso_session`)
    #[arg(long)]
    pub sso_session: Option<String>,

    /// Bearer token to send as `Authorization: Bearer <token>`.
    #[arg(long, env = "WORSHIP_VIEWER_BEARER_TOKEN")]
    pub bearer_token: Option<String>,

    /// Output format. When set to `auto`, JSON is emitted when stdout is not a TTY.
    #[arg(long, env = "WORSHIP_VIEWER_OUTPUT", default_value = "auto")]
    pub output: OutputFormat,

    /// Global dry-run flag. When enabled, mutating commands are validated locally
    /// and the planned HTTP request is printed, but no request is sent.
    #[arg(long)]
    pub dry_run: bool,

    /// Request timeout in seconds.
    #[arg(long, env = "WORSHIP_VIEWER_TIMEOUT_SECS")]
    pub timeout_secs: Option<u64>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Introspect the OpenAPI schema exposed by the backend.
    Schema {
        /// Optional path prefix filter, e.g. `/api/v1/songs`.
        #[arg(long)]
        path_prefix: Option<String>,
    },
    /// Authentication and session bootstrap helpers.
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    /// User-related operations.
    Users {
        #[command(subcommand)]
        command: UsersCommand,
    },
    /// Session-related operations.
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },
    /// Song-related operations.
    Songs {
        #[command(subcommand)]
        command: SongsCommand,
    },
    /// Collection-related operations.
    Collections {
        #[command(subcommand)]
        command: CollectionsCommand,
    },
    /// Setlist-related operations.
    Setlists {
        #[command(subcommand)]
        command: SetlistsCommand,
    },
    /// Blob-related operations.
    Blobs {
        #[command(subcommand)]
        command: BlobsCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    /// Request a one-time password (OTP) to be sent to the given email.
    OtpRequest {
        /// Raw JSON payload matching `OtpRequest`.
        #[arg(long)]
        json: String,
    },
    /// Verify an OTP and establish a session.
    OtpVerify {
        /// Raw JSON payload matching `OtpVerify`.
        #[arg(long)]
        json: String,
    },
    /// Log out the current session.
    Logout,
}

#[derive(Debug, Subcommand)]
pub enum UsersCommand {
    /// List all users.
    List,
    /// Get a single user by id.
    Get {
        id: String,
    },
    /// Create a user from a JSON payload.
    Create {
        /// Raw JSON payload matching `CreateUserRequest`.
        #[arg(long)]
        json: String,
    },
    /// Delete a user by id.
    Delete {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SessionsCommand {
    /// List sessions for the current user.
    ListMine,
    /// Get a session for the current user.
    GetMine {
        id: String,
    },
    /// Delete a session for the current user.
    DeleteMine {
        id: String,
    },
    /// Create a session for the given user id.
    CreateForUser {
        user_id: String,
    },
    /// List sessions for a given user id.
    ListForUser {
        user_id: String,
    },
    /// Get a specific session for a given user id.
    GetForUser {
        user_id: String,
        id: String,
    },
    /// Delete a specific session for a given user id.
    DeleteForUser {
        user_id: String,
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SongsCommand {
    /// List all songs.
    List,
    /// Get a song by id.
    Get {
        id: String,
    },
    /// Get a player description for the given song id.
    Player {
        id: String,
    },
    /// Get an export URL for a song and format.
    ExportUrl {
        id: String,
        format: String,
    },
    /// Create a song from a JSON payload.
    Create {
        /// Raw JSON payload matching `CreateSong`.
        #[arg(long)]
        json: String,
    },
    /// Update a song with the given id from a JSON payload.
    Update {
        id: String,
        /// Raw JSON payload matching `CreateSong`.
        #[arg(long)]
        json: String,
    },
    /// Delete a song by id.
    Delete {
        id: String,
    },
    /// Import a song from an external identifier.
    Import {
        identifier: String,
    },
    /// Get like status for a song.
    LikeStatus {
        id: String,
    },
    /// Update like status for a song.
    UpdateLikeStatus {
        id: String,
        liked: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum CollectionsCommand {
    /// List all collections.
    List,
    /// Get a collection by id.
    Get {
        id: String,
    },
    /// List songs in a collection.
    Songs {
        id: String,
    },
    /// Get a player description for the given collection id.
    Player {
        id: String,
    },
    /// Get an export URL for a collection and format.
    ExportUrl {
        id: String,
        format: String,
    },
    /// Create a collection from a JSON payload.
    Create {
        /// Raw JSON payload matching `CreateCollection`.
        #[arg(long)]
        json: String,
    },
    /// Update a collection with the given id from a JSON payload.
    Update {
        id: String,
        /// Raw JSON payload matching `CreateCollection`.
        #[arg(long)]
        json: String,
    },
    /// Delete a collection by id.
    Delete {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SetlistsCommand {
    /// List all setlists.
    List,
    /// Get a setlist by id.
    Get {
        id: String,
    },
    /// List songs in a setlist.
    Songs {
        id: String,
    },
    /// Get a player description for the given setlist id.
    Player {
        id: String,
    },
    /// Get an export URL for a setlist and format.
    ExportUrl {
        id: String,
        format: String,
    },
    /// Create a setlist from a JSON payload.
    Create {
        /// Raw JSON payload matching `CreateSetlist`.
        #[arg(long)]
        json: String,
    },
    /// Update a setlist with the given id from a JSON payload.
    Update {
        id: String,
        /// Raw JSON payload matching `CreateSetlist`.
        #[arg(long)]
        json: String,
    },
    /// Delete a setlist by id.
    Delete {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BlobsCommand {
    /// List all blobs.
    List,
    /// Get a blob by id.
    Get {
        id: String,
    },
    /// Create a blob from a JSON payload.
    Create {
        /// Raw JSON payload matching `CreateBlob`.
        #[arg(long)]
        json: String,
    },
    /// Update a blob with the given id from a JSON payload.
    Update {
        id: String,
        /// Raw JSON payload matching `CreateBlob`.
        #[arg(long)]
        json: String,
    },
    /// Delete a blob by id.
    Delete {
        id: String,
    },
    /// Get the download URL for a blob's image data.
    DownloadUrl {
        id: String,
    },
}
