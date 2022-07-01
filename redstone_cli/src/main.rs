
mod cli;
mod ipc;
mod utils;

use redstone_common::config::assert_app_data_folder_is_created;

fn main() -> std::io::Result<()> {
    assert_app_data_folder_is_created()?;
    cli::input()
}
