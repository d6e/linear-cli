use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear issue tracking", version)]
#[command(after_help = "EXAMPLES:
    linear issues --mine              List your assigned issues
    linear issue show ENG-123         Show issue details
    linear issue create -t \"Title\"    Create a new issue
    linear issue close ENG-123        Close an issue
    linear issue comment ENG-123 \"Note\"  Add a comment")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Output as JSON for scripting
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage issues
    #[command(after_help = "EXAMPLES:
    linear issue list --mine --limit 10
    linear issue show ENG-123
    linear issue create -t \"Bug fix\" -d \"Description\" --priority 2
    linear issue update ENG-123 --status \"In Progress\"
    linear issue close ENG-123")]
    Issue {
        #[command(subcommand)]
        action: IssueCommands,
    },
    /// List issues (alias for 'issue list')
    #[command(after_help = "EXAMPLES:
    linear issues --mine
    linear issues --team ENG --status \"In Progress\"
    linear issues --project \"Backend\" --limit 50")]
    Issues(IssueListArgs),
    /// List teams
    #[command(after_help = "EXAMPLES:
    linear teams
    linear teams --json")]
    Teams,
    /// List projects
    #[command(after_help = "EXAMPLES:
    linear projects
    linear projects --team ENG")]
    Projects {
        /// Filter by team key (e.g., ENG)
        #[arg(long)]
        team: Option<String>,
    },
    /// List cycles/sprints
    #[command(after_help = "EXAMPLES:
    linear cycles
    linear cycles --team ENG")]
    Cycles {
        /// Filter by team key (e.g., ENG)
        #[arg(long)]
        team: Option<String>,
    },
    /// List labels
    #[command(after_help = "EXAMPLES:
    linear labels
    linear labels --team ENG")]
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
    #[command(after_help = "EXAMPLES:
    linear issue list --mine
    linear issue list --team ENG --status \"In Progress\"")]
    List(IssueListArgs),
    /// Show issue details
    #[command(after_help = "EXAMPLES:
    linear issue show ENG-123
    linear issue show abc123-uuid-here")]
    Show {
        /// Issue identifier (e.g., ENG-123) or UUID
        id: String,
    },
    /// Create a new issue
    #[command(after_help = "EXAMPLES:
    linear issue create -t \"Fix login bug\"
    linear issue create -t \"New feature\" -d \"Description\" --priority 2")]
    Create(IssueCreateArgs),
    /// Update an existing issue
    #[command(after_help = "EXAMPLES:
    linear issue update ENG-123 --status \"Done\"
    linear issue update ENG-123 --assignee me
    linear issue update ENG-123 --priority 2")]
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

    /// Priority (0=none, 1=urgent, 2=high, 3=medium, 4=low)
    #[arg(long, value_parser = clap::value_parser!(i32).range(0..=4))]
    pub priority: Option<i32>,
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

    /// New priority (0=none, 1=urgent, 2=high, 3=medium, 4=low)
    #[arg(long, value_parser = clap::value_parser!(i32).range(0..=4))]
    pub priority: Option<i32>,

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
