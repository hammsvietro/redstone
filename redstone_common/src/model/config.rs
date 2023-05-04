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
    pub hostname: String,
    pub use_https: bool,
    pub port: usize,
}

impl ServerConfig {
    pub fn new(hostname: String, port: usize, use_https: bool) -> Self {
        Self {
            hostname,
            use_https,
            port,
        }
    }
}
