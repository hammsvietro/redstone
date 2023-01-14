use std::io::Write;

use redstone_common::{
    config::store_cookies,
    model::{api::Endpoints, RedstoneError, Result},
    web::api::{jar::get_jar, AuthRequest},
};

pub fn run_auth_cmd() -> Result<()> {
    let auth_request = prompt_credentials()?;
    let cookie_jar = get_jar()?;

    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_jar.clone())
        .build()
        .unwrap();
    let res = client
        .post(Endpoints::Login.get_url())
        .json(&auth_request)
        .send()?;

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
