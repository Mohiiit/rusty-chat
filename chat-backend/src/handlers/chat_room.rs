use crate::models::User;
use crate::models::{ChatRoom, CreateChatRoom};
use crate::utils::token::Ctx;
use axum::{
    body::Body,
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
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
    ctx: Ctx,
    Json(payload): Json<CreateChatRoom>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let chat_room_collection: Collection<Document> = database.collection("chat_rooms");
    println!("username here: {:?}", ctx.username());
    let new_chat_room: Document = doc! {
        "owner": &ctx.username(),
        "name": &payload.name
    };
    let insert_result = chat_room_collection.insert_one(new_chat_room, None).await;
    let success_response = serde_json::json!({
        "status": "success",
        "message": format!("Chatroom with name: {} added with ownership of: {} created", payload.name, ctx.username()),
    });
    let fail_response = serde_json::json!({
        "status": "fail",
        "message": format!("Chatroom with name: {} added with ownership of: {} failed", payload.name, ctx.username()),
    });
    match insert_result {
        Ok(result) => Ok((StatusCode::CREATED, Json(success_response))),
        Err(error) => Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(fail_response))),
    }
}

pub async fn get_chat_room(
    State(database): State<Database>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let chat_room_collection: Collection<Document> = database.collection("chat_rooms");
    let chat_room_name = params.get("name");
    let chat_room_doc = chat_room_collection
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
        "data": chat_room_doc
    });

    Ok((StatusCode::OK, Json(success_response)))
}
