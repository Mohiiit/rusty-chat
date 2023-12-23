use axum::{response::Html, routing::get, Router, async_trait, http::request::Parts, extract::FromRequestParts};
use mongodb::{Client, options::{ClientOptions, ResolverConfig}, Database, bson::{Bson, document::Document}};
use std::{convert::Infallible, env};
use std::error::Error;
mod model;

use self::{
    model::{Fortune, FortuneInfo, World, Task}
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
    .route("/", get(handler))
    .route("/hello", get(handler_hello))
    .with_state(database);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
axum::serve(listener, app).await.unwrap();

   Ok(())
}

async fn handler(DatabaseConnection(db): DatabaseConnection,) {
    let todo_collection = db.collection::<Task>("ToDo");
    let new_task: Task = Task {
        id: 1,
        description: "testing description".to_string(),
        status: "testing".to_string()
    };
    // let doc = Document::from(new_task);
    let insert_result = todo_collection.insert_one(new_task, None).await;
    match insert_result {
        Ok(result) => println!("New document ID: {}", result.inserted_id),
        Err(error) => {
            // Handle the error appropriately, e.g., log it or return an error response
            println!("Error inserting task: {}", error);
        }
    }
}
// GAGraa2I586j9bDr
async fn handler_hello() -> Html<&'static str> {
    Html("<h1>Hello, World Ji!</h1>")
}