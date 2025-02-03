use axum::{
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::startup::ApplicationState;

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters, state))]
pub async fn confirm(
    State(state): State<ApplicationState>,
    parameters: Query<Parameters>,
) -> Result<(), StatusCode> {
    if let Some(id) = get_subscriber_id_from_token(&state.pool, &parameters.subscription_token)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    {
        if confirm_subscriber(&state.pool, id).await.is_err() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        return Ok(());
    }
    Err(StatusCode::UNAUTHORIZED)
}

#[derive(Deserialize)]
pub struct Parameters {
    pub subscription_token: String,
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(pool, subscriber_id))]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
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

#[tracing::instrument(name = "Get subscriber_id from token", skip(pool, subscription_token))]
pub async fn get_subscriber_id_from_token(
    pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?
    .map(|r| r.subscriber_id))
}
