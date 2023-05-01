use redstone_common::{config::store_server_config, model::Result};

use super::models::SetServerAddressArgs;

pub fn run_set_server_address_args(args: SetServerAddressArgs) -> Result<()> {
    store_server_config(args.address)?;
    Ok(())
}
