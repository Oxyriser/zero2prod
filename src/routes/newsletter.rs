use axum::{extract::Json, http::StatusCode};

#[derive(serde::Deserialize)]
pub struct Newsletter {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[tracing::instrument(name = "Publish a newsletter", skip(newsletter))]
pub async fn publish_newsletter(Json(newsletter): Json<Newsletter>) -> StatusCode {
    StatusCode::OK
}
