use crate::models::{CreateUserRequest, LoginUserRequest, User};
use crate::utils::auth::{hash_password, verify_password_internal};
use crate::utils::jwt::encode_jwt;
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
    let existing_user = user_collection
        .find_one(
            doc! {
                "$or": [
                    { "name": &payload.name },
                    { "email": &payload.email },
                ]
            },
            None,
        )
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        });

    if existing_user?.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(
                json!({ "status": "fail", "message": "User with same name or email already exists" }),
            ),
        ));
    }
    let hashed_password = hash_password(&payload.password).await?;
    let new_user: Document = doc! {
        "name": payload.name,
        "email": payload.email,
        "password": hashed_password
    };
    let insert_result = user_collection.insert_one(new_user, None).await;
    match insert_result {
        Ok(result) => Ok(StatusCode::CREATED),
        Err(error) => Ok(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn login_user(
    State(database): State<Database>,
    Json(payload): Json<LoginUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_collection: Collection<Document> = database.collection("users");

    let user_doc = user_collection
        .find_one(doc! { "name": &payload.name }, None)
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
    let token = encode_jwt(payload.name);
    // Generate authentication token (e.g., using a library like `jsonwebtoken`)
    // let token = generate_auth_token(&user_doc).await?;

    // Set authentication cookie (e.g., using a library like `cookie`)
    // let mut headers = HeaderMap::new();
    // set_auth_cookie(&mut headers, token).await?;

    // Return successful login response
    let success_response = serde_json::json!({
        "status": "success",
        "token": token.unwrap(),
    });

    Ok((StatusCode::OK, Json(success_response)))
}
