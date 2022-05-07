use config::{Config, File};
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use sqlx::ConnectOptions;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

#[derive(Deserialize)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

impl DatabaseSettings {
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = match self.require_ssl {
            true => PgSslMode::Require,
            false => PgSslMode::Prefer,
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
            .log_statements(tracing::log::LevelFilter::Trace)
            .to_owned()
    }

    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("Failed to determine the current directory");
    let config_dir = base_path.join("configuration");
    let base_config = File::from(config_dir.join("base"));

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");
    let env_specific_config = File::from(config_dir.join(environment.to_string()));

    let env_config = config::Environment::with_prefix("app").separator("_");

    let settings = Config::builder()
        .add_source(base_config)
        .add_source(env_specific_config)
        .add_source(env_config)
        .build()?;

    settings.try_deserialize()
}

pub enum Environment {
    Local,
    Production,
}

impl ToString for Environment {
    fn to_string(&self) -> String {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
        .to_string()
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{} is not a supported environment. Use either `local` or `production`",
                other
            )),
        }
    }
}
