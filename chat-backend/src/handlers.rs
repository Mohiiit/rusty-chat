use crate::models::{CreateUserRequest, LoginUserRequest, User};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    error::Error,
    Client, Collection, Database,
};
use rand_core::OsRng;
use serde_json::json;

pub async fn create_user(
    State(database): State<Database>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_collection: Collection<Document> = database.collection("users");
    let hashed_password = hash_password(&payload.password).await?;
    let new_user: Document = doc! {
        "name": payload.name,
        "email": payload.email,
        "password": hashed_password
    };
    let insert_result = user_collection.insert_one(new_user, None).await;
    match insert_result {
        Ok(result) => {
            println!("New document ID: {}", result.inserted_id);
            Ok(StatusCode::CREATED)
        }
        Err(error) => {
            // Handle the error appropriately, e.g., log it or return an error response
            println!("Error inserting user: {}", error);
            Ok(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn login_user(
    State(database): State<Database>,
    Json(payload): Json<LoginUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_collection: Collection<Document> = database.collection("users");

    let user_doc = user_collection
        .find_one(doc! { "name": payload.name }, None)
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
                "message": "Invalid email or password",
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    let saved_password = user_doc.get_str("password").unwrap();
    let verification_result = verify_password_internal(saved_password, &payload.password).await?;

    // Generate authentication token (e.g., using a library like `jsonwebtoken`)
    // let token = generate_auth_token(&user_doc).await?;

    // Set authentication cookie (e.g., using a library like `cookie`)
    // let mut headers = HeaderMap::new();
    // set_auth_cookie(&mut headers, token).await?;

    // Return successful login response
    let success_response = serde_json::json!({
        "status": "success",
        "message": "Correct Password",
    });

    Ok((StatusCode::OK, Json(success_response)))
}

async fn hash_password(password: &str) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
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

async fn verify_password_internal(
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
