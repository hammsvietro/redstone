use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthData {
    pub cookies: Option<String>,
}

impl AuthData {
    pub fn new(cookies: String) -> Self {
        Self {
            cookies: Some(cookies),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub url: String,
}

impl ServerConfig {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}
