use once_cell::sync::Lazy;
use secrecy::SecretString;
use sqlx::PgPool;
use zero2prod::{
    configuration::*,
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscruber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(default_filter_level, subscruber_name, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(default_filter_level, subscruber_name, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format! {"{}/subscriptions", &self.address})
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = Settings {
        database: DatabaseSettings {
            host: "postgres".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: SecretString::from("password"),
            database_name: "newsletter".to_string(),
        },
        application: ApplicationSettings {
            port: 0,
            host: "0.0.0.0".to_string(),
        },
        email_client: EmailClientSettings {
            sender_email: "cndoit18@outlook.com".to_string(),
            email_service: EmailService::Smtp(SmtpSettings {
                host: "mailtutan".to_string(),
                port: 1025,
                username: None,
                password: None,
            }),
            timeout_milliseconds: 10000,
        },
    };
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to build application.");
    let address = format!("http://127.0.0.1:{}", application.port());
    tokio::spawn(application.run_until_stopped());
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
    }
}
