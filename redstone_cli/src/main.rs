mod cli;
mod ipc;
mod utils;

use colored::Colorize;
use redstone_common::{config::assert_app_data_folder_is_created, model::Result};

fn main() -> Result<()> {
    assert_app_data_folder_is_created()?;
    if let Err(err) = cli::input() {
        eprintln!("{}", err.to_string().red());
    }
    Ok(())
}
