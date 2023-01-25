use reqwest::cookie::{CookieStore, Jar};

use super::model::Result;
use crate::{
    model::{config::AuthData, RedstoneError},
    web::api::get_api_base_url,
};
use std::{path::PathBuf, sync::Arc};

pub fn assert_app_data_folder_is_created() -> Result<()> {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(".redstone");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    {
        let mut dir = dir.clone();
        dir.push("config");
        if !dir.exists() {
            std::fs::File::create(&dir).unwrap();
        }
    }
    {
        let mut dir = dir.clone();
        dir.push("auth");
        if !dir.exists() {
            std::fs::File::create(&dir).unwrap();
        }
    }
    Ok(())
}

pub fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or(RedstoneError::NoHomeDir)
}

#[cfg(feature = "testing")]
pub fn get_auth_dir() -> Result<PathBuf> {
    let mut dir = std::env::temp_dir();
    dir.push("test");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    dir.push("auth");
    if !dir.exists() {
        std::fs::File::create(&dir).unwrap();
    }
    Ok(dir.into())
}

#[cfg(not(feature = "testing"))]
pub fn get_auth_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;
    home_dir.push(".redstone");
    home_dir.push("auth");
    Ok(home_dir)
}

pub fn get_auth_data() -> Result<Option<AuthData>> {
    let auth_dir = get_auth_dir()?;
    let content = std::fs::read_to_string(auth_dir)?;
    if content.len() == 0 {
        return Ok(None);
    }
    Ok(bincode::deserialize(content.as_bytes())?)
}

pub fn store_cookies(cookie_jar: Arc<Jar>) -> Result<()> {
    let base_url = get_api_base_url();
    let auth_data = String::from(cookie_jar.cookies(&base_url).unwrap().to_str().unwrap());
    let data = bincode::serialize(&Some(AuthData::new(auth_data))).unwrap();

    std::fs::write(get_auth_dir()?, data).unwrap();
    Ok(())
}
