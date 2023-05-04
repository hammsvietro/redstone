use redstone_common::{
    config::store_server_config,
    model::{config::ServerConfig, Result},
};

use super::models::ServerConfigArgs;

pub fn run_server_config(args: ServerConfigArgs) -> Result<()> {
    store_server_config(ServerConfig::new(args.address, args.port, args.use_https))?;
    Ok(())
}
