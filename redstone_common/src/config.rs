use regex::Regex;
use reqwest::cookie::{CookieStore, Jar};

use super::model::Result;
use crate::{
    model::{
        config::{AuthData, ServerConfig},
        DomainError, RedstoneError,
    },
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
        dir.push("server_config");
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
    Ok(dir)
}

#[cfg(not(feature = "testing"))]
pub fn get_auth_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;
    home_dir.push(".redstone");
    home_dir.push("auth");
    Ok(home_dir)
}

#[cfg(feature = "testing")]
pub fn get_server_config_dir() -> Result<PathBuf> {
    let mut dir = std::env::temp_dir();
    dir.push("test");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    dir.push("server_config");
    if !dir.exists() {
        std::fs::File::create(&dir).unwrap();
    }
    Ok(dir)
}

#[cfg(not(feature = "testing"))]
pub fn get_server_config_dir() -> Result<PathBuf> {
    let mut home_dir = get_home_dir()?;
    home_dir.push(".redstone");
    home_dir.push("server_config");
    Ok(home_dir)
}

pub fn get_auth_data() -> Result<Option<AuthData>> {
    let auth_dir = get_auth_dir()?;
    if !auth_dir.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(auth_dir)?;
    if content.is_empty() {
        return Ok(None);
    }
    Ok(bincode::deserialize(content.as_bytes())?)
}

pub fn store_cookies(cookie_jar: Arc<Jar>) -> Result<()> {
    let base_url = get_api_base_url()?;
    let auth_data = String::from(cookie_jar.cookies(&base_url).unwrap().to_str().unwrap());
    let data = bincode::serialize(&Some(AuthData::new(auth_data)))?;

    std::fs::write(get_auth_dir()?, data).unwrap();
    Ok(())
}

pub fn get_server_config() -> Result<Option<ServerConfig>> {
    let config_dir = get_server_config_dir()?;
    if !config_dir.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(config_dir)?;
    if content.is_empty() {
        return Ok(None);
    }
    Ok(bincode::deserialize(content.as_bytes())?)
}

pub fn store_server_config(mut server_address: String) -> Result<()> {
    let re = Regex::new("^http(s?)://").unwrap();
    if !re.is_match(&server_address) {
        server_address = format!("http://{server_address}");
    }
    let data = bincode::serialize(&Some(ServerConfig::new(server_address)))?;
    std::fs::write(get_server_config_dir()?, data).unwrap();
    Ok(())
}

pub fn assert_configuration() -> Result<()> {
    if get_server_config()?.is_none() {
        return Err(RedstoneError::DomainError(DomainError::NoServerConfigFound));
    }
    Ok(())
}

pub fn assert_configuration_and_authentication() -> Result<()> {
    if get_auth_data()?.is_none() {
        return Err(RedstoneError::DomainError(DomainError::NotAuthenticated));
    }
    assert_configuration()?;
    Ok(())
}
