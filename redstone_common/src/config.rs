use super::model::Result;
use crate::model::{config::AuthData, RedstoneError};
use std::path::PathBuf;

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
    Ok(bincode::deserialize(&content.as_bytes())?)
}

pub fn set_auth_data(auth_data: AuthData) -> Result<()> {
    let auth_dir = get_auth_dir()?;
    let data = &bincode::serialize(&Some(auth_data)).unwrap();
    std::fs::write(auth_dir, &data).unwrap();
    Ok(())
}
