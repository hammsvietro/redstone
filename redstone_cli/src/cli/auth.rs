use std::io::Write;

use redstone_common::{
    config::store_cookies,
    model::{api::Endpoints, RedstoneError, Result},
    web::api::{jar::get_jar, AuthRequest, RedstoneBlockingClient},
};
use reqwest::Method;

pub fn run_auth_cmd() -> Result<()> {
    let auth_request = prompt_credentials()?;
    let cookie_jar = get_jar()?;

    let client = RedstoneBlockingClient::new(cookie_jar.clone());
    let res = client.send_json(
        Method::POST,
        Endpoints::Login.get_url(),
        &Some(auth_request),
    )?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(RedstoneError::BaseError(String::from(
            "Incorrect Credentails",
        )));
    }

    store_cookies(cookie_jar)?;
    println!("Successfully authenticated!");
    Ok(())
}

fn prompt_credentials() -> Result<AuthRequest> {
    print!("E-mail: ");
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buffer)?;
    let email = buffer.trim();
    let password = rpassword::prompt_password("Password: ")?;
    Ok(AuthRequest::new(String::from(email), password))
}
