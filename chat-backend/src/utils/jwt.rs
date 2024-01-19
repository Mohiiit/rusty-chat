use axum::{
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
    body::Body,
};
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    error::Error,
    Client, Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize)]
pub struct Cliams {
    pub exp: usize,
    pub iat: usize,
    pub name: String,
}

pub fn encode_jwt(name: String) -> Result<String, StatusCode> {
    dotenv().ok();

    let now = Utc::now();
    let expire = Duration::hours(2);

    let claim = Cliams {
        iat: now.timestamp() as usize,
        exp: (now + expire).timestamp() as usize,
        name: name,
    };
    let secret = env::var("TOKEN").expect("You must set the TOKEN environment var!");

    return encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
}

pub fn decode_jwt(jwt: String) -> Result<TokenData<Cliams>, StatusCode> {
    let secret = env::var("TOKEN").expect("You must set the TOKEN environment var!");
    let res: Result<TokenData<Cliams>, StatusCode> = decode(
        &jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR);
    return res;
}

pub async fn auth(
    State(database): State<Database>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });

    let token = token.ok_or_else(|| {
        let json_error = serde_json::json!({
            "status": "fail",
            "message": "You are not logged in, please provide token".to_string(),
        });
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?;
    let secret = env::var("TOKEN").expect("You must set the TOKEN environment var!");
    let claims = decode::<Cliams>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| {
        let json_error = serde_json::json!({
            "status": "fail",
            "message": "Invalid token".to_string(),
        });
        (StatusCode::UNAUTHORIZED, Json(json_error))
    })?
    .claims;

    // let username = to_str(&claims.name).map_err(|_| {
    //     let json_error = serde_json::json!({
    //         "status": "fail",
    //         "message": "Invalid token".to_string(),
    //     });
    //     (StatusCode::UNAUTHORIZED, Json(json_error))
    // })?;

    let username = claims.name.to_string();

    let user_collection: Collection<Document> = database.collection("users");

    let user_doc = user_collection
        .find_one(doc! { "name": &username }, None)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or_else(|| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Invalid username",
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    req.extensions_mut().insert(user_doc);
    Ok(next.run(req).await)
}
