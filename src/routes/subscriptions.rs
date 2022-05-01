use std::sync::Arc;

use axum::extract::{Extension, Form};
use axum::http::StatusCode;
use chrono::Utc;
use serde::Deserialize;
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
    .await;

    match res {
        Ok(_) => StatusCode::CREATED,
        Err(e) => {
            println!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        },
    }
}
