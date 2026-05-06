mod routes;
mod schema;
mod state;
use crate::routes::item::Item;
use crate::state::AppState;
use axum::routing::post;
use axum::{Router, routing::get};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::str::FromStr;
use std::time::Duration;
use std::{
    fs::{self, File},
    sync::Arc,
};

async fn setup_db(state: Arc<AppState>) -> Result<(), sqlx::Error> {

    tracing::info!("Setting up database");
    tracing::info!("Creating items DB");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS items(
        id  TEXT PRIMARY KEY,
        name TEXT NOT NULL UNIQUE
        )
    "#,
    )
    .execute(&state.pool)
    .await?;

    tracing::info!("Creating location DB");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            parent_id INTEGER
        )
        "#
    )
    .execute(&state.pool)
    .await?;

    tracing::info!("Creating inventory DB");
    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS inventory (
            id INTEGER PRIMARY KEY,
            timestamp integer NOT NULL,
            location_id INTEGER NOT NULL REFERENCES locations(id),
            item_id TEXT NOT NULL REFERENCES items(id),
            action_type TEXT NOT NULL,
            value INTEGER NOT NULL

        )
        "#
    )
    .execute(&state.pool)
    .await?;

    Ok(())
}

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let key = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    println!("cwd: {:?}", std::env::current_dir().unwrap());

    let db_file = format!(
        "{}/test.db",
        std::env::current_dir()
            .unwrap()
            .into_os_string()
            .to_str()
            .unwrap()
    );
    print!("{}", db_file);
    if !fs::exists(&db_file).unwrap() {
        tracing::info!("Database file not found. Creating '{}'...", &db_file);
        File::create_new(&db_file)?;
    }
    let db_url = format!("sqlite://{}", db_file);
    let conn_opts = SqliteConnectOptions::from_str(&db_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    let pool = Arc::new(AppState {
        pool: SqlitePool::connect_with(conn_opts).await?,
    });


    sqlx::migrate!("./migrations")
        .run(&pool.pool)
        .await?;
    
    let addr = "0.0.0.0:3000";
    let app = Router::new()
        .route("/", get(|| async { "Hello, world" }))
        .route("/add-location", post(schema::Location::add_location))
         .nest("/items", Item::routes())
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("listening at http://{}", &addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

