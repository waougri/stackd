use crate::schema::{AppError, HttpResponse, StackdMessage};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::post;
// Added get
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Location {
    pub id: String, // Changed from i64 to String to match your UUID/TEXT migration
    pub name: String,
    pub parent_id: Option<String>, // Changed to String to match ID type
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LocationUpdate {
    pub name: Option<String>,
    pub parent_id: Option<String>,
}
#[derive(Deserialize, Validate)]
pub struct CreateLocation {
    #[validate(length(min = 1))]
    pub name: String,
    pub parent_id: Option<String>,
}
impl Location {
    pub async fn add_location(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<CreateLocation>,
    ) -> HttpResponse<StackdMessage> {
        // Use v7 as discussed for better B-Tree performance
        let uuid = Uuid::now_v7().to_string();

        sqlx::query!(
            "insert into locations (id, parent_id, name) values (?, ?, ?)",
            uuid,
            payload.parent_id,
            payload.name
        )
        .execute(&state.pool)
        .await
        .map_err(AppError::SqlError)?; // Simplified map_err

        Ok((
            StatusCode::CREATED,
            Json(StackdMessage {
                message: format!("Location {} successfully created!", payload.name),
            }),
        ))
    }

    pub async fn remove_location(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
    ) -> HttpResponse<()> {
        // We pass the string directly to SQLite since it's stored as TEXT
        sqlx::query!("delete from locations where id = ?", id)
            .execute(&state.pool)
            .await
            .map_err(AppError::SqlError)?;

        Ok((StatusCode::NO_CONTENT, Json(())))
    }

    pub async fn get_all_locations(
        State(state): State<Arc<AppState>>,
    ) -> HttpResponse<Vec<Location>> {
        // Fixed return type nesting
        let locations = sqlx::query_as!(Location, "select id, name, parent_id from locations")
            .fetch_all(&state.pool) // Use fetch_all for lists, not execute
            .await
            .map_err(AppError::SqlError)?;

        Ok((StatusCode::OK, Json(locations)))
    }

    pub async fn get_location_by_id(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
    ) -> HttpResponse<Location> {
        let _uuid = Uuid::parse_str(&id).map_err(AppError::UUIDParseError)?;
        let query = sqlx::query_as!(
            Location,
            "select id, name, parent_id from locations where id
         = ?",
            id
        )
        .fetch_optional(&state.pool)
        .await
        .map_err(AppError::SqlError)?
        .ok_or_else(|| AppError::NotFound(id))?;

        Ok((StatusCode::OK, Json(query)))
    }

    pub async fn update_location(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
        Json(payload): Json<LocationUpdate>,
    ) -> HttpResponse<()> {
        let _uuid = Uuid::parse_str(&id).map_err(AppError::UUIDParseError)?;

        let mut loc = sqlx::query_as!(Location, "SELECT * FROM locations WHERE id = ?", id)
            .fetch_optional(&state.pool)
            .await
            .map_err(AppError::SqlError)?
            .ok_or_else(|| AppError::NotFound(id.clone()))?;

        // apply updates onto loc directly
        if let Some(name) = payload.name {
            loc.name = name;
        }
        if let Some(parent_id) = payload.parent_id {
            loc.parent_id = Some(parent_id);
        }

        sqlx::query!(
            "UPDATE locations SET name = ?, parent_id = ? WHERE id = ?",
            loc.name,
            loc.parent_id,
            id
        )
        .execute(&state.pool)
        .await
        .map_err(AppError::SqlError)?;

        Ok((StatusCode::OK, Json(())))
    }
    pub fn routes() -> axum::Router<Arc<AppState>> {
        axum::Router::new()
            .route("/", post(Self::add_location).get(Self::get_all_locations))
            .route(
                "/{id}",
                axum::routing::delete(Self::remove_location).get(Self::get_location_by_id).put
                (Self::update_location),
            )
    }
}
