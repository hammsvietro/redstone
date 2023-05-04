pub mod models;

mod auth;
mod clone;
mod progress_bar;
mod pull;
mod push;
mod server_config;
mod status;
mod track;

use clap::Parser;
use models::{Cli, Commands};
use redstone_common::web::api::RedstoneBlockingClient;

pub fn input() -> redstone_common::model::Result<()> {
    let cmd = Cli::parse();
    match cmd.command {
        Commands::Auth => {
            let client = RedstoneBlockingClient::new();
            auth::run_auth_cmd(client)
        }
        Commands::Clone(clone_args) => clone::run_clone_cmd(clone_args),
        Commands::Pull => pull::run_pull_cmd(),
        Commands::Push => push::run_push_cmd(),
        Commands::ServerConfig(set_server_args) => {
            server_config::run_server_config(set_server_args)
        }
        Commands::Status => status::run_status_cmd(),
        Commands::Track(track_args) => track::run_track_cmd(track_args),
    }
}
