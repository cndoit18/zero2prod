use axum::{
    extract::{Form, State},
    http::StatusCode,
    response,
};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
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
    insert_subscriber(&state.pool, &new_subscriber)
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
    INSERT INTO subscriptions (id, email, name, subscribed_at, status)
    VALUES ($1, $2, $3, $4, 'confirmed')
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
