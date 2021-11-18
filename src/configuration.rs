use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub server: ServerSettings,
}

#[derive(Deserialize)]
pub struct ServerSettings {
    pub host: String,
    pub application_port: u16,
}

impl ServerSettings {
    pub fn public_addr(&self) -> String {
        format!("{}:{}", &self.host, &self.application_port)
    }

    pub fn application_port(&self) -> u16 {
        self.application_port
    }

    pub fn host(&self) -> &str {
        &self.host
    }
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let mut declared_settings = config::Config::default();

    declared_settings.merge(config::File::with_name("configuration"))?;

    let declared_settings: Settings = declared_settings.try_into()?;

    Ok(declared_settings)
}
