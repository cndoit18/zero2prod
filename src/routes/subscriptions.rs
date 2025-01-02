use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use serde::Deserialize;
use sqlx::types::chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(pool): State<PgPool>,
    Form(form_data): Form<FormData>,
) -> Result<String, StatusCode> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscribed_at)
    VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form_data.email,
        form_data.name,
        Utc::now(),
    )
    .execute(&pool)
    .await
    .map_err(|err| {
        println!("Failed to execute query: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(format!("Welcome {}({})!", form_data.name, form_data.email))
}
