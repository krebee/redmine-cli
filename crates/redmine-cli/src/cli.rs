use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "redmine-cli")]
#[command(version)]
#[command(about = "Agent-friendly Redmine CLI")]
pub struct Cli {
    #[arg(long, global = true)]
    pub json: bool,

    #[arg(long, global = true, value_enum, default_value = "json")]
    pub format: OutputFormat,

    #[arg(long, global = true)]
    pub profile: Option<String>,

    #[arg(long, global = true, default_value_t = 30000)]
    pub timeout_ms: u64,

    #[arg(long, global = true)]
    pub ssl_no_revoke: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Json,
    Text,
    Table,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Config(ConfigCommand),
    Projects(ProjectsCommand),
    Issues(IssuesCommand),
}

#[derive(Debug, Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    Init {
        #[arg(long)]
        url: String,

        #[arg(long, default_value = "REDMINE_API_KEY")]
        api_key_env: String,

        #[arg(long, default_value = "default")]
        profile: String,

        #[arg(long)]
        default_project: Option<String>,

        #[arg(long)]
        ssl_no_revoke: bool,

        #[arg(long)]
        dry_run: bool,
    },
    Show,
}

#[derive(Debug, Args)]
pub struct ProjectsCommand {
    #[command(subcommand)]
    pub command: ProjectsSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum ProjectsSubcommand {
    List {
        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    Get {
        project_id: String,
    },
}

#[derive(Debug, Args)]
pub struct IssuesCommand {
    #[command(subcommand)]
    pub command: IssuesSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum IssuesSubcommand {
    Get {
        issue_id: u64,
    },
    List {
        #[arg(long)]
        project: Option<String>,

        #[arg(long)]
        status: Option<String>,

        #[arg(long, default_value_t = 100)]
        limit: u32,
    },
    Create {
        #[arg(long)]
        project: String,

        #[arg(long)]
        subject: String,

        #[arg(long)]
        description: Option<String>,

        #[arg(long)]
        description_file: Option<std::path::PathBuf>,

        #[arg(long)]
        tracker_id: Option<u64>,

        #[arg(long)]
        status_id: Option<u64>,

        #[arg(long)]
        priority_id: Option<u64>,

        #[arg(long)]
        assigned_to_id: Option<u64>,

        #[arg(long)]
        dry_run: bool,
    },
    Update {
        issue_id: u64,

        #[arg(long)]
        subject: Option<String>,

        #[arg(long)]
        description: Option<String>,

        #[arg(long)]
        status_id: Option<u64>,

        #[arg(long)]
        priority_id: Option<u64>,

        #[arg(long)]
        assigned_to_id: Option<u64>,

        #[arg(long)]
        notes: Option<String>,

        #[arg(long)]
        dry_run: bool,
    },
    Comment {
        issue_id: u64,

        #[arg(long)]
        notes: String,

        #[arg(long)]
        dry_run: bool,
    },
}
