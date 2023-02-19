pub mod models;

mod auth;
mod clone;
mod push;
mod track;

use clap::Parser;
use models::{Cli, Commands};
use redstone_common::web::api::RedstoneBlockingClient;

pub fn input() -> redstone_common::model::Result<()> {
    let args = Cli::parse();
    let client = RedstoneBlockingClient::new();
    match args.command {
        Commands::Auth => auth::run_auth_cmd(client),
        Commands::Clone(clone_args) => clone::run_clone_cmd(clone_args),
        Commands::Push => push::run_push_cmd(),
        Commands::Track(track_args) => track::run_track_cmd(track_args),
    }
}
