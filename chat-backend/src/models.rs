use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub _id: ObjectId,
    pub name: String,
    pub email: String,
    password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRoom {
    pub _id: ObjectId,
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateChatRoom {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginUserResponseModel {
    pub status: String,
    pub token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub _id: ObjectId,
    pub chat_room_id: ObjectId,
    pub message: String,
    pub sender: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendMessage {
    pub chat_room_id: ObjectId,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetMessage {
    pub message: String,
}
