use axum::{extract::Query, http::StatusCode};
use serde::Deserialize;

#[tracing::instrument(name = "Confirm a pending subscriber", skip(_parameters))]
pub async fn confirm(_parameters: Query<Parameters>) -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
pub struct Parameters {
    pub subscription_token: String,
}
