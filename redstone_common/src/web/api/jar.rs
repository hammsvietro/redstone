use super::get_api_base_url;
/// Cookie Jar methods
use crate::{config::get_auth_data, model::Result};
use reqwest::cookie::Jar;
use std::sync::Arc;

pub fn get_jar() -> Result<Arc<Jar>> {
    let jar = Jar::default();
    let auth_data = get_auth_data()?;
    if let Some(auth_data) = auth_data {
        if let Some(cookies) = auth_data.cookies {
            jar.add_cookie_str(cookies.as_str(), &get_api_base_url());
        }
    }
    Ok(Arc::new(jar))
}
