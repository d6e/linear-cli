use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::types::Priority;

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Compact,
}

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear issue tracking", version)]
#[command(after_help = "EXAMPLES:
    linear issues --mine              List your assigned issues
    linear issue view ENG-123         View issue details
    linear issue create -t \"Title\"    Create a new issue
    linear issue close ENG-123        Close an issue
    linear issue comment ENG-123 \"Note\"  Add a comment")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output format (table, json, compact)
    #[arg(long, short = 'o', global = true, value_enum, default_value = "table")]
    pub format: OutputFormat,

    /// Output as JSON (alias for --format json)
    #[arg(long, global = true, hide = true)]
    pub json: bool,

    /// Suppress success messages
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Show detailed error information
    #[arg(long, short, global = true)]
    pub verbose: bool,
}

impl Cli {
    /// Get the effective output format, considering --json flag
    pub fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            self.format
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage issues
    #[command(
        alias = "i",
        after_help = "EXAMPLES:
    linear issue list --mine --limit 10
    linear issue view ENG-123
    linear issue create -t \"Bug fix\" -d \"Description\" --priority 2
    linear issue update ENG-123 --status \"In Progress\"
    linear issue close ENG-123"
    )]
    Issue {
        #[command(subcommand)]
        action: IssueCommands,
    },
    /// List issues (alias for 'issue list')
    #[command(
        alias = "is",
        after_help = "EXAMPLES:
    linear issues --mine
    linear issues --team ENG --status \"In Progress\"
    linear issues --project \"Backend\" --limit 50"
    )]
    Issues(IssueListArgs),
    /// List teams
    #[command(
        alias = "t",
        after_help = "EXAMPLES:
    linear teams
    linear teams --format json"
    )]
    Teams,
    /// List projects
    #[command(
        alias = "p",
        after_help = "EXAMPLES:
    linear projects
    linear projects --team ENG"
    )]
    Projects {
        /// Filter by team key (e.g., ENG)
        #[arg(long)]
        team: Option<String>,
    },
    /// Manage cycles/sprints
    #[command(after_help = "EXAMPLES:
    linear cycle list
    linear cycle list --team ENG
    linear cycle view abc123")]
    Cycle {
        #[command(subcommand)]
        action: CycleCommands,
    },
    /// List cycles (alias for 'cycle list')
    #[command(after_help = "EXAMPLES:
    linear cycles
    linear cycles --team ENG")]
    Cycles(CycleListArgs),
    /// List labels
    #[command(
        alias = "l",
        after_help = "EXAMPLES:
    linear labels
    linear labels --team ENG"
    )]
    Labels {
        /// Filter by team key (e.g., ENG)
        #[arg(long)]
        team: Option<String>,
    },
    /// Generate shell completions
    #[command(after_help = "EXAMPLES:
    linear completions bash > ~/.bash_completion.d/linear
    linear completions zsh > ~/.zfunc/_linear
    linear completions fish > ~/.config/fish/completions/linear.fish")]
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Initialize configuration file interactively
    #[command(after_help = "EXAMPLES:
    linear init")]
    Init,
}

#[derive(Subcommand)]
pub enum IssueCommands {
    /// List issues
    #[command(
        alias = "ls",
        after_help = "EXAMPLES:
    linear issue list --mine
    linear issue list --team ENG --status \"In Progress\""
    )]
    List(IssueListArgs),
    /// View issue details
    #[command(
        alias = "v",
        after_help = "EXAMPLES:
    linear issue view ENG-123
    linear issue view abc123-uuid-here"
    )]
    View {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
    },
    /// Create a new issue
    #[command(
        alias = "c",
        after_help = "EXAMPLES:
    linear issue create -t \"Fix login bug\"
    linear issue create -t \"New feature\" -d \"Description\" --priority 2"
    )]
    Create(IssueCreateArgs),
    /// Update an existing issue
    #[command(
        alias = "u",
        after_help = "EXAMPLES:
    linear issue update ENG-123 --status \"Done\"
    linear issue update ENG-123 --assignee me
    linear issue update ENG-123 --priority 2"
    )]
    Update(IssueUpdateArgs),
    /// Close an issue (set status to Done/Completed)
    #[command(after_help = "EXAMPLES:
    linear issue close ENG-123")]
    Close {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
    },
    /// List attachments on an issue
    #[command(after_help = "EXAMPLES:
    linear issue attachments ENG-123")]
    Attachments {
        /// Issue identifier (e.g., ENG-123)
        id: String,
    },
    /// Attach a URL to an issue
    #[command(after_help = "EXAMPLES:
    linear issue attach ENG-123 https://example.com
    linear issue attach ENG-123 https://example.com -t \"Reference\"")]
    Attach(AttachUrlArgs),
    /// Upload a file and attach it to an issue
    #[command(after_help = "EXAMPLES:
    linear issue upload ENG-123 ./screenshot.png
    linear issue upload ENG-123 ./doc.pdf -t \"Documentation\"")]
    Upload(UploadFileArgs),
    /// List comments on an issue
    #[command(after_help = "EXAMPLES:
    linear issue comments ENG-123")]
    Comments {
        /// Issue identifier (e.g., ENG-123)
        id: String,
    },
    /// Add a comment to an issue
    #[command(after_help = "EXAMPLES:
    linear issue comment ENG-123 \"This is a comment\"")]
    Comment(CommentArgs),
}

