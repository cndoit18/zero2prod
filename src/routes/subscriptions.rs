use std::iter::repeat_with;

use axum::{
    extract::{Form, State},
    http::StatusCode,
    response,
};
use rand::prelude::*;
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::ApplicationState,
};

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        Ok(Self {
            name: SubscriberName::parse(value.name.clone())?,
            email: SubscriberEmail::parse(value.email.clone())?,
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form_data, state),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form_data.email,
        subscriber_name = %form_data.name,
    ),
)]
pub async fn subscribe(
    State(state): State<ApplicationState>,
    Form(form_data): Form<FormData>,
) -> response::Result<(), StatusCode> {
    let new_subscriber: NewSubscriber =
        form_data.try_into().map_err(|_| StatusCode::BAD_REQUEST)?;
    let subscriber_id = insert_subscriber(&state.pool, &new_subscriber)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let subscription_token = generate_subscription_token();

    store_token(&state.pool, subscriber_id, &subscription_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    send_confirmation_email(
        &state.email_client,
        new_subscriber,
        state.base_url,
        &subscription_token,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: String,
    subscription_token: &str,
) -> Result<(), String> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        base_url, subscription_token,
    );
    email_client
        .send_email(
            new_subscriber.email,
            "Welcome newsletter!",
            &format!(
                "Welcome to our newsletter!<br />
    Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link,
            ),
        )
        .await
}
pub async fn store_token(
    pool: &PgPool,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

fn generate_subscription_token() -> String {
    let mut rng = rand::rng();
    repeat_with(|| rng.sample(rand::distr::Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now(),
    )
    .execute(pool)
    .await
    .map_err(|err| {
        tracing::error!("Failed to execute query: {:?}", err);
        err
    })?;
    Ok(subscriber_id)
}
