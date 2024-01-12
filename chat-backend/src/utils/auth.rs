use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use rand_core::OsRng;
use serde_json::json;

pub async fn hash_password(
    password: &str,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let mut rng = OsRng;
    let salt = SaltString::generate(&mut rng);

    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = json!({
                "status": "fail",
                "message": format!("Error while hashing password: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())
}

pub async fn verify_password_internal(
    saved_password: &str,
    entered_password: &str,
) -> Result<bool, (StatusCode, Json<serde_json::Value>)> {
    let is_valid = match PasswordHash::new(&saved_password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(entered_password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Invalid email or password"
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
    Ok(true)
}
