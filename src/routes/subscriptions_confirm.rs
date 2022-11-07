use axum::extract::Query;
use axum::http::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_query))]
pub async fn confirm(_query: Query<Parameters>) -> StatusCode {
    StatusCode::OK
}
