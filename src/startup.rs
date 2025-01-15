use std::sync::Arc;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes,
};
use axum::{
    routing::{get, post},
    serve::Serve,
    Router,
};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub struct Application {
    port: u16,
    server: Serve<Router, Router>,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let pool = get_connection_pool(&configuration.database);

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to migrate db.");

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");

        let email_client = EmailClient::new(
            configuration
                .email_client
                .connection_string()
                .expose_secret(),
            sender_email,
            configuration.email_client.timeout(),
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address).await?;
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, pool, email_client)?;

        Ok(Self { port, server })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    pool: PgPool,
    email_client: EmailClient,
) -> Result<Serve<Router, Router>, std::io::Error> {
    let app: Router = Router::new()
        .layer(TraceLayer::new_for_http())
        .route("/health_check", get(routes::health_check))
        .route("/subscriptions", post(routes::subscribe))
        .with_state(pool)
        .with_state(Arc::new(email_client));
    Ok(axum::serve(listener, app))
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPool::connect_lazy(configuration.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.")
}
