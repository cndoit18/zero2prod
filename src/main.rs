use secrecy::ExposeSecret;
use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::configuration::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::run;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber(
        "zero2prod".into(),
        "trace,tower_http=trace".into(),
        std::io::stdout,
    );
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let pool = PgPool::connect_lazy(configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres.");

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
    );

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate db.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    run(TcpListener::bind(address).await?, pool, email_client)?.await
}
