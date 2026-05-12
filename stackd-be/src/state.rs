use sqlx::SqlitePool;

pub struct AppState {
    pub pool: SqlitePool,
    pub jwt_secret: String,
}
