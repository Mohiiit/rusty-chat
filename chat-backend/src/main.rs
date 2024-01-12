mod handlers;
mod models;
use axum::{
    handler,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use handlers::{create_user, login_user};
use models::User;
use mongodb::{
    options::{ClientOptions, ResolverConfig},
    Client, Database,
};
use std::env;
use std::error::Error;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let mongodb_pool = get_database_connection()
        .await
        .expect("failed to connect to mongodb");
    let app = Router::new()
        .route("/", get(root))
        .route("/create_user", post(create_user))
        .route("/login", get(login_user))
        .with_state(mongodb_pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
    // let client_options = ClientOptions::parse(database_config.connection_string()).await?;
    // let client = Client::with_options(client_options)?;
    Ok(client.database("rusty-chat"))
}

pub async fn root() -> &'static str {
    "Hello, World! from Axum"
}
