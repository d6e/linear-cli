use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "linear")]
#[command(about = "A CLI for Linear issue tracking", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage issues
    Issue {
        #[command(subcommand)]
        action: IssueCommands,
    },
    /// List issues (alias for 'issue list')
    Issues(IssueListArgs),
    /// List teams
    Teams,
    /// List projects
    Projects {
        /// Filter by team key (e.g., ENG)
        #[arg(long)]
        team: Option<String>,
    },
    /// List cycles/sprints
    Cycles {
        /// Filter by team key (e.g., ENG)
        #[arg(long)]
        team: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum IssueCommands {
    /// List issues
    List(IssueListArgs),
    /// Show issue details
    Show {
        /// Issue identifier (e.g., ENG-123)
        id: String,
    },
    /// Create a new issue
    Create(IssueCreateArgs),
    /// Update an existing issue
    Update(IssueUpdateArgs),
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

    /// Maximum number of issues to show
    #[arg(long, short, default_value = "25")]
    pub limit: u32,
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
    #[arg(long)]
    pub priority: Option<i32>,
}

#[derive(Args)]
pub struct IssueUpdateArgs {
    /// Issue identifier (e.g., ENG-123)
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

    /// New priority
    #[arg(long)]
    pub priority: Option<i32>,

    /// Assign to user (email or "me")
    #[arg(long)]
    pub assignee: Option<String>,
}
