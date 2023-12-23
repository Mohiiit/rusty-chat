use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Fortune {
    pub id: i32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct FortuneInfo {
    pub id: i32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct World {
    pub id: i32,
    #[serde(rename = "randomNumber")]
    pub random_number: i32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Task {
    pub id: i32,
    pub description: String,
    pub status: String
}

#[derive(Serialize, Debug, Deserialize)]
pub struct TaskList {
    pub tasks: Vec<Task>,
}
