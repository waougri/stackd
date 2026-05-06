use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use sqlx::SqlitePool;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Location {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
}

impl Location {
    pub async fn add_location(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<Location>,
    ) -> Result<StatusCode, (StatusCode, String)> {
        let query = sqlx::query!(
            "insert into locations (parent_id, name) values  (?, ?)",
            payload.parent_id,
            payload.name
        )
        .execute(&state.pool)
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()));
        Ok(StatusCode::CREATED)
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Inventory {
    pub id: Option<i64>,
    pub item_id: String,
    pub location_id: i64,
    pub quantity: i64,
}

impl Inventory {
    pub async fn add_item_to_location(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<Inventory>,
    ) -> Result<StatusCode, (StatusCode, String)> {
        Ok(StatusCode::CREATED)
    }
}
