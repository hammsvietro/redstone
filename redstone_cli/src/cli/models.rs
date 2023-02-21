use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author="Pedro Vietro", version="0.0.1", about="Redstone is a Self-hosted CLI backup tool ", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Authenticate by using your email and password
    Auth,

    /// Clone a backup by providing the backup name
    Clone(CloneArgs),

    /// Push the changes made to the server
    Push,

    /// Recursively scan a directory and create a backup with all the files scanned
    Track(TrackArgs),

    /// Check for changes in the current bakcup
    Status,

    /// Pull the latest changes from the server
    Pull,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct CloneArgs {
    pub backup_name: String,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct TrackArgs {
    pub backup_name: String,

    pub path: Option<String>,

    #[clap(long, name = "sync-every")]
    pub sync_every: Option<String>,

    #[clap(long, conflicts_with = "sync-every")]
    pub watch: bool,

    #[clap(long, short = 'd')]
    pub detached: bool,
}
