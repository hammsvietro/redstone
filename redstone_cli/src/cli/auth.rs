use std::io::Write;

use redstone_common::{
    config::store_cookies,
    model::{api::Endpoints, RedstoneError, Result},
    web::api::{AuthRequest, BlockingHttpSend, RedstoneBlockingClient},
};
use reqwest::Method;

pub fn run_auth_cmd(client: RedstoneBlockingClient) -> Result<()> {
    let auth_request = prompt_credentials()?;
    login(auth_request, client)
}

fn login<S: BlockingHttpSend>(
    auth_request: AuthRequest,
    client: RedstoneBlockingClient<S>,
) -> Result<()> {
    let res = client.send(
        Method::POST,
        Endpoints::Login.get_url(),
        &Some(auth_request),
    )?;

    if res.status() != reqwest::StatusCode::OK {
        return Err(RedstoneError::BaseError(String::from(
            "Incorrect Credentails",
        )));
    }

    store_cookies(client.jar)?;
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use httpmock::prelude::*;
    use redstone_common::{
        config::{get_auth_data, get_auth_dir},
        web::api::BlockingHttpSend,
    };
    use reqwest::cookie::Jar;

    use super::*;

    struct AuthMockSender {
        pub is_success: bool,
    }

    struct TestCleaner;

    impl Drop for TestCleaner {
        fn drop(&mut self) {
            let path = get_auth_dir().unwrap();
            std::fs::remove_file(path).unwrap();
        }
    }

    impl BlockingHttpSend for AuthMockSender {
        fn send(
            &self,
            _request: reqwest::blocking::RequestBuilder,
            client: &reqwest::blocking::Client,
        ) -> Result<reqwest::blocking::Response> {
            let server = MockServer::start();
            if self.is_success {
                let api_mock = server.mock(|when, then| {
                    when.method(POST)
                        .path("/api/auth");
                    then.status(200)
                        .header("content-type", "text/html")
                        .header("set-cookie", "_redstone_server_key=SFMyNTY.g3QAAAABbQAAAAp1c2VyX3Rva2VubQAAACAd6VBb3CeXpExUMxzSInfAWclBhjo7K413049XYe7EnA.8Rauz2PiF0AyFX4-P68kqVu4LHMODZ-RhWLJ-OOrkTI; path=/; HttpOnly")
                        .header("set-cookie", "_redstone_server_web_user_remember_me=SFMyNTY.g2gDbQAAACAd6VBb3CeXpExUMxzSInfAWclBhjo7K413049XYe7EnG4GADoz9cyFAWIATxoA.Tz2ecG76yKjj6HQsdzHPBKp9l7KswJEJ8a61HMf6r3A; path=/; expires=Tue, 21 Mar 2023 02:14:59 GMT; max-age=5184000; HttpOnly; SameSite=Lax")
                        .header("cache-control", "max-age=0, private, must-revalidate")
                        .header("content-length", "0")
                        .header("date", "Fri, 20 Jan 2023 17:31:41 GMT")
                        .header("x-request-id", "FzwUroJxKFMJv6MAAAmB")
                        .header("server", "Cowboy");
                });
                let response = client
                    .request(Method::POST, server.url("/api/auth"))
                    .send()?;
                api_mock.assert();
                Ok(response)
            } else {
                let api_mock = server.mock(|when, then| {
                    when.method(POST).path("/api/auth");
                    then.status(401).header("content-type", "text/html");
                });

                let response = client
                    .request(Method::POST, server.url("/api/auth"))
                    .send()?;
                api_mock.assert();
                Ok(response)
            }
        }
    }

    #[test]
    fn should_throw_an_error_when_login_is_incorrect() {
        let sender = AuthMockSender { is_success: false };
        let jar = Arc::new(Jar::default());
        let client = RedstoneBlockingClient::with_sender(sender, jar);

        let auth_request = AuthRequest {
            email: "test@test.com".into(),
            password: "123123123123".into(),
        };

        let result = login(auth_request, client);

        assert!(result.is_err());
    }

    #[test]
    fn should_save_cookies() {
        let sender = AuthMockSender { is_success: true };
        let jar = Arc::new(Jar::default());
        let client = RedstoneBlockingClient::with_sender(sender, jar);

        let auth_request = AuthRequest {
            email: "test@test.com".into(),
            password: "123123123123".into(),
        };

        let result = login(auth_request, client);

        assert!(result.is_ok());

        let auth_data = get_auth_data().unwrap();
        assert!(auth_data.is_some());
        let _ = TestCleaner;
    }
}
