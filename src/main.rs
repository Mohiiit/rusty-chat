use axum::{routing::{get, post}, Router, async_trait, http::request::Parts, http::StatusCode,  extract::{FromRequestParts, Json, Path, Query}, response::IntoResponse};
use mongodb::{Client, options::{ClientOptions, ResolverConfig}, Database};
use std::{convert::Infallible, env, collections::HashMap};
use std::error::Error;
use bson::doc;
use futures_util::stream::StreamExt;
use log::{error, LevelFilter};
use serde_json::to_string;

mod model;

use self::{
    model::{Task, TaskList}
};

pub struct DatabaseConnection(pub Database);

#[async_trait]
impl FromRequestParts<Database> for DatabaseConnection {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        db: &Database,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(db.clone()))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    // build our application with a route

    // // run it
    // let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
    //     .await
    //     .unwrap();
    // println!("listening on {}", listener.local_addr().unwrap());
    let client_uri =
      env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
    let options =
      ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
         .await?;
   let client = Client::with_options(options)?;
   let database = client.database("ToDoDemo");
   let app = Router::new()
    .route("/add_task", post(add_task))
    .route("/get_task", get(get_task))
    .with_state(database);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
axum::serve(listener, app).await.unwrap();

   Ok(())
}

async fn add_task(DatabaseConnection(db): DatabaseConnection, Json(body): Json<Task>) {
    let todo_collection = db.collection::<Task>("ToDo");
    
    // let doc = Document::from(new_task);
    let insert_result = todo_collection.insert_one(body, None).await;
    match insert_result {
        Ok(result) => println!("New document ID: {}", result.inserted_id),
        Err(error) => {
            // Handle the error appropriately, e.g., log it or return an error response
            println!("Error inserting task: {}", error);
        }
    }
}
// GAGraa2I586j9bDr
async fn get_task(
    DatabaseConnection(db): DatabaseConnection,
    Query(query): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let todo_collection = db.collection::<Task>("ToDo");

    // Extract and parse task_id, handling potential errors
    let task_id_str = match query.get("id") {
        Some(id) => id,
        None => return (StatusCode::BAD_REQUEST, Json("Missing task_id query parameter".to_string())),
    };

    let task_id = match task_id_str.parse::<i32>() {
        Ok(id) => id,
        Err(_) => return (StatusCode::BAD_REQUEST, Json("Invalid task_id format".to_string())),
    };

    // Find tasks with the specified ID, handling database errors
    let filter = doc! { "id": task_id };
    let mut cursor = match todo_collection.find(filter, None).await {
        Ok(cursor) => cursor,
        Err(error) => {
            error!("Failed to query MongoDB: {}", error);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json("Failed to retrieve tasks".to_string()));
        }
    };

    // Collect matching tasks, handling potential errors from the cursor
    let mut tasks = Vec::new();
    while let Some(task) = cursor.next().await {
        tasks.push(match task {
            Ok(task) => task,
            Err(error) => {
                error!("Error reading task from cursor: {}", error);
                break; // Stop iterating if an error occurs
            }
        });
    }

     
    (StatusCode::OK, Json(to_string(&tasks).unwrap()))
}