use secrecy::ExposeSecret;
use secrecy::SecretString;
use serde::Deserialize;
use url::Url;
use validator::ValidateEmail;

use crate::domain::SubscriberEmail;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(Deserialize)]
pub struct EmailClientSettings {
    pub sender_email: String,
    pub email_service: EmailService,
}

#[derive(Deserialize)]
pub enum EmailService {
    #[serde(rename = "smtp")]
    Smtp(SmtpSettings),
}

#[derive(Deserialize)]
pub struct SmtpSettings {
    pub host: String,
    pub port: u32,
    pub username: SecretString,
    pub password: SecretString,
}

impl EmailClientSettings {
    pub fn connection_string(&self) -> SecretString {
        match &self.email_service {
            EmailService::Smtp(smtp_settings) => {
                SecretString::from(Url::parse(&format! {
                    "smtp://{}:{}@{}:{}",  smtp_settings.username.expose_secret(), smtp_settings.password.expose_secret(),smtp_settings.host, smtp_settings.port,
                }).unwrap().as_str())
            }
        }
    }
}

impl EmailClientSettings {
    pub fn sender(&self) -> Result<SubscriberEmail, String> {
        {
            let s = self.sender_email.clone();
            if ValidateEmail::validate_email(&s) {
                return SubscriberEmail::parse(s);
            }
            Err(format!("{} is not a valid subscriber email.", s))
        }
    }
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: SecretString,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> SecretString {
        SecretString::from(format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        ))
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new(
            "configuration.yaml",
            config::FileFormat::Yaml,
        ))
        .build()?;
    settings.try_deserialize::<Settings>()
}
