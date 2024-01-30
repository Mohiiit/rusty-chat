use crate::models::{ChatRoom, GetMessage, SendMessage};
use crate::models::{Message, User};
use crate::utils::token::Ctx;
use axum::{
    body::Body,
    extract::{Query, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use bson::Array;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    error::Error,
    Client, Collection, Database,
};
use rand_core::OsRng;
use serde_json::json;
use std::collections::HashMap;

pub async fn add_message(
    State(database): State<Database>,
    ctx: Ctx,
    Json(payload): Json<SendMessage>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let chat_room_collection: Collection<Document> = database.collection("chat_rooms");
    let message_collection: Collection<Document> = database.collection("messages");
    let chat_room_doc = chat_room_collection
        .find_one(doc! { "_id": &payload.chat_room_id }, None)
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

    let new_message: Document = doc! {
        "sender": &ctx.username(),
        "message": &payload.message,
        "chat_room_id":&payload.chat_room_id
    };
    let insert_result = message_collection.insert_one(new_message, None).await;
    let success_response = serde_json::json!({
        "status": "success",
        "message": format!("Message added successfully"),
    });
    let fail_response = serde_json::json!({
        "status": "fail",
        "message": format!("Message not delivered"),
    });
    match insert_result {
        Ok(result) => Ok((StatusCode::CREATED, Json(success_response))),
        Err(error) => Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(fail_response))),
    }
}

pub async fn get_messages(
    State(database): State<Database>,
    Query(mut params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let message_collection: Collection<Message> = database.collection("messages");
    let chat_room_id: ObjectId = ObjectId::parse_str(params.get_mut("chat_room_id").unwrap())
        .unwrap_or_else(|e| {
            // Handle invalid chat room ID error
            todo!();
        });

    let mut messages = message_collection
        .find(doc! { "chat_room_id": chat_room_id }, None)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
              "status": "error",
              "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    let mut rows: Vec<Message> = Vec::new();

    while let Some(doc) = messages.next().await {
        rows.push(doc.unwrap());
    }
    let success_response = serde_json::json!({
      "status": "success",
      "message": "Correct Chat Room Name",
      "data": rows
    });

    Ok((StatusCode::OK, Json(success_response)))
}
