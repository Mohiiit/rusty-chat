use axum::{
    body::Body,
    extract::{Extension, Json, State, FromRequest, FromRequestParts},
    http::{header, Request, StatusCode, request::Parts},
    middleware::Next,
    response::IntoResponse, Error, async_trait
};
use chrono::{Duration, Utc};
use dotenv::dotenv;
use std::env;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use mongodb::{bson::{doc, Document}, Collection, Database};
use serde::{Deserialize, Serialize};
use crate::utils::token::Ctx;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
    pub name: String,
}

pub fn encode_jwt(name: String) -> Result<String, StatusCode> {
    dotenv().ok();

    let now = Utc::now();
    let expire = Duration::hours(1);

    let claim = Claims {
        iat: now.timestamp() as usize,
        exp: (now + expire).timestamp() as usize,
        name,
    };
    let secret = env::var("TOKEN").expect("You must set the TOKEN environment var!");

    encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn decode_jwt(jwt: String) -> Result<TokenData<Claims>, (StatusCode, Json<serde_json::Value>)> {
    let secret = env::var("TOKEN").expect("You must set the TOKEN environment var!");

    Ok(decode(
        &jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("Token error: {}", e),
        });
        (StatusCode::UNAUTHORIZED, Json(error_response))
    })?)
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
    let claims = decode_jwt(token)?;

    let username = claims.claims.name.to_string();

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
    
    // let username = user_doc.get_str("name").unwrap();

    req.extensions_mut().insert(Ctx::new(username.clone()));
    println!("this is it: {:?}", username.clone());
    Ok(next.run(req).await)
}


#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");

        let ctx = parts
            .extensions
            .get::<Ctx>()
            .ok_or_else(|| {
                // Return a suitable error response
                (StatusCode::BAD_REQUEST, Json(serde_json::json!({
                    "status": "fail",
                    "message": "Invalid username"
                })))
            })?;

        Ok(ctx.clone()) // Clone the extracted Ctx
    }
}
