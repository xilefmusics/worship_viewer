use clap::{Args, Parser, Subcommand};

use crate::output::OutputFormat;

#[derive(Debug, Parser)]
#[command(name = "worship-viewer", version, about = "CLI for the Worship Viewer REST API")]
pub struct Cli {
    #[arg(long, global = true)]
    pub base_url: Option<String>,

    #[arg(long, global = true)]
    pub sso_session: Option<String>,

    #[arg(long, env = "WORSHIP_VIEWER_BEARER_TOKEN", global = true)]
    pub bearer_token: Option<String>,

    #[arg(long, env = "WORSHIP_VIEWER_OUTPUT", default_value = "auto", global = true)]
    pub output: OutputFormat,

    #[arg(long, global = true)]
    pub dry_run: bool,

    #[arg(long, env = "WORSHIP_VIEWER_TIMEOUT_SECS", global = true)]
    pub timeout_secs: Option<u64>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Schema(SchemaArgs),
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
    Users {
        #[command(subcommand)]
        command: UsersCommand,
    },
    Sessions {
        #[command(subcommand)]
        command: SessionsCommand,
    },
    Songs {
        #[command(subcommand)]
        command: SongsCommand,
    },
    Collections {
        #[command(subcommand)]
        command: CollectionsCommand,
    },
    Setlists {
        #[command(subcommand)]
        command: SetlistsCommand,
    },
    Blobs {
        #[command(subcommand)]
        command: BlobsCommand,
    },
}

#[derive(Debug, Args)]
pub struct SchemaArgs {
    #[command(subcommand)]
    pub command: Option<SchemaCommand>,

    #[arg(long)]
    pub path_prefix: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommand {
    Inspect {
        domain: String,
        action: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommand {
    OtpRequest {
        #[arg(long)]
        json: String,
    },
    OtpVerify {
        #[arg(long)]
        json: String,
    },
    Logout,
}

#[derive(Debug, Subcommand)]
pub enum UsersCommand {
    /// List all users.
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        page_size: Option<u32>,
    },
    /// Get a single user by id.
    Get {
        id: String,
    },
    Create {
        #[arg(long)]
        json: String,
    },
    Delete {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SessionsCommand {
    ListMine,
    GetMine {
        id: String,
    },
    DeleteMine {
        id: String,
    },
    CreateForUser {
        user_id: String,
    },
    ListForUser {
        user_id: String,
    },
    GetForUser {
        user_id: String,
        id: String,
    },
    DeleteForUser {
        user_id: String,
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SongsCommand {
    /// List all songs.
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        page_size: Option<u32>,
    },
    /// Get a song by id.
    Get {
        id: String,
    },
    Player {
        id: String,
    },
    ExportUrl {
        id: String,
        format: String,
    },
    Create {
        #[arg(long)]
        json: String,
    },
    Update {
        id: String,
        #[arg(long)]
        json: String,
    },
    Delete {
        id: String,
    },
    LikeStatus {
        id: String,
    },
    UpdateLikeStatus {
        id: String,
        liked: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum CollectionsCommand {
    /// List all collections.
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        page_size: Option<u32>,
    },
    /// Get a collection by id.
    Get {
        id: String,
    },
    Songs {
        id: String,
    },
    Player {
        id: String,
    },
    ExportUrl {
        id: String,
        format: String,
    },
    Create {
        #[arg(long)]
        json: String,
    },
    Update {
        id: String,
        #[arg(long)]
        json: String,
    },
    Delete {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum SetlistsCommand {
    /// List all setlists.
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        page_size: Option<u32>,
    },
    /// Get a setlist by id.
    Get {
        id: String,
    },
    Songs {
        id: String,
    },
    Player {
        id: String,
    },
    ExportUrl {
        id: String,
        format: String,
    },
    Create {
        #[arg(long)]
        json: String,
    },
    Update {
        id: String,
        #[arg(long)]
        json: String,
    },
    Delete {
        id: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BlobsCommand {
    /// List all blobs.
    List {
        #[arg(long)]
        page: Option<u32>,
        #[arg(long)]
        page_size: Option<u32>,
    },
    /// Get a blob by id.
    Get {
        id: String,
    },
    Create {
        #[arg(long)]
        json: String,
    },
    Update {
        id: String,
        #[arg(long)]
        json: String,
    },
    Delete {
        id: String,
    },
    DownloadUrl {
        id: String,
    },
}
