use config::{Config, ConfigError, File};
use secrecy::{ExposeSecret, Secret};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production"
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(format!("{} is not a supported environment. Use either `local` or `production`.", other))
        }
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = config::Config::default();
        let base_path = std::env::current_dir().expect("Failed to determine the current directory");
        let configuration_directory = base_path.join("configuration");
        let environment: Environment = std::env::var("APP_ENVIRONMENT")
            .unwrap_or_else(|_| "local".into())
            .try_into()
            .expect("Failed to parse APP_ENVIRONMENT.");

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(config::File::from(configuration_directory.join("base")))
            .add_source(config::File::from(configuration_directory.join(environment.as_str())))
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        format!("postgres://{}:{}@{}:{}/{}", self.username, self.password.expose_secret(), self.host, self.port, self.database_name).into()
    }

    pub fn connection_string_without_db(&self) -> Secret<String> {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password.expose_secret(), self.host, self.port
        ).into()
    }
}