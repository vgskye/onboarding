use std::env::var;

use axum::{extract::State, response::ErrorResponse, routing::post, Form, Router};
use eyre::Result;
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/signup", post(submit))
        .route("/signup-pridecraft", post(submit_pridecraft))
        .with_state(Client::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:80").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize)]
struct BoardingRequest {
    username: String,
    invite_code: String,
}

async fn submit(
    State(client): State<Client>,
    Form(BoardingRequest {
        username,
        invite_code,
    }): Form<BoardingRequest>,
) -> Result<String, ErrorResponse> {
    if invite_code != var("INVITE_CODE").unwrap() {
        return Err("Bad invite code!".into());
    }
    let password = Alphanumeric.sample_string(&mut thread_rng(), 48);
    board(&client, &username, &password)
        .await
        .map_err(|e| format!("An error occurred: {e:?}"))?;
    Ok(format!("Your temporary password is: {password}"))
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

async fn board(client: &Client, username: &str, password: &str) -> Result<()> {
    let TokenResponse { access_token } = client
        .post(format!(
            "{}/realms/{}/protocol/openid-connect/token",
            var("KEYCLOAK_BASE_URL")?,
            var("KEYCLOAK_REALM")?
        ))
        .basic_auth(
            var("KEYCLOAK_CLIENT_ID")?,
            Some(var("KEYCLOAK_CLIENT_SECRET")?),
        )
        .form(&json!({
            "grant_type": "client_credentials"
        }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    client
        .post(format!(
            "{}/admin/realms/{}/users",
            var("KEYCLOAK_BASE_URL")?,
            var("KEYCLOAK_REALM")?
        ))
        .bearer_auth(access_token)
        .json(&json!({
            "username": username,
            "email": format!("{}@{}", username, var("EMAIL_DOMAIN")?),
            "enabled": true,
            "emailVerified": true,
            "credentials": [
                {
                    "type": "password",
                    "temporary": true,
                    "value": password
                }
            ]
        }))
        .send()
        .await?
        .error_for_status()?;
    client
        .post(format!("{}/api/v1/add/mailbox", var("MAILCOW_BASE_URL")?,))
        .header("X-API-Key", var("MAILCOW_TOKEN")?)
        .json(&json!({
            "active": 1,
            "domain": var("EMAIL_DOMAIN")?,
            "local_part": username,
            "authsource": "keycloak"
        }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}


async fn submit_pridecraft(
    State(client): State<Client>,
    Form(BoardingRequest {
        username,
        invite_code,
    }): Form<BoardingRequest>,
) -> Result<String, ErrorResponse> {
    if invite_code != var("PRIDECRAFT_INVITE_CODE").unwrap() {
        return Err("Bad invite code!".into());
    }
    let password = Alphanumeric.sample_string(&mut thread_rng(), 48);
    board_pridecraft(&client, &username, &password)
        .await
        .map_err(|e| format!("An error occurred: {e:?}"))?;
    Ok(format!("Your temporary password is: {password}"))
}

async fn board_pridecraft(client: &Client, username: &str, password: &str) -> Result<()> {
    client
        .post(format!("{}/api/v1/add/mailbox", var("MAILCOW_BASE_URL")?,))
        .header("X-API-Key", var("MAILCOW_TOKEN")?)
        .json(&json!({
            "active": 1,
            "domain": var("PRIDECRAFT_EMAIL_DOMAIN")?,
            "local_part": username,
            "password": password,
            "password2": password,
        }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}