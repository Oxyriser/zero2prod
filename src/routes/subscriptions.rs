use std::sync::Arc;

use axum::extract::{Extension, Form};
use axum::http::StatusCode;
use chrono::Utc;
use serde::Deserialize;
use tracing::Instrument;
use uuid::Uuid;

use crate::app::State;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    Form(form): Form<FormData>,
    Extension(state): Extension<Arc<State>>,
) -> StatusCode {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name,
    );
    let _request_span_guard = request_span.enter();

    let query_span = tracing::info_span!("Saving new subscriber details in the database.");

    let res = sqlx::query!(
        r#"
        INSERT INTO subscription (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&state.db_pool)
    .instrument(query_span)
    .await;

    match res {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}