#[derive(Subcommand)]
pub enum CycleCommands {
    /// List cycles
    #[command(
        alias = "ls",
        after_help = "EXAMPLES:
    linear cycle list
    linear cycle list --team ENG"
    )]
    List(CycleListArgs),
    /// View cycle details
    #[command(
        alias = "v",
        after_help = "EXAMPLES:
    linear cycle view abc123-uuid"
    )]
    View {
        /// Cycle UUID
        id: String,
    },
}

#[derive(Args, Clone)]
pub struct CycleListArgs {
    /// Filter by team key (e.g., ENG)
    #[arg(long)]
    pub team: Option<String>,
}

#[derive(Args, Clone)]
pub struct IssueListArgs {
    /// Show only my issues
    #[arg(long)]
    pub mine: bool,

    /// Filter by team key (e.g., ENG)
    #[arg(long)]
    pub team: Option<String>,

    /// Filter by status name
    #[arg(long)]
    pub status: Option<String>,

    /// Filter by project name
    #[arg(long)]
    pub project: Option<String>,

    /// Filter by label name
    #[arg(long)]
    pub label: Option<String>,

    /// Filter by cycle name
    #[arg(long)]
    pub cycle: Option<String>,

    /// Maximum number of issues to show (default: 25, max: 250)
    #[arg(long, short, default_value = "25")]
    pub limit: u32,

    /// Fetch all results (may be slow for large result sets)
    #[arg(long)]
    pub all: bool,
}

#[derive(Args)]
pub struct IssueCreateArgs {
    /// Issue title
    #[arg(long, short)]
    pub title: String,

    /// Issue description
    #[arg(long, short)]
    pub description: Option<String>,

    /// Team key (uses default if not specified)
    #[arg(long)]
    pub team: Option<String>,

    /// Project name
    #[arg(long)]
    pub project: Option<String>,

    /// Priority level
    #[arg(long, value_enum)]
    pub priority: Option<Priority>,
}

#[derive(Args)]
pub struct IssueUpdateArgs {
    /// Issue identifier (e.g., ENG-123) or UUID
    pub id: String,

    /// New title
    #[arg(long)]
    pub title: Option<String>,

    /// New description
    #[arg(long)]
    pub description: Option<String>,

    /// New status
    #[arg(long)]
    pub status: Option<String>,

    /// New priority level
    #[arg(long, value_enum)]
    pub priority: Option<Priority>,

    /// Assign to user (ID or "me")
    #[arg(long)]
    pub assignee: Option<String>,
}

#[derive(Args)]
pub struct AttachUrlArgs {
    /// Issue identifier (e.g., ENG-123) or UUID
    pub id: String,

    /// URL to attach
    pub url: String,

    /// Title for the attachment
    #[arg(long, short)]
    pub title: Option<String>,
}

#[derive(Args)]
pub struct UploadFileArgs {
    /// Issue identifier (e.g., ENG-123) or UUID
    pub id: String,

    /// Path to file to upload
    pub file: String,

    /// Title for the attachment (defaults to filename)
    #[arg(long, short)]
    pub title: Option<String>,
}

#[derive(Args)]
pub struct CommentArgs {
    /// Issue identifier (e.g., ENG-123) or UUID
    pub id: String,

    /// Comment body (markdown supported)
    pub body: String,
}
