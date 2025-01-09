use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form_data, pool),
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %form_data.email,
        subscriber_name = %form_data.name,
    ),
)]
pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(form_data): Form<FormData>,
) -> Result<(), StatusCode> {
    let new_subscriber = NewSubscriber {
        name: SubscriberName::parse(form_data.name.clone()).map_err(|_| StatusCode::BAD_REQUEST)?,
        email: SubscriberEmail::parse(form_data.email.clone())
            .map_err(|_| StatusCode::BAD_REQUEST)?,
    };
    insert_subscriber(&pool, &new_subscriber)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
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
    Ok(())
}
