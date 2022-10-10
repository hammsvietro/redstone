use std::sync::Arc;

use reqwest::{cookie::Jar, Url};
use serde::Serialize;
pub mod jar;

#[derive(Serialize, Debug)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
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

pub fn get_http_client(cookie_jar: Arc<Jar>) -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_jar.clone())
        .build()
        .unwrap()
}

pub fn get_blocking_http_client(cookie_jar: Arc<Jar>) -> reqwest::blocking::Client {
    reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_jar.clone())
        .build()
        .unwrap()
}
