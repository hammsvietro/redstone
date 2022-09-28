use std::io::Write;

use redstone_common::{api::{AuthRequest, jar::get_jar, get_api_base_url}, config::{set_auth_data,get_auth_data}, model::config::AuthData};
use reqwest::cookie::CookieStore;

pub fn run_auth_cmd() -> std::io::Result<()> {
    let auth_request = prompt_credentials()?;
    let base_url = get_api_base_url();
    let cookie_jar = get_jar()?;

    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_jar.clone())
        .build()
        .unwrap();
    let res = client
        .post(base_url.join("/api/login").unwrap())
        .json(&auth_request)
        .send();

    if let Err(_) = res {
        println!("Something went wrong");
        return Ok(());
    }
    let res = res.unwrap();
    println!("{:?}", res.status());
    if res.status() != reqwest::StatusCode::OK {
        println!("Couldn't login\nCheck if the server url ({}) or the email and password provided is correct", &base_url.as_str());
        return Ok(())
    }
    let auth_cookies = String::from(cookie_jar.cookies(&base_url).unwrap().to_str().unwrap());
    set_auth_data(AuthData::new(auth_cookies))?;
    println!("Successfully authenticated!");
    Ok(())
}

fn prompt_credentials() -> std::io::Result<AuthRequest> {
    print!("E-mail: ");
    std::io::stdout().flush()?;
    let mut buffer = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut buffer)?;
    let email = buffer.trim();
    let password = rpassword::prompt_password("Password: ")?;
    Ok(AuthRequest::new(String::from(email), password))
}
