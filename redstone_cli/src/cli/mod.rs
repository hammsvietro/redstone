mod auth;
pub mod models;
mod track;
mod clone;

use clap::Parser;
use models::{Cli, Commands};
use redstone_common::web::api::{jar::get_jar, RedstoneBlockingClient};

pub fn input() -> redstone_common::model::Result<()> {
    let args = Cli::parse();
    let jar = get_jar()?;
    let client = RedstoneBlockingClient::new(jar);
    match args.command {
        Commands::Auth => auth::run_auth_cmd(client),
        Commands::Track(track_args) => track::run_track_cmd(track_args),
        Commands::Clone(clone_args) => clone::run_clone_cmd(clone_args)
    }
}
