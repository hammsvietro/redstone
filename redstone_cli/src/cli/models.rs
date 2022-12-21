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
    Auth,
}

#[derive(Debug, Args)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct TrackArgs {
    pub name: String,

    pub path: Option<String>,
    // #[clap(long, name="replication-count")]
    // pub replication_count: Option<u16>,
    #[clap(long, name = "sync-every")]
    pub sync_every: Option<String>,

    #[clap(long, conflicts_with = "sync-every")]
    pub watch: bool,

    #[clap(long, short = 'd')]
    pub detached: bool,

    #[clap(long)]
    pub dry_run: bool,
}
