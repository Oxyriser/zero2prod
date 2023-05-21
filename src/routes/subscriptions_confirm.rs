use std::sync::Arc;

use anyhow::Context;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::app::State as AppState;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[derive(Debug, Error)]
pub enum ConfirmError {
    #[error("Non-existing token")]
    NonExistingToken,

    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for ConfirmError {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self);
        match self {
            Self::NonExistingToken => StatusCode::UNAUTHORIZED,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
        .into_response()
    }
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(query, state))]
pub async fn confirm(
    State(state): State<Arc<AppState>>,
    query: Query<Parameters>,
) -> Result<StatusCode, ConfirmError> {
    let id = get_subscriber_id_from_token(&state.db_pool, &query.subscription_token)
        .await
        .context("Failed to get an optional subscriber id from token")?;

    match id {
        None => Err(ConfirmError::NonExistingToken),
        Some(subscriber_id) => {
            confirm_subscriber(&state.db_pool, subscriber_id)
                .await
                .context("Failed to confirm subscriber")?;
            Ok(StatusCode::OK)
        },
    }
}

#[tracing::instrument(name = "Get subscriber_id from token", skip(subscription_token, pool))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query_scalar!(
        r#"SELECT subscriber_id FROM subscription_token WHERE subscription_token = $1"#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, pool))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscription SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
