use redstone_common::{
    model::{
        api::{Endpoints, Update},
        Result,
    },
    web::api::{handle_response, RedstoneClient},
};
use reqwest::Method;

pub async fn check_latest_update(backup_id: String) -> Result<Update> {
    let client = RedstoneClient::new();
    let latest_update_response = client
        .send::<()>(
            Method::GET,
            Endpoints::FetchUpdate(backup_id.to_owned()).get_url()?,
            &None,
        )
        .await?;

    let latest_update: Update = handle_response(latest_update_response).await?;
    Ok(latest_update)
}
