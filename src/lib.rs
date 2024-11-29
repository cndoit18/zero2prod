use axum::{
    extract::Form,
    http::StatusCode,
    routing::{get, post},
    serve::Serve,
    Router,
};
use serde::Deserialize;
use tokio::net::TcpListener;

pub fn run(listener: TcpListener) -> Result<Serve<Router, Router>, std::io::Error> {
    let app: Router = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe));
    Ok(axum::serve(listener, app))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Deserialize)]
struct FormData {
    email: String,
    name: String,
}

async fn subscribe(Form(form_data): Form<FormData>) -> Result<String, StatusCode> {
    Ok(format!("Welcome {}({})!", form_data.name, form_data.email))
}
