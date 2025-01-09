pub mod configuration;
pub mod domain;
pub mod routes;
pub mod telemetry;

use axum::{
    routing::{get, post},
    serve::Serve,
    Router,
};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Serve<Router, Router>, std::io::Error> {
    let app: Router = Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(pool);
    Ok(axum::serve(listener, app))
}
