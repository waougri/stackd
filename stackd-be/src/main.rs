mod routes;
mod schema;
mod state;
pub mod auth;

use crate::routes::item::Item;
use crate::routes::location::Location;
use crate::state::AppState;
use axum::{Router, routing::get};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::str::FromStr;
use std::time::Duration;
use std::{
    fs::{self},
    sync::Arc,
};
use crate::routes::inventory::{Inventory, InventoryHandler};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let _key = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let img_dir = format!(
        "{}/stackd/images/",
        std::env::var("HOME").expect("HOME not set")
    );

    println!("cwd: {:?}", std::env::current_dir().unwrap());

    if !fs::exists(&img_dir).unwrap() {
        tracing::info!("stackd image file not found. Creating '{}'...", &img_dir);
        tokio::fs::create_dir_all(&img_dir).await?;
    }

    tracing::info!("Setting up the DB connection");
    let db_url = format!("sqlite://{}", _key);
    let conn_opts = SqliteConnectOptions::from_str(&db_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = Arc::new(AppState {
        pool: SqlitePool::connect_with(conn_opts).await?,
    });

    tracing::info!("Running database migrations");
    sqlx::migrate!("./migrations").run(&pool.pool).await?;

    tracing::info!("Setting up the axum::Router and server.");
    let addr = "0.0.0.0:3000";
    // In your main.rs setup
    let cors = tower_http::cors::CorsLayer::permissive();
    let app = Router::new()
        .route("/", get(|| async { "Hello, world" }))
        .nest("/locations", Location::routes())
        .nest("/inventory", InventoryHandler::routes())
        .nest("/items", Item::routes())
        .layer(cors)
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening at http://{}", &addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
