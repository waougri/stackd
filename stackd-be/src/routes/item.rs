use crate::schema::{AppError, HttpResponse, StackdMessage};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
struct UpdateItem {
    #[validate(length(min = 1, max = 4096))]
    name: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub image_path: Option<String>,
}

impl Item {
    #[tracing::instrument(skip(state))]
    async fn get_all(State(state): State<Arc<AppState>>) -> HttpResponse<Vec<Item>> {
        tracing::debug!("fetching all items");
        let res = sqlx::query_as!(Item, "SELECT id, name, image_path FROM items")
            .fetch_all(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to fetch all items");
                AppError::SqlError(e)
            })?;

        tracing::info!(count = res.len(), "retrieved items");
        Ok((StatusCode::OK, Json(res)))
    }

    #[tracing::instrument(skip(state))]
    async fn get_by_id(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
    ) -> HttpResponse<Self> {
        tracing::debug!(item_id = %id, "fetching item by id");
        let res = sqlx::query_as!(Item, "SELECT id, name, image_path FROM items WHERE id = ?", id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, item_id = %id, "db error fetching item");
                AppError::SqlError(e)
            })?
            .ok_or_else(|| {
                tracing::warn!(item_id = %id, "item not found");
                AppError::NotFound(id.clone())
            })?;

        Ok((StatusCode::OK, Json(res)))
    }

    #[tracing::instrument(skip(state, payload), fields(item_id = %payload.id, item_name = %payload.name))]
    async fn create(
        State(state): State<Arc<AppState>>,
        payload: Json<Item>,
    ) -> HttpResponse<StackdMessage> {
        tracing::info!("creating new item");
        sqlx::query!(
            "INSERT INTO items (id, name) VALUES (?, ?)",
            payload.id,
            payload.name
        )
            .execute(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to insert item");
                AppError::SqlError(e)
            })?;

        tracing::info!("item created successfully");
        Ok((
            StatusCode::CREATED,
            Json(StackdMessage{message: format!("item {} created", &payload.name)}),
        ))
    }

    #[tracing::instrument(skip(state))]
    async fn delete_all(State(state): State<Arc<AppState>>) -> HttpResponse<StackdMessage> {
        tracing::warn!("deleting all items from database");
        sqlx::query!("DELETE FROM items")
            .execute(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "failed to delete all items");
                AppError::SqlError(e)
            })?;

        tracing::info!("purge complete");
        Ok((
            StatusCode::OK,
            Json(StackdMessage {
                message: "all items have been deleted.".to_string(),
            }),
        ))
    }

    #[tracing::instrument(skip(state))]
    async fn delete_item(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
    ) -> HttpResponse<StackdMessage> {
        tracing::debug!(item_id = %id, "attempting to delete item");
        let res = sqlx::query!("DELETE FROM items WHERE id = ?", id)
            .execute(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, item_id = %id, "failed to delete item");
                AppError::SqlError(e)
            })?;

        if res.rows_affected() == 0 {
            tracing::warn!(item_id = %id, "delete failed: item not found");
            return Err(AppError::NotFound(id));
        }

        tracing::info!(item_id = %id, "item deleted");
        Ok((
            StatusCode::OK,
            Json(StackdMessage {
                message: format!("item {} deleted", id),
            }),
        ))
    }

    #[tracing::instrument(skip(state, payload))]
    async fn update_item(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
        Json(payload): Json<UpdateItem>,
    ) -> HttpResponse<StackdMessage> {
        tracing::debug!(item_id = %id, "validating update request");
        payload
            .validate()
            .map_err(|e| {
                tracing::warn!(item_id = %id, error = %e, "validation failed");
                AppError::BadRequest("Invalid input".into())
            })?;

        let res = sqlx::query!("UPDATE items set name = ? WHERE id = ?", payload.name, id)
            .execute(&state.pool)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, item_id = %id, "failed to update item");
                AppError::SqlError(e)
            })?;

        if res.rows_affected() == 0 {
            tracing::warn!(item_id = %id, "update failed: item not found");
            return Err(AppError::NotFound(id));
        }

        tracing::info!(item_id = %id, "item name updated");
        Ok((
            StatusCode::OK,
            Json(StackdMessage {
                message: "item updated".to_string(),
            }),
        ))
    }

    #[tracing::instrument(skip(state, multipart))]
    async fn upload_image(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
        mut multipart: Multipart,
    ) -> HttpResponse<StackdMessage> {
        tracing::info!(item_id = %id, "starting image upload");

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "multipart stream error");
                AppError::BadRequest("Invalid input".into())
            })?
        {
            if field.name() != Some("image") {
                tracing::debug!(field = ?field.name(), "skipping non-image field");
                continue;
            }

            tracing::debug!("reading image bytes");
            let bytes = field
                .bytes()
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "failed to read image bytes");
                    AppError::BadRequest("Invalid input".into())
                })?;

            tracing::debug!(byte_count = bytes.len(), "loading image from memory");
            let img = image::load_from_memory(&bytes)
                .map_err(|e| {
                    tracing::error!(error = %e, "image load failure");
                    AppError::BadRequest("Invalid input".into())
                })?;

            tracing::debug!("resizing image to 400x400");
            let resized = img.resize(400, 400, image::imageops::FilterType::Lanczos3);

            let img_dir = format!(
                "{}/stackd/images/",
                std::env::var("HOME").expect("HOME not set")
            );
            let path = format!("{}{}.webp", img_dir, id);

            tracing::debug!(save_path = %path, "saving image to disk");
            resized
                .save_with_format(&path, image::ImageFormat::WebP)
                .map_err(|e| {
                    tracing::error!(error = %e, path = %path, "disk write failure");
                    AppError::BadRequest("Image resize error".into())
                })?;

            tracing::debug!("updating image path in database");
            sqlx::query!("UPDATE items set image_path = ? WHERE id = ?", path, id)
                .execute(&state.pool)
                .await
                .map_err(|e| {
                    tracing::error!(error = %e, "failed to save image path to db");
                    AppError::SqlError(e)
                })?;

            tracing::info!(item_id = %id, "image upload complete");
            return Ok((StatusCode::OK, Json(StackdMessage {
                message: "image uploaded".to_string(),
            })))
        }

        tracing::warn!("multipart upload contained no image field");
        Ok((
            StatusCode::BAD_REQUEST,
            Json(StackdMessage {
                message: "No image field".into(),
            }),
        ))
    }

    pub fn routes() -> axum::Router<Arc<AppState>> {
        axum::Router::new()
            .route(
                "/",
                post(Item::create)
                    .get(Item::get_all)
                    .delete(Item::delete_all),
            )
            .route(
                "/{id}",
                get(Item::get_by_id)
                    .patch(Item::update_item)
                    .delete(Item::delete_item),
            ).route("/{id}/image", post(Item::upload_image))
    }
}