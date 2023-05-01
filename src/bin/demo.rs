use anyhow::anyhow;
use anyhow::Context;
use oxide_client::types::UsernamePasswordCredentials;
use oxide_client::Client;
use oxide_client::ClientLoginExt;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let silo_name = "test-suite-silo";
    let username: oxide_client::types::UserId =
        "test-privileged".parse().map_err(|s| {
            anyhow!("parsing configured recovery user name: {:?}", s)
        })?;
    let password: oxide_client::types::Password = "oxide".parse().unwrap();

    let reqwest_login_client = reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(60))
        .build()?;
    let base_url = "http://127.0.0.1:12220/";
    let oxide_login_client =
        Client::new_with_client(base_url, reqwest_login_client);
    let response = oxide_login_client
        .login_local()
        .silo_name(silo_name)
        .body(UsernamePasswordCredentials {
            username: username.clone(),
            password: password.clone(),
        })
        .send()
        .await
        .context("logging in")?;
    eprintln!(
        "logged in: status code {} ({:?})",
        response.status().as_str(),
        response.status().canonical_reason()
    );
    let headers = response.headers();
    for (header_name, value) in headers.iter() {
        eprintln!(
            "header: {:?} = {:?}",
            header_name,
            value.to_str().unwrap_or("<unformattable>")
        );
    }

    let session_cookie = headers
        .get(http::header::SET_COOKIE)
        .ok_or_else(|| anyhow!("expected session cookie after login"))?
        .to_str()
        .context("expected session cookie token to be a string")?;
    let (session_token, rest) = session_cookie.split_once("; ").unwrap();
    assert!(session_token.starts_with("session="));
    assert_eq!(rest, "Path=/; HttpOnly; SameSite=Lax; Max-Age=3600");

    Ok(())
}
