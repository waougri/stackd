use crate::schema::{AppError, HttpResponse, StackdMessage};
use crate::state::AppState;
use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::Arc;
use uuid::Uuid;
use validator::{Validate, ValidationError};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(rename_all = "UPPERCASE")]
pub enum ActionType {
    Add,
    Remove,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Inventory {
    pub id: String,
    pub item_id: String,
    pub location_id: String,
    pub value: i64,
    pub action_type: ActionType,
    pub timestamp: Option<String>,
    pub move_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Request payloads
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateLog {
    #[validate(length(min = 1), custom(function = "validate_uuid"))]
    pub item_id: String,

    #[validate(length(min = 1), custom(function = "validate_uuid"))]
    pub location_id: String,

    #[validate(range(min = 1, max = 1_000_000))]
    pub value: i64,

    pub action_type: ActionType,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MoveItem {
    #[validate(length(min = 1), custom(function = "validate_uuid"))]
    pub item_id: String,

    #[validate(length(min = 1), custom(function = "validate_uuid"))]
    pub location_from_id: String,

    #[validate(length(min = 1), custom(function = "validate_uuid"))]
    pub location_to_id: String,

    #[validate(range(min = 1, max = 1_000_000))]
    pub value: i64,
}

#[derive(Debug, Deserialize)]
pub struct StockQuery {
    pub item_id: String,
    pub location_id: String,
}

#[derive(Debug, Serialize)]
pub struct StockResponse {
    pub item_id: String,
    pub location_id: String,
    pub quantity: i64,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate_uuid(id: &str) -> Result<(), ValidationError> {
    Uuid::parse_str(id)
        .map(|_| ())
        .map_err(|_| ValidationError::new("invalid_uuid"))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub struct InventoryHandler;

impl InventoryHandler {
    /// POST /inventory
    #[tracing::instrument(skip(state, payload), fields(item_id = %payload.item_id))]
    pub async fn log_event(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<CreateLog>,
    ) -> HttpResponse<StackdMessage> {
        tracing::debug!("Validating inventory log payload");
        payload.validate().map_err(|e| {
            tracing::warn!("Validation failed for log_event: {}", e);
            AppError::BadRequest(format!("validation failed: {}", e))
        })?;

        let id = Uuid::now_v7().to_string();
        tracing::info!(
            action = ?payload.action_type,
            qty = payload.value,
            loc = %payload.location_id,
            "Logging inventory event"
        );

        sqlx::query!(
            "INSERT INTO inventory (id, item_id, location_id, value, action_type)
             VALUES (?, ?, ?, ?, ?)",
            id,
            payload.item_id,
            payload.location_id,
            payload.value,
            payload.action_type,
        )
        .execute(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Database error while inserting inventory log");
            AppError::SqlError(e)
        })?;

        tracing::info!(event_id = %id, "Inventory event successfully committed");

        Ok((
            StatusCode::CREATED,
            Json(StackdMessage {
                message: format!("event logged for item {}", payload.item_id),
            }),
        ))
    }

    /// POST /inventory/move
    #[tracing::instrument(skip(state, payload), fields(item_id = %payload.item_id))]
    pub async fn move_item(
        State(state): State<Arc<AppState>>,
        Json(payload): Json<MoveItem>,
    ) -> HttpResponse<StackdMessage> {
        payload.validate().map_err(|e| {
            tracing::warn!("Validation failed for move_item: {}", e);
            AppError::BadRequest(format!("validation failed: {}", e))
        })?;

        let move_id = Uuid::now_v7().to_string();
        let remove_id = Uuid::now_v7().to_string();
        let add_id = Uuid::now_v7().to_string();

        tracing::info!(
            move_id = %move_id,
            from = %payload.location_from_id,
            to = %payload.location_to_id,
            qty = payload.value,
            "Initiating inventory move transaction"
        );

        let mut tx = state.pool.begin().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to begin transaction for move");
            AppError::SqlError(e)
        })?;

        // Step 1: REMOVE
        tracing::debug!(remove_id = %remove_id, "Executing REMOVE leg of move");
        sqlx::query!(
            "INSERT INTO inventory (id, item_id, location_id, action_type, value, move_id)
             VALUES (?, ?, ?, 'REMOVE', ?, ?)",
            remove_id,
            payload.item_id,
            payload.location_from_id,
            payload.value,
            move_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Move failed during REMOVE step");
            AppError::SqlError(e)
        })?;

        // Step 2: ADD
        tracing::debug!(add_id = %add_id, "Executing ADD leg of move");
        sqlx::query!(
            "INSERT INTO inventory (id, item_id, location_id, action_type, value, move_id)
             VALUES (?, ?, ?, 'ADD', ?, ?)",
            add_id,
            payload.item_id,
            payload.location_to_id,
            payload.value,
            move_id,
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Move failed during ADD step");
            AppError::SqlError(e)
        })?;

        tx.commit().await.map_err(|e| {
            tracing::error!(error = %e, "Failed to commit move transaction");
            AppError::SqlError(e)
        })?;

        tracing::info!(move_id = %move_id, "Move transaction completed successfully");

        Ok((
            StatusCode::CREATED,
            Json(StackdMessage {
                message: format!("moved {} units (move_id: {})", payload.value, move_id),
            }),
        ))
    }

