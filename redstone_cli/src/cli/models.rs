use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author="Pedro Vietro", version="0.0.1", about="TODO", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Track(TrackArgs),
    Clone(CloneArgs),
    Auth,
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
