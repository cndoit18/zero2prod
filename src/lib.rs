pub mod configuration;
pub mod routes;

use axum::{
    routing::{get, post},
    serve::Serve,
    Router,
};
use sqlx::PgPool;
use tokio::net::TcpListener;

pub fn run(listener: TcpListener, pool: PgPool) -> Result<Serve<Router, Router>, std::io::Error> {
    let app: Router = Router::new()
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(pool);
    Ok(axum::serve(listener, app))
}
