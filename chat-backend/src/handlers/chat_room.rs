use crate::models::{ChatRoom, CreateChatRoom};
use axum::{
    body::Body,
    extract::{Query, State},
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
use std::collections::HashMap;

pub async fn create_chat_room(
    State(database): State<Database>,
    Json(payload): Json<CreateChatRoom>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let chat_room_collection: Collection<Document> = database.collection("chat_rooms");

    let new_chat_room: Document = doc! {
        "owner": payload.owner,
        "name": payload.name
    };
    let insert_result = chat_room_collection.insert_one(new_chat_room, None).await;
    match insert_result {
        Ok(result) => Ok(StatusCode::CREATED),
        Err(error) => Ok(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_chat_room(
    State(database): State<Database>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let chat_room_collection: Collection<Document> = database.collection("chat_rooms");
    let chat_room_name = params.get("name");
    let user_doc = chat_room_collection
        .find_one(doc! { "name": chat_room_name }, None)
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
                "message": "Invalid Chat Room Name",
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    // Generate authentication token (e.g., using a library like `jsonwebtoken`)
    // let token = generate_auth_token(&user_doc).await?;

    // Set authentication cookie (e.g., using a library like `cookie`)
    // let mut headers = HeaderMap::new();
    // set_auth_cookie(&mut headers, token).await?;

    // Return successful login response
    let success_response = serde_json::json!({
        "status": "success",
        "message": "Correct Chat Room Name",
    });

    Ok((StatusCode::OK, Json(success_response)))
}