    /// GET /inventory/stock
    #[tracing::instrument(skip(state))]
    pub async fn get_stock(
        State(state): State<Arc<AppState>>,
        Query(params): Query<StockQuery>,
    ) -> HttpResponse<StockResponse> {
        tracing::debug!(
            item = %params.item_id,
            loc = %params.location_id,
            "Calculating current stock"
        );

        let row = sqlx::query!(
            "SELECT COALESCE(
                SUM(CASE WHEN action_type = 'ADD' THEN value ELSE -value END),
                0
             ) as quantity
             FROM inventory
             WHERE item_id = ? AND location_id = ?",
            params.item_id,
            params.location_id,
        )
        .fetch_one(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to query stock level");
            AppError::SqlError(e)
        })?;

        tracing::info!(
            item = %params.item_id,
            loc = %params.location_id,
            qty = row.quantity,
            "Stock level retrieved"
        );

        Ok((
            StatusCode::OK,
            Json(StockResponse {
                item_id: params.item_id,
                location_id: params.location_id,
                quantity: row.quantity,
            }),
        ))
    }

    /// GET /inventory
    #[tracing::instrument(skip(state))]
    pub async fn get_all(State(state): State<Arc<AppState>>) -> HttpResponse<Vec<Inventory>> {
        tracing::debug!("Fetching full inventory event log");

        let events = sqlx::query_as!(
            Inventory,
            "SELECT id, item_id, location_id, value,
        action_type as 'action_type: ActionType',
        CAST(timestamp AS TEXT) as 'timestamp: String',
        move_id as 'move_id: String'
 FROM inventory
 ORDER BY timestamp DESC"
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to fetch event log");
            AppError::SqlError(e)
        })?;

        tracing::info!(count = events.len(), "Retrieved inventory events");

        Ok((StatusCode::OK, Json(events)))
    }

    /// GET /inventory/item/:item_id
    #[tracing::instrument(skip(state))]
    pub async fn get_by_item(
        State(state): State<Arc<AppState>>,
        axum::extract::Path(item_id): axum::extract::Path<String>,
    ) -> HttpResponse<Vec<Inventory>> {
        tracing::debug!(item = %item_id, "Fetching event history for item");

        let events = sqlx::query_as!(
            Inventory,
            "SELECT id, item_id, location_id, value,
        action_type as 'action_type: ActionType',
        CAST(timestamp AS TEXT) as 'timestamp: String',
        move_id as 'move_id: String'
 FROM inventory
 WHERE item_id = ?
 ORDER BY timestamp DESC",
            item_id,
        )
        .fetch_all(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!(item = %item_id, error = %e, "Failed to fetch history for item");
            AppError::SqlError(e)
        })?;

        tracing::info!(item = %item_id, count = events.len(), "Item history retrieved");

        Ok((StatusCode::OK, Json(events)))
    }

    pub fn routes() -> axum::Router<Arc<AppState>> {
        axum::Router::new()
            .route("/", post(Self::log_event).get(Self::get_all))
            .route("/move", post(Self::move_item))
            .route("/stock", get(Self::get_stock))
            .route("/item/{item_id}", get(Self::get_by_item))
    }
}
