use std::{collections::HashMap, sync::Arc};

use crate::model::{RedstoneError, Result};

use async_trait::async_trait;
use reqwest::{cookie::Jar, Method, Url};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use self::jar::get_jar;
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

const API_BASE_URL: &str = "http://127.0.0.1:4000"; // TODO: Change this

//
// NON-BLOCKING
//

#[async_trait]
pub trait HttpSend {
    async fn send(
        &self,
        request: reqwest::RequestBuilder,
        client: &reqwest::Client,
    ) -> Result<reqwest::Response>;
}

pub struct Sender;

#[async_trait]
impl HttpSend for Sender {
    async fn send(
        &self,
        request: reqwest::RequestBuilder,
        _client: &reqwest::Client,
    ) -> Result<reqwest::Response> {
        Ok(request.send().await?)
    }
}

pub struct RedstoneClient<S: HttpSend = Sender> {
    pub jar: Arc<Jar>,
    client: reqwest::Client,
    sender: S,
}

impl RedstoneClient<Sender> {
    pub fn new() -> Self {
        let jar = get_jar();
        Self {
            client: get_http_client(jar.clone()),
            jar,
            sender: Sender,
        }
    }

    pub fn with_custom_jar(jar: Arc<Jar>) -> Self {
        Self {
            client: get_http_client(jar.clone()),
            jar,
            sender: Sender,
        }
    }

    pub async fn send<T>(
        &self,
        method: Method,
        url: Url,
        body: &Option<T>,
    ) -> Result<reqwest::Response>
    where
        T: Serialize,
    {
        let mut request = self.client.request(method, url);
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = self.sender.send(request, &self.client).await?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(RedstoneError::Unauthorized);
        }
        Ok(response)
    }
}

impl<S: HttpSend> RedstoneClient<S> {
    pub fn with_sender(sender: S, jar: Arc<Jar>) -> Self {
        Self {
            client: get_http_client(jar.clone()),
            jar,
            sender,
        }
    }
}

//
// BLOCKING
//

pub trait BlockingHttpSend {
    fn send(
        &self,
        request: reqwest::blocking::RequestBuilder,
        client: &reqwest::blocking::Client,
    ) -> Result<reqwest::blocking::Response>;
}

pub struct BlockingSender;

impl BlockingHttpSend for BlockingSender {
    fn send(
        &self,
        request: reqwest::blocking::RequestBuilder,
        _client: &reqwest::blocking::Client,
    ) -> Result<reqwest::blocking::Response> {
        Ok(request.send()?)
    }
}

pub struct RedstoneBlockingClient<S: BlockingHttpSend = BlockingSender> {
    pub jar: Arc<Jar>,
    client: reqwest::blocking::Client,
    sender: S,
}

impl<S: BlockingHttpSend> RedstoneBlockingClient<S> {
    pub fn with_sender(sender: S, jar: Arc<Jar>) -> Self {
        Self {
            client: get_blocking_http_client(jar.clone()),
            jar,
            sender,
        }
    }

    pub fn send<T>(
        &self,
        method: Method,
        url: Url,
        body: &Option<T>,
    ) -> Result<reqwest::blocking::Response>
    where
        T: Serialize,
    {
        let mut request = self.client.request(method, url);
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = self.sender.send(request, &self.client)?;
        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(RedstoneError::Unauthorized);
        }
        Ok(response)
    }
}

impl RedstoneBlockingClient<BlockingSender> {
    pub fn new() -> Self {
        let jar = get_jar();
        Self {
            client: get_blocking_http_client(jar.clone()),
            jar,
            sender: BlockingSender,
        }
    }

    pub fn with_jar(jar: Arc<Jar>) -> Self {
        Self {
            client: get_blocking_http_client(jar.clone()),
            jar,
            sender: BlockingSender,
        }
    }
}

pub fn get_api_base_url() -> Url {
    API_BASE_URL.parse().unwrap()
}

fn get_blocking_http_client(cookie_jar: Arc<Jar>) -> reqwest::blocking::Client {
    reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_jar)
        .build()
        .unwrap()
}

fn get_http_client(cookie_jar: Arc<Jar>) -> reqwest::Client {
    reqwest::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_jar)
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
            "Error while making request with the API:\nStatus code: {status_code}"
        )));
    }
    Ok(response.json::<T>().await?)
}
