use crate::state::AppState;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
struct StackdMessage {
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UpdateItem {
    name: String,
}
type Response<T> = (StatusCode, Json<T>);
type SuccessResponse<T> = Response<T>;
type ErrorResponse = Response<StackdMessage>;
type HttpResponse<T> = Result<SuccessResponse<T>, ErrorResponse>;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Item {
    pub id: String,
    pub name: String,
}

impl Item {
    async fn get_all(State(state): State<Arc<AppState>>) -> HttpResponse<Vec<Item>> {
        let res = sqlx::query_as!(Item, "SELECT id, name FROM items")
            .fetch_all(&state.pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(StackdMessage {
                        message: format!("No items found. {}", err),
                    }),
                )
            })?;
        Ok((StatusCode::OK, Json(res)))
    }

    async fn get_by_id(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
    ) -> HttpResponse<Self> {
        let res = sqlx::query_as!(Item, "SELECT * FROM items WHERE id = $1", id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(StackdMessage {
                        message: format!("{}", err),
                    }),
                )
            })?;
        match res {
            None => Err((
                StatusCode::NOT_FOUND,
                Json(StackdMessage {
                    message: format!(
                        "No item \
            found for id: {}",
                        id
                    ),
                }),
            )),
            Some(item) => Ok((StatusCode::OK, Json(item))),
        }
    }

    async fn create(
        State(state): State<Arc<AppState>>,
        payload: Json<Item>,
    ) -> HttpResponse<String> {
        let res = sqlx::query!(
            "INSERT INTO items (id, name) VALUES ($1, $2)",
            payload.id,
            payload.name
        )
        .execute(&state.pool)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(StackdMessage {
                    message: format!("Cannot insert item: {}", err),
                }),
            )
        })?;

        Ok((
            StatusCode::CREATED,
            Json(format!("item {} created", &payload.name)),
        ))
    }
    async fn delete_all(State(state): State<Arc<AppState>>) -> HttpResponse<StackdMessage> {
        let res = sqlx::query!("DELETE FROM items")
            .execute(&state.pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(StackdMessage {
                        message: format!(
                            "Cannot \
            delete items: {}",
                            err
                        ),
                    }),
                )
            });

        Ok((
            StatusCode::OK,
            Json(StackdMessage {
                message: "all items have been deleted.".to_string(),
            }),
        ))
    }

    async fn delete_item(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
    ) -> HttpResponse<StackdMessage> {
        let res = sqlx::query!("DELETE FROM items WHERE id = $1", id)
            .execute(&state.pool)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    StackdMessage {
                        message: format!("Cannot delete item: {}", err),
                    },
                )
            });

        Ok((
            StatusCode::OK,
            Json(StackdMessage {
                message: format!("item {} deleted", &id),
            }),
        ))
    }

    async fn update_item(
        State(state): State<Arc<AppState>>,
        Path(id): Path<String>,
        Json(payload): Json<UpdateItem>,
    ) -> HttpResponse<StackdMessage> {
        let res = sqlx::query!(
            "UPDATE  items set name = $1 WHERE id = $2",
            payload.name,
            id
        )
        .execute(&state.pool)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                StackdMessage {
                    message: format!("Cannot update item: {}", err),
                },
            )
        });

        Ok((
            StatusCode::OK,
            Json(StackdMessage {
                message: "item updated".to_string(),
            }),
        ))
    }

    pub fn routes() -> axum::Router<Arc<AppState>> {
        axum::Router::new()
            .route("/", post(Item::create))
            .route("/", get(Item::get_all))
            .route("/{id}", get(Item::get_by_id))
            .route("/{id}", patch(Item::update_item))
            .route("/{id}", delete(Item::delete_item))
            .route("/", delete(Item::delete_all))
    }
}
