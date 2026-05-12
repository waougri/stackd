use crate::schema::AppError;
use crate::state::AppState;
use async_trait::async_trait; // add this
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub user_id: String,
    pub exp: usize,
}

#[async_trait] // add this
impl FromRequestParts<Arc<AppState>> for Claims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, &())
                .await
                .map_err(|_| AppError::NotAuthorized("Missing or invalid token".into()))?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
            &Validation::default(),
        )
            .map_err(|e| {
                tracing::error!("JWT Validation Error: {}", e);
                AppError::NotAuthorized("Token expired or invalid".into())
            })?;

        Ok(token_data.claims)
    }
}