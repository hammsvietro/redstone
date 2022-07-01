use std::{path::PathBuf, io::ErrorKind};
use crate::model::config::AuthData;

pub fn assert_app_data_folder_is_created() -> std::io::Result<()> {
    let mut dir = dirs::home_dir().unwrap();
    dir.push(".redstone");
    std::fs::create_dir_all(&dir)?;
    {
        let mut dir = dir.clone();
        dir.push("config");
        std::fs::File::create(&dir).unwrap();
    }
    {
        let mut dir = dir.clone();
        dir.push("auth");
        std::fs::File::create(&dir).unwrap();
    }
    Ok(())
}

pub fn get_home_dir() -> std::io::Result<PathBuf> {
    match dirs::home_dir() {
        None => Err(std::io::Error::new(ErrorKind::NotFound, "Couldn't open home dir.")),
        Some(pathbuf) => Ok(pathbuf)
    }
}

pub fn get_auth_dir() -> std::io::Result<PathBuf> {
    match get_home_dir() {
        Ok(mut dir) => {
            dir.push(".redstone");
            dir.push("auth");
            return Ok(dir);
        }
        Err(err) => Err(err)
    }
}

pub fn get_auth_data() -> std::io::Result<Option<AuthData>> {
    let auth_dir = get_auth_dir()?; 
    println!("auth_dir: {auth_dir:?}");
    let content = std::fs::read_to_string(auth_dir)?;
    if content.len() == 0 {
        return Ok(None)
    }
    match bincode::deserialize(&content.as_bytes()) {
        Err(err) => {
            let error = std::io::Error::new(ErrorKind::Other, err.to_string());
            return Err(error);

        }
        Ok(auth_data) => Ok(auth_data)
    }
}

pub fn set_auth_data(auth_data: AuthData) -> std::io::Result<()> {
    let auth_dir = get_auth_dir()?;
    let data = &bincode::serialize(&auth_data).unwrap();
    std::fs::write(auth_dir, &data).unwrap();
    Ok(())
}
