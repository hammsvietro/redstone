mod api;
mod backup;
mod ipc;
mod scheduler;
mod service;

use ipc::assert_socket_is_available;
use redstone_common::{config::assert_app_data_folder_is_created, model::Result};

#[tokio::main]
async fn main() -> Result<()> {
    assert_resources_are_ready()?;
    service::run_service().await
}

fn assert_resources_are_ready() -> Result<()> {
    assert_socket_is_available();
    assert_app_data_folder_is_created()?;
    Ok(())
}
