use axum::{Router, routing::get};
use sqlx::SqlitePool;
use std::sync::Arc;

struct StackdState {
    db: SqlitePool,
    app_name: String,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // let addr = "0.0.0.0:3000";
    // let app = Router::new().route("/", get(|| async { "Hello, world" }));
    // let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    // println!("listening at http://{}", &addr);
    // axum::serve(listener, app).await.unwrap();
    println!("cwd: {:?}", std::env::current_dir().unwrap());
    let pool =
        SqlitePool::connect("sqlite:///home/aougri/dev/stackd/stackd-be/target/debug/test.db")
            .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL
        )
    "#,
    )
    .execute(&pool)
    .await?;

    Ok(())
}
