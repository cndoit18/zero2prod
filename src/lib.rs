use axum::{http::StatusCode, routing::get, serve::Serve, Router};
use tokio::net::TcpListener;

pub fn run(listener: TcpListener) -> Result<Serve<Router, Router>, std::io::Error> {
    let app: Router = Router::new().route("/health_check", get(health_check));
    Ok(axum::serve(listener, app))
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
