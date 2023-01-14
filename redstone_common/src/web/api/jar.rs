use crate::{config::get_auth_data, model::Result};

use super::get_api_base_url;
use reqwest::cookie::Jar;
/// Cookie Jar methods
use std::sync::Arc;

pub fn get_jar() -> Result<Arc<Jar>> {
    let jar = Jar::default();
    let auth_data = get_auth_data()?;
    match auth_data {
        Some(auth_data) => match auth_data.cookies {
            Some(cookies) => jar.add_cookie_str(cookies.as_str(), &get_api_base_url()),
            _ => (),
        },
        _ => (),
    }
    Ok(Arc::new(jar))
}

pub fn set_cookie(jar: Arc<Jar>, cookie: &str) -> () {
    let url = &get_api_base_url();
    jar.add_cookie_str(cookie, url);
}
