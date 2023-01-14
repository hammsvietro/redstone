use std::{collections::HashMap, sync::Arc};

use crate::model::{RedstoneError, Result};

use reqwest::{cookie::Jar, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub mod jar;

#[derive(Serialize, Debug)]
pub struct AuthRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiErrorResponse {
    pub errors: HashMap<String, Vec<String>>,
    pub stringified_errors: String,
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

pub async fn handle_response<T: DeserializeOwned>(response: reqwest::Response) -> Result<T> {
    let status_code = response.status();
    if status_code != reqwest::StatusCode::OK {
        if let Ok(parsed_error) = response.json::<ApiErrorResponse>().await {
            return Err(RedstoneError::ApiError(parsed_error));
        }
        return Err(RedstoneError::BaseError(format!(
            "Error while making request with the API:\nStatus code: {}",
            status_code
        )));
    }
    Ok(response.json::<T>().await?)
}
