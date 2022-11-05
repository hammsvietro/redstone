mod cli;
mod ipc;
mod utils;

use redstone_common::{config::assert_app_data_folder_is_created, model::Result};

fn main() -> Result<()> {
    assert_app_data_folder_is_created()?;
    cli::input()
}
