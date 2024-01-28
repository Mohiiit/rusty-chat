mod handlers;
mod models;
mod utils;
use axum::{
    handler, middleware,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use handlers::chat_room::{create_chat_room, get_chat_room};
use handlers::user::{create_user, login_user};
use models::User;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client, Database,
};
use std::env;
use std::error::Error;
use tokio;
use utils::jwt::auth;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let mongodb_pool = get_database_connection()
        .await
        .expect("failed to connect to mongodb");
    let app = Router::new()
        .route("/", get(root))
        .route("/create_chat_room", post(create_chat_room))
        .route("/get_chat_room", get(get_chat_room))
        .layer(middleware::from_fn_with_state(mongodb_pool.clone(), auth))
        .route("/create_user", post(create_user))
        .route("/login", get(login_user))
        .with_state(mongodb_pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

pub async fn get_database_connection() -> Result<Database, mongodb::error::Error> {
    let client_uri =
        env::var("MONGODB_URI").expect("You must set the MONGODB_URI environment var!");
    let options =
        ClientOptions::parse_with_resolver_config(&client_uri, ResolverConfig::cloudflare())
            .await?;
    let client = Client::with_options(options)?;
    Ok(client.database("rusty-chat"))
}

pub async fn root() -> &'static str {
    "Hello, World! from Axum"
}
