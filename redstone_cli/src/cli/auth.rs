use std::io::Write;

use redstone_common::api::{AuthRequest, jar::get_jar, get_api_base_url};

pub fn run_auth_cmd() -> std::io::Result<()> {
    let auth_request = prompt_credentials()?;
    println!("req: {auth_request:?}");
    let base_url = get_api_base_url();
    let cookie_jar = get_jar()?;
    let client = reqwest::blocking::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_jar.clone())
        .build().unwrap();
    let res = client
        .get(base_url.join("/api/auth_test").unwrap())
        .send();

    println!("res: {:?}", res);
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
