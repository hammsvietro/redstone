use reqwest::Url;
use serde::Serialize;
pub mod jar;

#[derive(Serialize, Debug)]
pub struct AuthRequest {
    pub email: String,
    pub password: String
}

impl AuthRequest {
    pub fn new(email: String, password: String) -> Self {
        AuthRequest { email, password }
    }
}

const API_BASE_URL: &'static str = "http://localhost:4000";

pub fn get_api_base_url() -> Url {
    API_BASE_URL.parse().unwrap()
}
