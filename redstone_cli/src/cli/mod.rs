pub mod models;
mod auth;
mod track;

use clap::Parser;
use models::{Cli, Commands};

pub fn input() -> std::io::Result<()> {
    let args = Cli::parse();
    match args.command {
        Commands::Auth => auth::run_auth_cmd(),
        Commands::Track(track_args) => track::run_track_cmd(track_args),
    }
}
