use std::sync::Arc;

use axum::{
    extract::{Extension, Form},
    http::StatusCode,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use serde::Deserialize;
use sqlx::{types::time::OffsetDateTime, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    app::State,
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
};

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[derive(Debug)]
pub struct StoreTokenError(sqlx::Error);

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token."
        )
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(state, form),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    Extension(state): Extension<Arc<State>>,
    Form(form): Form<FormData>,
) -> StatusCode {
    let new_subscriber = match form.try_into() {
        Ok(form) => form,
        Err(_) => return StatusCode::UNPROCESSABLE_ENTITY,
    };

    let mut db_transaction = match state.db_pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let subscriber_id = match insert_subscriber(&mut db_transaction, &new_subscriber).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    let subscription_token = generate_subscription_token();
    if store_token(&mut db_transaction, subscriber_id, &subscription_token)
        .await
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if db_transaction.commit().await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    if send_confirmation_email(
        &state.email_client,
        &new_subscriber,
        &state.base_url,
        &subscription_token,
    )
    .await
    .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::CREATED
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(db_pool, new_subscriber)
)]
pub async fn insert_subscriber(
    db_pool: &mut Transaction<'_, Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    //TODO: make status an enum
    sqlx::query!(
        r#"
        INSERT INTO subscription (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        OffsetDateTime::now_utc(),
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    Alphanumeric.sample_string(&mut rng, 25)
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, pool)
)]
pub async fn store_token(
    pool: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscription_token (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id,
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: &NewSubscriber,
    base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token
    );
    let html_body = format!(
        "Welcome to our newsletter!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let text_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(&new_subscriber.email, "Welcome", &html_body, &text_body)
        .await?;
    Ok(())
}
